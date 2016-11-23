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

use std::fmt;
use std::str;

use quick_xml::Event;


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

pub struct ETBuilder {
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
