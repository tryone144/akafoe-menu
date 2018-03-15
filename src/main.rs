// MIT License
// 
// Copyright (c) 2016 Bernd Busse
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! # akafoe-menu
//!
//! Get today's menus from http://www.akafoe.de
//!

extern crate quick_xml;
extern crate regex;
extern crate reqwest;
extern crate time;

mod etree;

use std::io::BufReader;
use std::{fmt, process, str};

use etree::{ETBuilder, ETElement};
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;


static FEED_URL_MENSA: &'static str = "http://www.akafoe.de/gastronomie/speiseplaene-der-mensen/ruhr-universitaet-bochum/\
                                       ?mid=1?tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";
static FEED_URL_BISTRO: &'static str = "http://www.akafoe.de/gastronomie/speiseplaene-der-mensen/bistro-der-ruhr-universitaet-bochum/\
                                        ?mid=37?tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";
static FEED_URL_QWEST: &'static str = "http://www.akafoe.de/gastronomie/gastronomien/q-west/\
                                       ?mid=38?tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";
static FEED_URL_HENKELMANN: &'static str = "http://www.akafoe.de/gastronomie/henkelmann/\
                                            ?mid=21&tx_akafoespeiseplan_mensadetails%5Baction%5D=feed&tx_akafoespeiseplan_mensadetails%5Bcontroller%5D=AtomFeed";


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
                .captures(description.as_ref());

        let name = match captures.as_ref().and_then(|c| c.get(1)) {
            Some(name) => Regex::new(r"(?:\s*)([,.:])(?:\s*)").unwrap().replace_all(name.as_str(), "$1 ").into_owned(),
            None => {
                println!("Warning: Cannot get description of menu!");
                "No description".to_owned()
            }
        };
        let info = captures.as_ref()
            .and_then(|c| c.get(2))
            .map(|c| c.as_str())
            .unwrap_or("");
        let price_student = captures.as_ref()
            .and_then(|c| c.get(3))
            .map(|p| price_filter.replace_all(p.as_str(), "."))
            .and_then(|p| p.parse::<f32>().ok())
            .unwrap_or(0.0);
        let price_regular = captures.as_ref()
            .and_then(|c| c.get(4))
            .map(|p| price_filter.replace_all(p.as_str(), "."))
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
            title: Regex::new(r"\s+").unwrap().replace_all(title, " ").into_owned(),
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

        let mut parser = Reader::from_reader(reader);
        let mut builder = ETBuilder::new();
        let mut buf = Vec::new();
        parser.trim_text(true);

        let mut feed = ETElement::default();
        loop {
            match parser.read_event(&mut buf) {
                Err(e) => panic!("Ooops an error occured! {:?}", e),
                Ok(Event::Eof) => break,
                Ok(ev) => {
                    if let Some(elem) = builder.handle_event(&parser, ev) {
                        feed = elem;
                    }
                },
            }
            buf.clear();
        }
        
        let feed_items = feed.get_children_ref();
        let title = match feed_items.iter().find(|e| e.name == "title").map(|e| e.get_text()) {
            Some(text) => Regex::new(r"\s+").unwrap().replace_all(text.as_str(), " ").into_owned(),
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
                .map(|e| date_exp.replace_all(e.get_text().as_ref(), "$1").into_owned()) {
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
    //for facility in vec![FEED_URL_QWEST, FEED_URL_HENKELMANN] {
        let response = match reqwest::get(facility) {
            Ok(resp) => resp,
            Err(e) => {
                println!("Unable to load menu: {}", e.to_string());
                process::exit(1);
            }
        };
        //println!("The response: {:?}", response);
        //use std::io::Read;
        //let mut reader = BufReader::new(response);
        //let mut buf = Vec::new();
        //reader.read_to_end(&mut buf);
        //println!("    {}", String::from_utf8(buf).unwrap());
        //return;
        let reader = BufReader::new(response);
        let menu = Menu::from_reader(reader);

        let title = format!(":: {}", menu);
        println!("{}\n{:=<width$}", title, "=", width = title.len());
        for sec in menu.sections.iter() {
            if sec.meals.len() == 0 {
                continue;
            }
            println!("  :: {}", sec);
            for meal in sec.meals.iter() {
                println!("     * {}", meal);
            }
        }
        println!("");
    }
}
