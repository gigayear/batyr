// Batyr Library
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

//! An XML-to-Postscript converter for creating typed screenplays on a
//! home or office printer
//!
//! Produces typewriter output in standard screenplay format.  In the
//! film industry today, proprietary software is the norm.  For those
//! of us who are not in the industry, proprietary software is too
//! expensive, and it will put our work in jeopardy one day when we
//! need to foot the bill for an upgrade we don't want.
//!
//! Nothing can beat plain text for the long-term security of the
//! words we write, but formatting is crucial for screenplays.  Enter
//! XML, which allows us to add semantics in text form.  We don't need
//! much.  The screenplay format is actually simpler than it looks;
//! all of the finesse is in breaking the pages correctly.
//!
//! That said, Batyr is kind of stupid, relatively speaking.  It does
//! not really "know" the format; _you_ need to know the format.  But
//! if you do know what you want to see on the page, you should find
//! that it is fairly easy to express using the XML schema provided.
//!
//! The focus here is on the mostly unadorned spec script, but there
//! is minimal support for scene numbers because it's fun, and because
//! having a different look can sometimes help with visualization.
//! The target audience for this software is writers who are looking
//! for a robust modern way to produce old-fashioned typed hard copy.
//!
//! This crate is named after an elephant from Kazakhstan who
//! addressed the Soviet Union on state television in 1980.
//!
//! Pronunciation (IPA): ,bɑ‘tir
//!
//! # Examples
//!
//! Processing a valid document, an encoding of the shooting script
//! for _It's a Wonderful Life_, by Frances Goodrich, Albert Hackett,
//! Frank Capra and Jo Swerling:
//!
//! ```sh
//! $ head -4 goodrich.tyr
//! <?xml version="1.0" encoding="utf-8"?>
//! <screenplay
//!   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
//!   xsi:noNamespaceSchemaLocation="http://www.matchlock.com/batyr/screenplay.xsd">
//! $ wc -l goodrich.tyr
//! 9909 goodrich.tyr
//! $ xmllint --noout --schema screenplay.xsd goodrich.tyr
//! goodrich.tyr validates
//! $ batyr goodrich.tyr > goodrich.ps
//! $ head -6 goodrich.ps
//! %!PS
//! %%Title: IT'S A WONDERFUL LIFE
//! %%Creator: batyr
//! %%DocumentFonts: Courier
//! %%BoundingBox: 0 0 612 792
//! %%Pages: 198
//! $
//! ```
//! Output: [`goodrich.pdf`]
//!
//! Batyr can also show you its internal element representation using
//! the <tt>-e</tt> flag.  The internal element representation will
//! also be printed if the input file contains a fragment of the
//! screenplay schema:
//!
//! ```sh
//! $ cat minimal.tyr
//! <br/>
//! $ batyr minimal.tyr
//! Br(EmptyElement { attributes: Br, break_info: Disposable(1) })
//! $
//! ```
//!
//! # References
//! <ol>
//!   <li>Christopher Riley, <em>The Hollywood Standard: The Complete
//!   and Authoritative Guide to Script Format & Style</em>, 3rd
//!   edition (Studio City, CA: Michael Wiese Productions, 2021).</li>
//!   <li>Hillis R. Cole, Jr. and Judith H. Haag, <em>The Complete
//!   Guide to Standard Script Formats</em> (North Hollywood, CA: CMC
//!   Publishing, 1996).</li>
//! </ol>
//!
//! [`goodrich.pdf`]: <http://www.matchlock.com/batyr/goodrich.pdf>

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

use lazy_static::lazy_static;

use crate::document::*;
use crate::document::reader::Reader;
use crate::document::formatter::Formatter;
use crate::document::writer::Writer;

pub mod document;
pub mod text;

// configuration

lazy_static! {
    #[doc(hidden)]
    pub static ref PROGRAM_NAME: String = match get_program_name() {
        Some(name) => name,
        None => "batyr".to_string(),
    };

    /// Path to prologue.ps
    static ref PROLOGUE_FILE: PathBuf
        = PathBuf::from("/home/gene/share/batyr/prologue.ps");
}

/// Command-line arguments
#[derive(Parser, Default, Debug)]
#[clap(author="Gene Yu", version, about="Screenplay Typewriter")]
pub struct Arguments {
    /// An XML file conforming to the screenplay schema
    pub input_file: PathBuf,

    #[clap(short, long)]
    /// Show the internal element representation instead of the usual output.
    pub elements: bool,
}

impl From<&str> for Arguments {
    // This method is for testing.
    fn from(s: &str) -> Self {
        Self {
            input_file: PathBuf::from(s),
            elements: false,
        }
    }
}

/// Reads an XML input string and construct an element hierarchy from
/// its contents
///
/// # Examples
///
/// ```rust,no_run
/// let args = batyr::Arguments::from("dummy.tyr");
/// let root = batyr::read(&args).unwrap();
/// ```
pub fn read(args: &Arguments) -> Result<ElementType, Box<dyn Error>> {
    let xml_string = fs::read_to_string(&args.input_file).unwrap();
    let reader = Reader::new(&xml_string);
    reader.run().ok_or("No elements!".into())
}

/// Writes an element hierarchy to the standard output in Postscript
///
/// # Examples
///
/// ```rust,no_run
/// # let args = batyr::Arguments::from("dummy.tyr");
/// let root = batyr::read(&args).unwrap();
/// batyr::write(root, &args);
/// ```
pub fn write(root: ElementType, args: &Arguments)
             -> Result<(), Box<dyn Error>>
{
    match root {
        ElementType::Screenplay(_) => {
            if args.elements {
                eprintln!("{:?}", &root);
            } else {
                let mut formatter = Formatter::new();
                formatter.run(root);

                let mut writer = Writer::new(&formatter.title);
                writer.run(formatter.body)?;
            }
        },
        _ => eprintln!("{:?}", &root),
    }

    Ok(())
}

#[doc(hidden)]
fn get_program_name() -> Option<String> {
    env::current_exe().ok()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}
