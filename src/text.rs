// Batyr Text Module
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

//! Low-level text processing
//!
//! This module defines token lists that represent a stream of typed
//! characters that can be broken up into lines in various ways.  The
//! contents of an XML text element can be fed directly into the
//! parser.

use lazy_static::lazy_static;

use regex::Regex;

use crate::text::tokens::*;

pub mod tokens;
pub mod parser;

/// A line of output
///
/// Different sets of display flags require different Postscript
/// commands.  Each line is split put into [`Segment`]s, based on the
/// number of display state changes there are.
#[derive(Debug, Clone)]
pub struct Line {
    /// The start column
    pub column: usize,
    /// The line segments in order from left to right
    pub segments: Vec<Segment>,
}

impl Line {
    // The total number of characters in the line
    pub fn length(&self) -> usize {
        self.segments.iter().map(|x| { x.text.chars().count() }).sum()
    }

    // The unescaped text
    pub fn text(&self) -> String {
        self.segments.iter().map(|x| { x.text.clone() }).collect()
    }

    // The Postscript command to ouutput the line
    pub fn ps(&self) -> String {
        self.segments.iter().map(|x| { x.ps.clone() }).collect()
    }
}

/// A line segment
///
/// Within a line segment, all of the tokens have the same set of
/// display flags.
#[derive(Debug, Clone)]
pub struct Segment {
    /// The text of the line segment
    pub text: String,
    /// The Postscript command to print the line segment
    pub ps: String,
}

lazy_static! {
    #[doc(hidden)]
    static ref PS_ESC_BACKSLASH: Regex = Regex::new(r"\\").unwrap();

    #[doc(hidden)]
    static ref PS_ESC_OPEN_PAREN: Regex = Regex::new(r"\(").unwrap();

    #[doc(hidden)]
    static ref PS_ESC_CLOSE_PAREN: Regex = Regex::new(r"\)").unwrap();
}

impl From<String> for Segment {
    fn from(s: String) -> Self {
        let mut ps = s.clone();

        ps = PS_ESC_BACKSLASH.replace_all(&ps, "\\\\").to_string();
        ps = PS_ESC_OPEN_PAREN.replace_all(&ps, "\\(").to_string();
        ps = PS_ESC_CLOSE_PAREN.replace_all(&ps, "\\)").to_string();
        
        Self {
            text: s,
            ps: format!("({}) show ", ps),
        }
    }
}

impl From<&str> for Segment {
    fn from(s: &str) -> Self {
        let mut ps = s.to_string();

        ps = PS_ESC_BACKSLASH.replace_all(&ps, "\\\\").to_string();
        ps = PS_ESC_OPEN_PAREN.replace_all(&ps, "\\(").to_string();
        ps = PS_ESC_CLOSE_PAREN.replace_all(&ps, "\\)").to_string();
        
        Self {
            text: s.to_string(),
            ps: format!("({}) show ", ps),
        }
    }
}

impl From<Segment> for Line {
    fn from(segment: Segment) -> Self {
        Self {
            column: 0,
            segments: vec![segment],
        }
    }
}

// Each state change requires a distinct set of Postscript commands.
// The following conversion splits a line of output into segments,
// and intersperses Postscript commands.
impl From<&[TokenType]> for Line {
    fn from(tokens: &[TokenType]) -> Line {
        let mut segments: Vec<Segment> = Vec::new();
        let mut state_changes: Vec<usize> = Vec::new();

        let mut dpy: DisplayFlags = match tokens.first() {
            Some(token) => token.display_flags(),
            None => Default::default(),
        };

        state_changes.push(0);
        
        for (i, token) in tokens.iter().enumerate() {
            match token {
                TokenType::Close(token) => {
                    if dpy != token.dpy {
                        dpy = token.dpy;
                        state_changes.push(i);
                    }
                },
                TokenType::LineBreak(_) => {},
                TokenType::Open(token) => {
                    if dpy != token.dpy {
                        dpy = token.dpy;
                        state_changes.push(i);
                    }
                },
                TokenType::Punct(token) => {
                    if dpy != token.dpy {
                        dpy = token.dpy;
                        state_changes.push(i);
                    }
                },
                TokenType::Space(token) => {
                    if dpy != token.dpy {
                        dpy = token.dpy;
                        state_changes.push(i);
                    }
                },
                TokenType::Symbol(token) => {
                    if dpy != token.dpy {
                        dpy = token.dpy;
                        state_changes.push(i);
                    }
                },
                TokenType::Word(token) => {
                    if dpy != token.dpy {
                        dpy = token.dpy;
                        state_changes.push(i);
                    }
                },
            }
        }

        state_changes.push(tokens.len());

        let mut iter = state_changes.windows(2);

        while let Some(state_change) = iter.next() {
            let i = state_change[0];
            let j = state_change[1];

            if j - i > 0 {
                segments.push((&tokens[i..j]).into());
            }
        }

        Line {
            column: 0,
            segments: segments,
        }
    }
}

// This conversion generates Postscript code.
impl From<&[TokenType]> for Segment {
    fn from(tokens: &[TokenType]) -> Segment {
        let mut text = String::new();
        let mut ps = String::new();

        let dpy: DisplayFlags = match tokens.first() {
            Some(token) => token.display_flags(),
            None => Default::default(),
        };

        // Postscript prefix
        ps.push('(');

        // text and Postscript-escaped text
        for token in tokens.iter() {
            match token {
                TokenType::Close(token) => {
                    text.push_str(&token.data.text);

                    // Escape parentheses with backslash.
                    if &token.data.text == ")" {
			ps.push_str("\\)");
                    } else {
			ps.push_str(&token.data.text);
                    }
                },
                TokenType::Open(token) => {
                    text.push_str(&token.data.text);

                    // Escape parentheses with backslash.
                    if &token.data.text == "(" {
			ps.push_str("\\(");
                    } else {
			ps.push_str(&token.data.text);
                    }
                },
                TokenType::Punct(token) => {
                    text.push_str(&token.data.text);
                    ps.push_str(&token.data.text);
                },
                TokenType::Space(token) => {
                    text.push_str(&token.data.text);
                    ps.push_str(&token.data.text);
                },
                TokenType::Symbol(token) => {
                    text.push_str(&token.data.text);

                    // Escape backslash with backslash.
                    if &token.data.text == "\\" {
			ps.push_str("\\\\");
                    } else {
			ps.push_str(&token.data.text);
                    }
                },
                TokenType::Word(token) => {
                    text.push_str(&token.data.text);
                    ps.push_str(&token.data.text);
                },
                _ => {},
            }
        }

        // Postscript suffix
        ps.push_str(") ");

        if dpy.intersects(DisplayFlags::EM) {
            ps.push_str("ushow ");
        } else {
            ps.push_str("show ");
        }

        Segment {
            text: text,
            ps: ps,
        }
    }
}

/// Looks ahead to the next valid break point and checks if it will
/// fit into the current line
///
/// # Examples
///
/// ```
/// # use batyr::text::tokens::*;
/// # use batyr::text::next_word_fits;
/// let tokens = vec![TokenType::Space(Token::from(1)),
///                   TokenType::Word(Token::from("foo")),
///                   TokenType::Space(Token::from(1)),
///                   TokenType::Word(Token::from("bar"))];
/// assert_eq!(next_word_fits(&tokens[..], 4, 0, 0), true);
/// assert_eq!(next_word_fits(&tokens[..], 3, 0, 0), false);
/// ```
pub fn next_word_fits(tokens: &[TokenType], line_length: usize, i: usize, x: usize)
    -> bool
{
    let mut j = i + 1;
    let mut u = x + tokens[i].length();

    while j < tokens.len() {
        let frm = tokens[j].format_flags();
        let len = tokens[j].length();

        if frm.intersects(FormatFlags::MLB) {
            return u <= line_length;
            
        } else if frm.intersects(FormatFlags::DLB) {
            if frm.intersects(FormatFlags::DOB) {
                return u <= line_length;
            } else {
                return u + len <= line_length;
            }
        } else {
            u += len;
        }

        j += 1;
    }

    u <= line_length
}

/// Counts the number of lines it will take to render a slice of
/// tokens in a column of a given line length
///
/// # Examples
///
/// ```
/// # use batyr::text::tokens::*;
/// # use batyr::text::count_lines;
/// let tokens = vec![TokenType::Word(Token::from("foo")),
///                   TokenType::Space(Token::from(1)),
///                   TokenType::Word(Token::from("bar"))];
/// assert_eq!(count_lines(&tokens[..], 4), 2);
/// ```
pub fn count_lines(tokens: &[TokenType], line_length: usize) -> usize {
    let mut n: usize = 1; // line count
    let mut x: usize = 0; // char count

    for (i, token) in tokens.iter().enumerate() {
        let frm = token.format_flags();

        if frm.intersects(FormatFlags::MLB) {
            n += 1;
            x = 0;

        } else if frm.intersects(FormatFlags::DLB) {
            if !next_word_fits(tokens, line_length, i, x) {
                n += 1;
                x = 0;
                
            } else {
                x += token.length();
            }

        } else {
            x += token.length();
        }
    }

    n
}

/// Breaks a token list into lines to fill a text block
///
/// This function break the token list into lines, but does not set
/// <tt>line.column</tt> because it relies on the line's <tt>From</tt>
/// implementation.
///
/// # Examples
///
/// ```
/// # use batyr::text::tokens::*;
/// # use batyr::text::linebreak_fill;
/// let tokens = vec![TokenType::Word(Token::from("foo")),
///                   TokenType::Space(Token::from(1)),
///                   TokenType::Word(Token::from("bar"))];
/// let lines = linebreak_fill(&tokens[..], 6);
/// assert_eq!(lines.len(), 2);
/// ```
pub fn linebreak_fill(tokens: &[TokenType], line_length: usize) -> Vec<Line> {
    // tuple (index, discard)
    let mut splits: Vec<(usize, bool)> = Vec::new();
    let mut x: usize = 0;
    
    splits.push((0, false));

    for (i, token) in tokens.iter().enumerate() {
        let frm = token.format_flags();

        if frm.intersects(FormatFlags::MLB) {
            splits.push((i + 1, true));
            x = 0;

        } else if frm.intersects(FormatFlags::DLB) {
            if !next_word_fits(tokens, line_length, i, x) {
                splits.push((i + 1, frm.intersects(FormatFlags::DOB)));
                x = 0;

            } else {
                x += token.length();
            }

        } else {
            x += token.length();
        }
    }

    splits.push((tokens.len(), false));

    let mut lines: Vec<Line> = Vec::new();
    let mut iter = splits.windows(2);

    while let Some(split) = iter.next() {
        let i = split[0].0;
        let j = match split[1].1 {
            true => split[1].0 - 1,  // discard the current token
            false => split[1].0,     // retain the current token
        };
        
        if j - i > 0 {
            lines.push((&tokens[i..j]).into());
        }
    }

    lines
}

/// Breaks a token list into lines to be centered on the page
///
/// This function break the token list into lines, but does not set
/// <tt>line.column</tt> because it relies on the line's <tt>From</tt>
/// implementation.
///
/// # Examples
///
/// ```
/// # use batyr::text::tokens::*;
/// # use batyr::text::linebreak_balance;
/// let tokens = vec![TokenType::Word(Token::from("foo")),
///                   TokenType::Space(Token::from(1)),
///                   TokenType::Word(Token::from("bar"))];
/// let lines = linebreak_balance(&tokens[..], 6);
/// assert_eq!(lines.len(), 2);
/// ```
pub fn linebreak_balance(tokens: &[TokenType], line_length: usize)
                         -> Vec<Line>
{
    let text_length: usize = tokens.iter()
        .fold(0, |sum, token| sum + token.length());
    let height = text_length / line_length + 1;
    let cutoff = text_length / height;

    // tuple (index, discard)
    let mut splits: Vec<(usize, bool)> = Vec::new();
    let mut x: usize = 0;
    
    splits.push((0, false));

    for (i, token) in tokens.iter().enumerate() {
        let frm = token.format_flags();

        if frm.intersects(FormatFlags::MLB) {
            splits.push((i + 1, true));
            x = 0;

        } else if frm.intersects(FormatFlags::DLB) {
            if x + token.length() >= cutoff {
                splits.push((i + 1, frm.intersects(FormatFlags::DOB)));
                x = 0;

            } else {
                x += token.length();
            }

        } else {
            x += token.length();
        }
    }

    splits.push((tokens.len(), false));

    let mut lines: Vec<Line> = Vec::new();
    let mut iter = splits.windows(2);

    while let Some(split) = iter.next() {
        let i = split[0].0;
        let j = match split[1].1 {
            true => split[1].0 - 1,  // discard the current token
            false => split[1].0,     // retain the current token
        };

        if j - i > 0 {
            lines.push((&tokens[i..j]).into());
        }
    }

    lines
}
