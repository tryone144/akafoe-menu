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

//! # etree
//!
//! Parse XML into basic ElementTree representation.
//!

use std::{default, fmt, io, str};

use quick_xml::{Reader, Error};
use quick_xml::events::{Event, BytesText};


pub enum ETNode {
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


pub struct ETElement {
    pub name: String,
    pub children: Vec<ETNode>,
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
                if !res.is_empty() {
                    res.push(' ');
                }
                res.push_str(text.as_str());
                res
            })
    }
}

impl default::Default for ETElement {
    fn default() -> Self {
        ETElement {
            name: "empty".to_owned(),
            children: Vec::new(),
        }
    }
}

impl fmt::Display for ETElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}


pub struct ETBuilder {
    stack: Vec<ETElement>,
}

impl ETBuilder {
    pub fn new() -> Self {
        ETBuilder { stack: Vec::new() }
    }

    pub fn handle_event<T: io::BufRead>(&mut self, parser: &Reader<T>, ev: Event) -> Option<ETElement> {
        match ev {
            Event::Start(ref ev) => {
                let elem = ETElement {
                    name: str::from_utf8(ev.name()).expect("Get element name of start tag").to_owned(),
                    children: Vec::new(),
                };
                self.stack.push(elem);
            }
            Event::End(ref ev) => {
                let elem = self.stack.pop().unwrap_or_else(|| panic!("improper nesting"));
                if elem.name != str::from_utf8(ev.name()).expect("Get element name of end tag") {
                    panic!("improper nesting");
                } else {
                    match self.stack.last_mut() {
                        Some(parent) => parent.children.push(ETNode::ElementNode(elem)),
                        None => return Some(elem),
                    }
                }
            }
            Event::Text(ref ev) => {
                if let Some(current) = self.stack.last_mut() {
                    let node = ev.unescaped()
                        .map(|x| x.into_owned())
                        .or_else(|err| match err {
                            Error::EscapeError(_) => {
                                let mut new = Vec::new();
                                ev.escaped().iter().for_each(|x| {
                                    new.push(*x);
                                    if char::from(*x) == '&' {
                                        let mut esc = vec!(b'a', b'm', b'p', b';');
                                        new.append(&mut esc);
                                    }
                                });

                                BytesText::from_escaped(new).unescaped().map(|x| x.into_owned())
                            },
                            _ => Err(err),
                        }).expect("Unescaped text node");
                    let text = parser.decode(node.as_ref()).expect("Cannot decode text node");
                    current.children.push(ETNode::TextNode(text.to_owned()));
                }
            }
            _ => {}
        }

        None
    }
}
