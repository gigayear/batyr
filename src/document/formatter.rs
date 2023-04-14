// Batyr Document Formatter
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

//! Breaks token lists into lines and flows the lines into pages

use std::cmp::max;
use std::cmp::min;
use std::collections::VecDeque;
use std::iter::repeat;

use crate::document::*;
use crate::text::*;

/// Information that goes on the fly page
#[derive(Debug)]
pub struct FlyInfo {
    /// Name of television series, if any
    pub series: Option<TokenList>,
    /// Title of movie or episode
    pub title: TokenList,
    /// List of authors
    pub authors: TokenList,
    /// Optional note prints centered beneath the authors
    pub note: Option<TokenList>,
    /// Contact information goes in the bottom left corner
    pub contact: Option<TokenList>,
}

/// A typed page to be output
#[derive(Debug)]
pub struct Page {
    /// Page number.  Printed in the top left corner if it is a
    /// positive number
    pub number: i32,
    /// Maximum number of lines allowed
    pub height: usize,
    /// Actual lines of text to output
    pub lines: Vec<Option<Line>>,
    /// Footer lines go at the bottom of the page
    pub footer: Vec<Option<Line>>,
}

/// Data type for a sequence of pages
pub type PageList = Vec<Page>;

/// Format driver
pub struct Formatter {
    /// Document title
    pub title: String,
    /// Document body
    pub body: PageList,
    next_page_no: i32,
    last_padding_after: usize,
    break_selection: VecDeque<Option<BreakType>>,
    cur_cue: Option<Line>,
    numbering: Numbering,
    cur_scene: Option<String>,
    scene_page_no: i32,
}

impl Formatter {
    /// Construct a new formatter
    ///
    /// # Examples
    ///
    /// ```
    /// use batyr::document::formatter::Formatter;
    /// let formatter = Formatter::new();
    /// assert!(formatter.body.is_empty());
    /// ```
    pub fn new() -> Self {
        Formatter {
            title: "Working Title".to_string(),
            body: Vec::new(),
            next_page_no: 1,
            last_padding_after: 0,
            break_selection: VecDeque::new(),
            cur_cue: None,
            numbering: Numbering::None,
            cur_scene: None,
            scene_page_no: -1,
        }
    }

    fn start_a_new_page(&mut self) {
        let page = Page {
	    number: self.next_page_no,
	    height: TOP_LINE - BOTTOM_LINE + 1,
	    lines: Vec::new(),
            footer:Vec::new(),
        };

        self.body.push(page);
	self.next_page_no += 1;
        self.last_padding_after = 0;

        if self.scene_page_no >= 0 {
            self.scene_page_no += 1;
        }
    }

    fn cur_page(&mut self) -> &mut Page {
	assert!(!self.body.is_empty());
	self.body.iter_mut().last().unwrap()
    }

    fn lines_remaining(&self) -> i32 {
        if let Some(page) = self.body.last() {
            page.height as i32 - page.lines.len() as i32
        } else {
            0
        }
    }

    fn push_blank_lines(&mut self, n: usize) {
        let r = self.lines_remaining();
        let m = min(n as i32, r);
        
        if m > 0 {
            for _ in 0..m {
                self.cur_page().lines.push(None);
            }
        }
    }

    fn push_continued_top(&mut self) {
        let mut line = Line::from(Segment::from("CONTINUED:"));
        line.column = P_BEGIN;

        if self.scene_page_no > 1 {
            let s = format!(" ({})", self.scene_page_no);
            line.segments.push(Segment::from(s));
        }

        if let Some(label) = &self.cur_scene {
            self.add_numbering(label, &mut line);
        }
        
        self.cur_page().lines.push(Some(line));
        self.push_blank_lines(1);                        
    }

    fn push_continued_bottom(&mut self) {
        let mut line = Line::from(Segment::from("(CONTINUED)"));
        line.column = TRANS_BEGIN;

        self.push_blank_lines(1);                        
        self.cur_page().lines.push(Some(line));
    }

    fn add_numbering(&self, label: &str, line: &mut Line) {
        let w = P_END - P_BEGIN + 1;
        
        if self.numbering == Numbering::Right
            || self.numbering == Numbering::Full
        {
            let n = w - line.length() + 1;
            let spaces = repeat(' ').take(n).collect::<String>();
            
            let suffix = format!("{}{}", spaces, &label);
            line.segments.push(Segment::from(suffix));
        }
        
        if self.numbering == Numbering::Left
            || self.numbering == Numbering::Full
        {
            let n =  max(6 - label.chars().count() as i32, 0) as usize;
            let spaces = repeat(' ').take(n).collect::<String>();
                                        
            let prefix = format!("{}{}", label, spaces);
            line.column -= prefix.chars().count();
            line.segments.insert(0, Segment::from(prefix));
        }
    }

    /// Process an element
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::document::*;
    /// # use batyr::document::formatter::Formatter;
    /// let root = ElementType::Br(EmptyElement::new(Br {}));
    /// let mut formatter = Formatter::new();
    /// formatter.run(root);
    /// assert_eq!(formatter.body.len(), 1);
    /// ```
    pub fn run(&mut self, root: ElementType) {
        let mut fly_info = FlyInfo {
            series: None,
            title: Vec::new(),
            authors: Vec::new(),
            note: None,
            contact: None,
        };
    
        for elem in root.into_iter() {
            let padding_before;

            match elem.get_padding_before() {
                Some(n) => {
                    if n < 0 {
                        self.start_a_new_page();
                        padding_before = -n - 1;
                    } else {
                        padding_before = n;
                    }
                },
                None => {
                    padding_before = 0;
                },
            };

            let padding_after = self.last_padding_after;
            
            match elem.get_padding_after() {
                Some(n) => {
                    self.last_padding_after = n;
                },
                None => (),
            }
            
            match elem {
                ElementType::Act(elem) => {
                    if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );
                    }
                    
                    let mut line = Line::from(&elem.tokens[..]);
                    let len = line.length();
                    line.column = CENTER - len / 2 - len % 2;
                    self.cur_page().lines.push(Some(line));
                },
                ElementType::Authors(_) => (),
                ElementType::Body(_) => (),
                ElementType::Br(_) => {
                    self.push_blank_lines(1);
                },
                ElementType::Contact(elem) => {
                    fly_info.contact = Some((&elem.tokens[..]).to_vec());
                },
                ElementType::Cue(elem) => {
                    let h = elem.count_lines() as i32;
                    let mut r = self.lines_remaining();

                    if self.scene_page_no >= 0 && !elem.at_scene_end {
                        // If not followed by Slug, the page must be
                        // shortened whether or not the dialogue fits.
                        r -= 2; // make room for (CONTINUED)
                    }
                    
                    if r < h as i32 + padding_before { // dialogue won't fit
                        if self.scene_page_no >= 0 && elem.at_scene_end {
                            // Even if followed by Slug, the page must
                            // be shortened because the dialogue will
                            // need to be broken.
                            r -= 2; // shorten the page for (CONTINUED)
                        }
                    
                        let (i, break_info)
                            = elem.select_break(r - padding_before);

                        if i >= 0 {
                            for _ in 0..i {
                                self.break_selection.push_back(None);
                            }

                            self.break_selection.push_back(Some(break_info));

                            self.push_blank_lines(
                                max(padding_before as usize, padding_after)
                            );
                        } else {
                            if self.scene_page_no >= 0 {
                                self.push_continued_bottom();
                            }
                            
                            self.start_a_new_page();

                            if self.scene_page_no >= 0 {
                                self.push_continued_top();
                            }
                        }
                    } else {
                        if self.cur_page().lines.is_empty()
                            && self.scene_page_no >= 0
                        {
                            self.push_continued_top();

                        } else {
                            self.push_blank_lines(
                                max(padding_before as usize, padding_after)
                            );
                        }
                    }

                    let mut line = Line::from(&elem.tokens[..]);
                    line.column = elem.attributes.tab_stop;
                    self.cur_cue = Some(line.clone());
                    self.cur_page().lines.push(Some(line));
                },
                ElementType::D(elem) => {
                    let w = elem.attributes.right_margin
                        - elem.attributes.left_margin + 1;

                    self.push_blank_lines(
                        max(padding_before as usize, padding_after)
                    );
                                
                    if !self.break_selection.is_empty() {
                        match self.break_selection.pop_front().unwrap() {
                            None => {
                                let lines = linebreak_fill(&elem.tokens[..], w);

                                for mut line in lines {
                                    line.column = elem.attributes.left_margin;
                                    self.cur_page().lines.push(Some(line));
                                }
                            },
                            Some(break_info) => {
                                match break_info {
                                    BreakType::Atomic(_) => {
                                        let lines = linebreak_fill(
                                            &elem.tokens[..], w
                                        );

                                        for mut line in lines {
                                            line.column = elem.attributes.left_margin;
                                            self.cur_page().lines.push(Some(line));
                                        }

                                        let more_line = Line {
                                            column: CUE_BEGIN,
                                            segments: vec![Segment::from("(MORE)")],
                                        };
                                        
                                        self.cur_page().lines.push(Some(more_line));

                                        if self.scene_page_no >= 0 {
                                            self.push_continued_bottom();
                                        }

                                        self.start_a_new_page();

                                        if self.scene_page_no >= 0 {
                                            self.push_continued_top();
                                        }
                                        
                                        if self.cur_cue.is_some() {
                                            let mut line = mem::replace(
                                                &mut self.cur_cue, None
                                            ).unwrap();
                                            
                                            line.segments.push(Segment::from(" (CONT'D)"));
                                            self.cur_page().lines.push(Some(line));
                                        }
                                    },
                                    BreakType::Point(break_point) => {
                                        let mut lines = linebreak_fill(
                                            &elem.tokens[0..break_point.token_index], w
                                        );

                                        for mut line in lines {
                                            line.column = elem.attributes.left_margin;
                                            self.cur_page().lines.push(Some(line));
                                        }

                                        let more_line = Line {
                                            column: CUE_BEGIN,
                                            segments: vec![Segment::from("(MORE)")],
                                        };
                                        
                                        self.cur_page().lines.push(Some(more_line));

                                        if self.scene_page_no >= 0 {
                                            self.push_continued_bottom();
                                        }
                                        
                                        self.start_a_new_page();

                                        if self.scene_page_no >= 0 {
                                            self.push_continued_top();
                                        }
                                        
                                        if self.cur_cue.is_some() {
                                            let mut line = mem::replace(
                                                &mut self.cur_cue, None
                                            ).unwrap();
                                            
                                            line.segments.push(Segment::from(" (CONT'D)"));
                                            self.cur_page().lines.push(Some(line));
                                        }

                                        lines = linebreak_fill(
                                            &elem.tokens[break_point.token_index..], w
                                        );

                                        for mut line in lines {
                                            line.column = elem.attributes.left_margin;
                                            self.cur_page().lines.push(Some(line));
                                        }
                                    },
                                    _ => (),
                                }
                            }
                        }
                            
                    } else { // No break information left by the cue.
                        let lines = linebreak_fill(&elem.tokens[..], w);

                        for mut line in lines {
                            line.column = elem.attributes.left_margin;
                            self.cur_page().lines.push(Some(line));
                        }
                    }
                },
                ElementType::Dir(elem) => {
                    if !self.break_selection.is_empty() {
                        // Page breaks must always come before, not
                        // after, personal direction, so this value
                        // should always be None.
                        let _ = self.break_selection.pop_front();
                    }

                    let w = elem.attributes.right_margin
                        - elem.attributes.left_margin + 1;

                    let lines = linebreak_fill(&elem.tokens[..], w);
                    let h = lines.len();
                    
                    for (i, mut line) in lines.into_iter().enumerate() {
                        line.column = elem.attributes.left_margin;

                        if i == 0 {
                            line.segments.insert(0, Segment::from("("));
                            line.column -= 1;
                        }

                        if i == h - 1 {
                            line.segments.push(Segment::from(")"));
                        }

                        self.cur_page().lines.push(Some(line));
                    }
                },
                ElementType::Em(_) => (),
                ElementType::End(elem) => {
                   if self.lines_remaining() < padding_before + 1 {
                        self.start_a_new_page();

                        if self.scene_page_no >= 0 {
                            self.push_continued_top();
                        }
                        
                    } else if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );

                       // One space will be skipped automatically due
                       // to the padding_before attribute.  Skip up to
                       // four additional lines if space allows.
                       let r = self.lines_remaining() - padding_before;

                       if r > 0 {
                           self.push_blank_lines(min(r as usize, 4));
                       }
                   }

                    let mut line = Line::from(&elem.tokens[..]);
                    let len = line.length();
                    line.column = CENTER - len / 2 - len % 2;
                    self.cur_page().lines.push(Some(line));

                    self.cur_scene = None;
                    self.scene_page_no = -1;
                },
                ElementType::FullName(elem) => {
                    if !fly_info.authors.is_empty() {
                        fly_info.authors.push(TokenType::Space(Token::from(1)));
                        fly_info.authors.push(TokenType::Symbol(Token::from("&")));
                        fly_info.authors.push(TokenType::Space(Token::from(1)));
                    }

                    fly_info.authors.extend_from_slice(&elem.tokens[..]);
                },
                ElementType::Head(_) => (),
                ElementType::Note(elem) => {
                    fly_info.note = Some((&elem.tokens[..]).to_vec());
                },
                ElementType::Open(elem) => {
                   if self.lines_remaining() < padding_before + 1 {
                        self.start_a_new_page();

                        if self.scene_page_no >= 0 {
                            self.push_continued_top();
                        }
                        
                    } else if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );
                    }
                    
                    let mut line = Line::from(&elem.tokens[..]);
                    line.column = elem.attributes.tab_stop;
                    self.cur_page().lines.push(Some(line));
                },
                ElementType::P(elem) => {
                    let h = elem.count_lines();
                    let mut r = self.lines_remaining();

                    if self.scene_page_no >= 0 && !elem.at_scene_end {
                        r -= 2; // make room for (CONTINUED)
                    }
                    
                    let mut break_point: Option<BreakPoint> = None;

                    if r <= 0 {
                        if self.scene_page_no >= 0 {
                            self.push_continued_bottom();
                        }
                            
                        self.start_a_new_page();

                        if self.scene_page_no >= 0 {
                            self.push_continued_top();
                        }

                    } else if r < h as i32 + padding_before {
                        match elem.select_break(r - padding_before) {
                            BreakType::Mandatory => {
                                if self.scene_page_no >= 0 {
                                    self.push_continued_bottom();
                                }
                                
                                self.start_a_new_page();

                                if self.scene_page_no >= 0 {
                                    self.push_continued_top();
                                }
                            },
                            BreakType::Point(selected_break_point) => {
                                break_point = Some(selected_break_point);

                                self.push_blank_lines(
                                    max(padding_before as usize,
                                        padding_after)
                                );
                            },
                            BreakType::None => {
                                self.push_blank_lines(
                                    max(padding_before as usize,
                                        padding_after)
                                );
                            },
                            _ => (),
                        }
                    } else if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );
                    }
                    
                    let w = elem.attributes.right_margin
                        - elem.attributes.left_margin + 1;

                    match break_point {
                        Some(break_point) => {
                            let mut lines = linebreak_fill(
                                &elem.tokens[0..break_point.token_index], w
                            );

                            for mut line in lines {
                                line.column = elem.attributes.left_margin;
                                self.cur_page().lines.push(Some(line));
                            }

                            if self.scene_page_no >= 0 {
                                self.push_continued_bottom();
                            }

                            self.start_a_new_page();

                            if self.scene_page_no >= 0 {
                                self.push_continued_top();
                            }
                                
                            lines = linebreak_fill(
                                &elem.tokens[break_point.token_index..], w
                            );

                            for mut line in lines {
                                line.column = elem.attributes.left_margin;
                                self.cur_page().lines.push(Some(line));
                            }
                        },
                        None => {
                            let lines = linebreak_fill(&elem.tokens[..], w);
                        
                            for mut line in lines {
                                line.column = elem.attributes.left_margin;
                                self.cur_page().lines.push(Some(line));
                            }
                        },
                    }
                },
                ElementType::PageBreak(_) => {
                    if self.scene_page_no >= 0 {
                        self.push_continued_bottom();
                    }

                    self.start_a_new_page();

                    if self.scene_page_no >= 0 {
                        self.push_continued_top();
                    }
                },
                ElementType::Screenplay(elem) => {
                    self.numbering = elem.attributes.numbering;
                    self.start_a_new_page();
                },
                ElementType::Series(elem) => {
                    let w = elem.attributes.right_margin
                        - elem.attributes.left_margin + 1;

                    let lines = linebreak_balance(&elem.tokens[..], w);
                    
                    for mut line in lines {
                        let len = line.length();
                        line.column = CENTER - len / 2 - len % 2;
                        self.cur_page().lines.push(Some(line));
                    }

                    fly_info.series = Some((&elem.tokens[..]).to_vec());
                },
                ElementType::Slug(elem) => {
                    let label;
                    
                    if self.numbering != Numbering::None {
                        if let Some(c) =  elem.attributes.addition {
                            label = format!("{}{}", elem.attributes.number, c);
                        } else {
                            label = format!("{}", elem.attributes.number);
                        }

                        self.cur_scene = Some(label.clone());
                        self.scene_page_no = 0;
                    
                    } else {
                        label = String::new();
                    }
                    
                    let w: usize = elem.attributes.right_margin
                        - elem.attributes.left_margin + 1;
                    
                    let lines = linebreak_fill(&elem.tokens[..], w);
                    let h = elem.lines_to_first_break();
                    let mut r = self.lines_remaining();

                    r -= 2; // Make room for (CONTINUED).

                    if r < h as i32 + padding_before {
                        self.start_a_new_page();

                    } else if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );
                    }
                    
                    for (i, mut line) in lines.into_iter().enumerate() {
                        line.column = elem.attributes.left_margin;

                        if i == 0 {
                            self.add_numbering(&label, &mut line);
                        }
                        
                        self.cur_page().lines.push(Some(line));
                    }
                },
                ElementType::Title(elem) => {
                    let w = elem.attributes.right_margin
                        - elem.attributes.left_margin + 1;
                    let lines = linebreak_balance(&elem.tokens[..], w);
                    let h = lines.len();
                    let r = self.lines_remaining();

                    if r < h as i32 + padding_before {
                        self.start_a_new_page();
                    } else if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );
                    }

                    for mut line in lines {
                        let len = line.length();
                        line.column = CENTER - len / 2 - len % 2;
                        self.cur_page().lines.push(Some(line));
                    }
                    
                    fly_info.title.extend_from_slice(&elem.tokens[..]);
                },
                ElementType::Trans(elem) => {
                    let h: usize = 1;
                    let r = self.lines_remaining();

                    if r < h as i32 + padding_before {
                        if self.scene_page_no >= 0 {
                            self.push_continued_bottom();
                        }

                        self.start_a_new_page();
                        
                    } else if !self.cur_page().lines.is_empty() {
                        self.push_blank_lines(
                            max(padding_before as usize, padding_after)
                        );
                    }
                    
                    let mut line = Line::from(&elem.tokens[..]);
                    let w = elem.attributes.right_margin
                        - elem.attributes.tab_stop - 1;
                    let n = line.length();
                    if n > w {
                        line.column = elem.attributes.right_margin - n;
                    } else {
                        line.column = elem.attributes.tab_stop;
                    }
                    
                    self.cur_page().lines.push(Some(line));

                    self.cur_scene = None;
                    self.scene_page_no = -1;
                },
            }            
        }

        let (title, fly_page) = self.format_fly_page(fly_info);

        self.title = title;
        self.body.insert(0, fly_page);
    }

    fn format_fly_page(&self, fly_info: FlyInfo) -> (String, Page) {
        let mut title = String::new();

        let mut page = Page {
            number: -1,
            height: TOP_LINE - BOTTOM_LINE + 1,
            lines: Vec::new(),
            footer: Vec::new(),
        };

        let left_margin = LEFT_MARGIN + 2 * INDENT;
        let right_margin = RIGHT_MARGIN - 2 * INDENT;
        let w = right_margin - left_margin + 1;
        
        for _ in 0 .. TITLE_SKIP {
            page.lines.push(None);
        }
        
        if let Some(series_tokens) = fly_info.series {
            let lines = linebreak_balance(&series_tokens[..], w);

            for mut line in lines {
                let len = line.length();
                line.column = CENTER - len / 2 - len % 2;
                page.lines.push(Some(line));
                page.lines.push(None);
            }
        }

        let title_lines = linebreak_balance(&fly_info.title[..], w);

        for (i, mut line) in title_lines.into_iter().enumerate() {
            if i == 0 {
                title = line.text();
            }

            let len = line.length();
            line.column = CENTER - len / 2 - len % 2;
            page.lines.push(Some(line));
            page.lines.push(None);
        }
        
        page.lines.push(None);
        page.lines.push(None);

        let mut author_lines = linebreak_balance(&fly_info.authors[..], w);
        author_lines.insert(0, Line::from(Segment::from("written by")));
        
        for mut line in author_lines {
            let len = line.length();
            line.column = CENTER - len / 2 - len % 2;
            page.lines.push(Some(line));
            page.lines.push(None);
        }

        if let Some(note_tokens) = fly_info.note {
            page.lines.push(None);
            page.lines.push(None);
            
            let lines = linebreak_balance(&note_tokens[..], w);

            for mut line in lines {
                let len = line.length();
                line.column = CENTER - len / 2 - len % 2;
                page.lines.push(Some(line));
                page.lines.push(None);
            }
        }

        if let Some(contact_tokens) = fly_info.contact {
            let w = CONTACT_END - CONTACT_BEGIN + 1;
            let lines = linebreak_fill(&contact_tokens[..], w);

            for mut line in lines {
                line.column = CONTACT_BEGIN;
                page.footer.push(Some(line));
            }
        }
        
        (title, page)
    }
}
