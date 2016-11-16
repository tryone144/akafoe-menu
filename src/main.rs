//! # akafoe-menu
//!
//! Get the menu for today

extern crate quick_xml;
extern crate regex;
extern crate reqwest;
extern crate time;

use std::io::BufReader;
use std::fmt;
use std::str;

use quick_xml::{XmlReader, Event};
use regex::Regex;

static FEED_URL_MENSA: &'static str = "http://www.akafoe.\
                                       de/gastronomie/speisepläne-der-mensen/ruhr-universitaet-bochum/?mid=1?tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";
static FEED_URL_BISTRO: &'static str = "http://www.akafoe.\
                                        de/gastronomie/speisepläne-der-mensen/bistro-der-ruhr-universitaet-bochum/?mid=37?tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";
static FEED_URL_QWEST: &'static str = "http://www.akafoe.\
                                       de/gastronomie/gastronomien/q-west/?mid=38?tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";
static FEED_URL_HENKELMANN: &'static str = "http://www.akafoe.\
                                            de/gastronomie/henkelmann/?mid=21&tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";


enum ETNode {
    ElementNode(ETElement),
    TextNode(String),
}

impl fmt::Display for ETNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ETNode::ElementNode(ref element) => fmt::Display::fmt(element, f),
            ETNode::TextNode(ref text) => fmt::Display::fmt(text, f),
        }
    }
}

struct ETElement {
    name: String,
    children: Vec<ETNode>,
}

impl ETElement {
    pub fn get_children_ref(&self) -> Vec<&ETElement> {
        self.children
            .iter()
            .filter_map(|c| match *c {
                ETNode::ElementNode(ref e) => Some(e),
                ETNode::TextNode(_) => None,
            })
            .collect::<Vec<&ETElement>>()
    }

    pub fn get_text(&self) -> String {
        self.children
            .iter()
            .filter_map(|c| match *c {
                ETNode::ElementNode(_) => None,
                ETNode::TextNode(ref text) => Some(text),
            })
            .fold(String::new(), |mut res, text| {
                if res.len() > 0 {
                    res.push(' ');
                }
                res.push_str(text.as_str());
                res
            })
    }
}

impl fmt::Display for ETElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}

struct ETBuilder {
    stack: Vec<ETElement>,
}

impl ETBuilder {
    pub fn new() -> Self {
        ETBuilder { stack: Vec::new() }
    }

    pub fn handle_event<T>(&mut self, ev: Result<Event, T>) -> Option<Result<ETElement, T>> {
        let ev = match ev {
            Ok(event) => event,
            Err(e) => return Some(Err(e)),
        };

        match ev {
            Event::Start(ref e) => {
                let elem = ETElement {
                    name: str::from_utf8(e.name()).unwrap().to_owned(),
                    children: Vec::new(),
                };
                self.stack.push(elem);
            }
            Event::End(ref e) => {
                let elem = self.stack.pop().unwrap_or_else(|| panic!("improper nesting"));
                if elem.name != str::from_utf8(e.name()).unwrap() {
                    panic!("improper nesting");
                } else {
                    match self.stack.last_mut() {
                        Some(parent) => parent.children.push(ETNode::ElementNode(elem)),
                        None => return Some(Ok(elem)),
                    }
                }
            }
            Event::Text(ref e) => {
                if let Some(current) = self.stack.last_mut() {
                    current.children
                        .push(ETNode::TextNode(str::from_utf8(e.content()).unwrap().to_owned()));
                }
            }
            _ => {}
        }

        None
    }
}

struct Meal {
    pub desc: String,
    pub info: String,
    pub price_student: f32,
    pub price_regular: f32,
}

impl Meal {
    fn new(description: &str) -> Self {
        let description = Regex::new(r"\s+").unwrap().replace_all(description, " ");
        let price_filter = Regex::new(r",").unwrap();

        let captures =
            Regex::new(r"^([^()]+\S)\s+((?:\(.*\)\s+){0,2})([\d,]+)\s*EUR\s*-\s*([\d,]+)\s*EUR$")
                .unwrap()
                .captures(description.as_str());

        let name = match captures.as_ref().and_then(|c| c.at(1)) {
            Some(name) => Regex::new(r"(?:\s*)([,.:])(?:\s*)").unwrap().replace_all(name, "$1 "),
            None => {
                println!("Warning: Cannot get description of menu!");
                "No description".to_owned()
            }
        };
        let info = captures.as_ref().and_then(|c| c.at(2)).unwrap_or("");
        let price_student = captures.as_ref()
            .and_then(|c| c.at(3))
            .map(|p| price_filter.replace_all(p, "."))
            .and_then(|p| p.parse::<f32>().ok())
            .unwrap_or(0.0);
        let price_regular = captures.as_ref()
            .and_then(|c| c.at(4))
            .map(|p| price_filter.replace_all(p, "."))
            .and_then(|p| p.parse::<f32>().ok())
            .unwrap_or(0.0);

        Meal {
            desc: name.trim().to_owned(),
            info: info.trim().to_owned(),
            price_student: price_student,
            price_regular: price_regular,
        }
    }
}

impl fmt::Display for Meal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = format!("{:70} {:>15}", self.desc, self.info);
        write!(f,
               "{:} \t{:.2}€ / {:.2}€",
               description,
               self.price_student,
               self.price_regular)
    }
}

struct Section {
    pub title: String,
    pub meals: Vec<Meal>,
}

impl Section {
    pub fn new(title: &str) -> Self {
        Section {
            title: Regex::new(r"\s+").unwrap().replace_all(title, " "),
            meals: Vec::new(),
        }
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

struct Menu {
    pub title: String,
    date: String,
    pub sections: Vec<Section>,
}

impl Menu {
    pub fn from_reader<R: std::io::Read>(reader: BufReader<R>) -> Self {
        let date_exp = Regex::new(r"^.*/(\d{2}-\d{2}-\d{2})").unwrap();

        let now = time::now();
        let today = time::strftime("%y-%m-%d", &now).unwrap();

        let parser = XmlReader::from_reader(reader).trim_text(true);
        let mut builder = ETBuilder::new();

        let feed = match parser.filter_map(|ev| builder.handle_event(ev)).next().unwrap() {
            Ok(element) => element,
            Err((e, pos)) => panic!("Error while parsing at {}: {}", pos, e),
        };
        let feed_items = feed.get_children_ref();

        let title = match feed_items.iter().find(|e| e.name == "title").map(|e| e.get_text()) {
            Some(text) => Regex::new(r"\s+").unwrap().replace_all(text.as_str(), " "),
            None => {
                println!("Warning: Menu has no title!");
                "Unkown".to_owned()
            }
        };

        let mut sections: Vec<Section> = Vec::new();
        for entry in feed_items.iter().filter(|e| e.name == "entry") {
            let entry_items = entry.get_children_ref();

            let date = match entry_items.iter()
                .find(|e| e.name == "id")
                .map(|e| date_exp.replace_all(e.get_text().as_str(), "$1")) {
                Some(date) => date.clone(),
                None => {
                    println!("Warning: Menu is missing date!");
                    continue;
                }
            };
            if date != today {
                continue;
            }

            let content_items = match entry_items.iter()
                .find(|e| e.name == "content")
                .map(|e| e.get_children_ref())
                .and_then(|c| c.first().map(|e| e.get_children_ref())) {
                Some(items) => items,
                None => {
                    println!("Warning: No meals found!");
                    break;
                }
            };
            for item in content_items.iter() {
                if item.name == "p" {
                    let sec_title = item.get_children_ref()
                        .first()
                        .map(|e| e.get_text())
                        .unwrap_or("Unkown".to_owned());
                    sections.push(Section::new(sec_title.as_str()));
                } else if item.name == "ul" {
                    if let Some(sec) = sections.last_mut() {
                        for m in item.get_children_ref().iter().filter(|e| e.name == "li") {
                            sec.meals.push(Meal::new(m.get_text().as_str()));
                        }
                    } else {
                        panic!("No Section found!");
                    }
                }
            }

            break;
        }

        if sections.len() == 0 {
            println!("Warning: no meal section found for {}!", today);
        }

        Menu {
            title: title,
            date: time::strftime("%d.%m.%y", &now).unwrap(),
            sections: sections,
        }
    }
}

impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.title, self.date)
    }
}

fn main() {
    println!(r"         __         ____");
    println!(r"  ____ _/ /______ _/ __/___  ___     ____ ___  ___  ____  __  __");
    println!(r" / __ `/ //_/ __ `/ /_/ __ \/ _ \   / __ `__ \/ _ \/ __ \/ / / /");
    println!(r"/ /_/ / ,< / /_/ / __/ /_/ /  __/  / / / / / /  __/ / / / /_/ /");
    println!(r"\__,_/_/|_|\__,_/_/  \____/\___/  /_/ /_/ /_/\___/_/ /_/\__,_/");
    println!("");

    for facility in vec![FEED_URL_MENSA, FEED_URL_BISTRO, FEED_URL_QWEST, FEED_URL_HENKELMANN] {
        let response = reqwest::get(facility).expect("Cannot load request");
        let reader = BufReader::new(response);
        let menu = Menu::from_reader(reader);

        let title = format!(":: {}", menu);
        println!("{}\n{:=<width$}", title, "=", width = title.len());
        for sec in menu.sections.iter() {
            println!("  :: {}", sec);
            for meal in sec.meals.iter() {
                println!("     * {}", meal);
            }
        }
        println!("");
    }
}
