// Copyright 2015 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Main public pull parse interface, running two passes over input.

use parse::{RawParser, Event, Tag, ParseInfo};
use std::vec;
use std::iter::IntoIterator;

pub struct Parser<'a> {
	events: vec::IntoIter<(Event<'a>, usize)>,
	offset: usize,
	info: ParseInfo,
	loose_stack: Vec<bool>,
}

impl<'a> Parser<'a> {
	pub fn new(text: &'a str) -> Parser<'a> {
		// first pass, collecting info
		let mut inner = RawParser::new(text);
		let mut event_vec = Vec::new();
		loop {
			match inner.next() {
				Some(event) => event_vec.push((event, inner.get_offset())),
				None => break
			}
		}

		// second pass runs in iterator
		let events = event_vec.into_iter();
		let info = inner.get_info();
		//println!("loose lists: {:?}", info.loose_lists);
		Parser {
			events: events,
			offset: 0,
			info: info,
			loose_stack: Vec::new(),
		}
	}

	pub fn get_offset(&self) -> usize {
		self.offset
	}
}

impl<'a> Iterator for Parser<'a> {
	type Item = Event<'a>;

	fn next(&mut self) -> Option<Event<'a>> {
		loop {
			match self.events.next() {
				Some((event, offset)) => {
					self.offset = offset;
					match event {
						Event::Start(Tag::List(_)) => {
							let is_loose = self.info.loose_lists.contains(&offset);
							self.loose_stack.push(is_loose);
						}
						Event::Start(Tag::BlockQuote) => {
							self.loose_stack.push(true);
						}
						Event::Start(Tag::Paragraph) | Event::End(Tag::Paragraph) => {
							if let Some(&false) = self.loose_stack.last() {
								continue;
							}
						}
						Event::End(Tag::List(_)) | Event::End(Tag::BlockQuote) => {
							let _ = self.loose_stack.pop();
						}
						_ => ()
					}
					return Some(event)
				}
				None => return None
			}
		}
	}
}
