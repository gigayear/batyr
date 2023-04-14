// Batyr Document Writer
// Copyright (C) 2023 Gene Yu
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <https://www.gnu.org/licenses/>.

//! Writes formatted pages to the standard output
//!
//! # Examples
//!
//! ```rust,no_run
//! use batyr::document::formatter::Page;
//! use batyr::document::writer::Writer;
//! use batyr::text::{Line, Segment};
//!
//! let page = Page {
//!     number: 1,
//!     height: 55,
//!     lines: vec![Some(Line::from(Segment::from("foo")))],
//!     footer: Vec::new(),
//! };
//!
//! let mut writer = Writer::new("WORKING TITLE");
//! let result = writer.run(vec![page]);
//! ```
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::str;

use encoding::{Encoding, EncoderTrap};
use encoding::all::ISO_8859_15;
use regex::Regex;

use crate::PROGRAM_NAME;
use crate::PROLOGUE_FILE;

use crate::document::*;
use crate::document::formatter::*;
use crate::text::*;

/// Output driver
pub struct Writer {
    title: String,
    real_page_no: usize,
}

impl Writer {
    /// Creates a document writer
    pub fn new(title: &str) -> Writer {
        Writer {
            title: title.to_string(),
            real_page_no: 1,
        }
    }

    /// Writes the document to the standard output
    pub fn run(&mut self, pages: PageList) -> Result<(), Box<dyn Error>> {
        self.write_prologue(pages.len())?;

        for page in pages {
            self.start_a_new_page(page.number)?;

            let mut y = (TOP_LINE as f32 * LINE_HEIGHT as f32).round() as i32;

            for line in page.lines {
                match line {
                    Some(line) => {
                        let x = (line.column as f32 * CHAR_WIDTH).round() as i32;

                        writeln(&format!("{} {} moveto {}", x, y, line.ps()))?;

                        y -= LINE_HEIGHT.round() as i32;
                    },
                    None => {
                        y -= LINE_HEIGHT.round() as i32;
                    },
                }
            }

            if !page.footer.is_empty() {
                y = ((BOTTOM_LINE + page.footer.len() - 1) as f32 * LINE_HEIGHT)
                    .round() as i32;

                for line in page.footer.iter() {
                    match line {
			Some(line) => {
                            let x = (line.column as f32 * CHAR_WIDTH).round() as i32;
		            writeln(&format!("{} {} moveto {}", x, y, line.ps()))?;
                            y -= LINE_HEIGHT.round() as i32;
			},
			None => {
                            y -= LINE_HEIGHT.round() as i32;
			},
                    }
                }
            }

            writeln("page-end")?;
        }

        writeln("%%Trailer")
    }

    #[doc(hidden)]
    fn write_prologue(&mut self, page_count: usize) -> Result<(), Box<dyn Error>> {
        let   title_pat = Regex::new(r"@title@")?;
        let creator_pat = Regex::new(r"@creator@")?;
        let   pages_pat = Regex::new(r"@pages@")?;

        let creator = PROGRAM_NAME.to_string();
	
        let num_pages = format!("{}", page_count);
        let mut prologue = fs::read_to_string(&*PROLOGUE_FILE)?;

        prologue = title_pat.replace(&prologue, &self.title).to_string();
        prologue = creator_pat.replace(&prologue, &creator).to_string();
        prologue = pages_pat.replace(&prologue, &num_pages).to_string();

        write(&prologue)
    }

    #[doc(hidden)]
    fn start_a_new_page(&mut self, page_no: i32) -> Result<(), Box<dyn Error>> {
        writeln(&format!("%%Page: {} {}", self.real_page_no, self.real_page_no))?;
        writeln("page-begin")?;

        if page_no > 0 {
            let s = format!("{}.", page_no);
            let x = (PAGE_NO_BEGIN as f32 * CHAR_WIDTH).round() as i32;
            let y = (HEADER_LINE as f32 * LINE_HEIGHT as f32).round() as i32;
            let line = Line::from(Segment::from(s));
            writeln(&format!("{} {} moveto {}", x, y, line.ps()))?;
        }
        
        self.real_page_no += 1;

        Ok(())
    }
}

/// Converts UTF-8 characters to ISO/IEC 8859-15 and writes them to
/// the standard output
fn write(text: &str) -> Result<(), Box<dyn Error>> {
    let chars = ISO_8859_15.encode(text, EncoderTrap::Replace)?;
    io::stdout().write_all(&chars)?;
    Ok(())
}

/// Converts UTF-8 characters to ISO/IEC 8859-15 and writes them to
/// the standard output, appending a newline
fn writeln(text: &str) -> Result<(), Box<dyn Error>> {
    let mut chars = ISO_8859_15.encode(text, EncoderTrap::Replace)?;
    chars.push(b'\n');
    io::stdout().write_all(&chars)?;
    Ok(())
}
