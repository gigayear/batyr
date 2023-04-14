// Batyr Document Module
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

//! In-memory representation of a document
//!
//! * The [`reader`] module builds an element tree, tokenizing the
//!   contents of each text element.
//!
//! * The [`formatter`] module breaks token lists into lines, and
//!   flows the lines into pages.
//!
//! * The [`writer`] module writes the pages to the standard output
//!   using the Latin-9 character set.

use std::mem;

use crate::text::tokens::*;

pub mod reader;
pub mod formatter;
pub mod writer;

// configuration

/// Character width in points
pub const CHAR_WIDTH: f32 = 7.2;

/// Line height in points
pub const LINE_HEIGHT: f32 = 12.0;

/// Default indent in spaces
pub const INDENT: usize = 5;

/// Center point in spaces
pub const CENTER: usize = 43;

/// Left margin in spaces
pub const LEFT_MARGIN: usize = 10;

/// Right margin in spaces
pub const RIGHT_MARGIN: usize = 75;

/// Transition left margin
pub const TRANS_BEGIN: usize = 60;

/// Transition right margin
pub const TRANS_END: usize = 75;

/// Contact left margin
pub const CONTACT_BEGIN: usize = 12;

/// Contact right margin
pub const CONTACT_END: usize = 42;

/// Cue tab stop
pub const CUE_BEGIN: usize = 42;

/// Dialogue left margin
pub const D_BEGIN: usize = 26;

/// Dialogue right margin
pub const D_END: usize = 59;

/// Personal direction left margin
pub const DIR_BEGIN: usize = 34;

/// Personal direction right margin
pub const DIR_END: usize = 52;

/// Stage direction left margin
pub const P_BEGIN: usize = 16;

/// Stage direction right margin
pub const P_END: usize = 72;

/// Page number tab stop
pub const PAGE_NO_BEGIN: usize = 72;

/// Left scene number tab stop
pub const LH_SCENE_NO_BEGIN: usize = 12;

/// Right scene number tab stop
pub const RH_SCENE_NO_BEGIN: usize = 73;

/// Line number of the page header
pub const HEADER_LINE: usize = 62;

/// Line number of the top line of the page
pub const TOP_LINE: usize = 60;

/// Line number of the middle of the page
pub const MIDDLE_LINE: usize = 27;

/// Line number of the bottom line of the page
pub const BOTTOM_LINE: usize = 6;

/// The number of lines to skip before the title on the fly page
pub const TITLE_SKIP: usize = 19;

// element type enum

/// Element type enum for in-memory representation of XML elements
#[derive(Debug)]
pub enum ElementType {
    Act       (TextElement     <Act       >),
    Authors   (ContainerElement<Authors   >),
    Body      (ContainerElement<Body      >),
    Br        (EmptyElement    <Br        >),
    Trans     (TextElement     <Trans     >),
    Contact   (TextElement     <Contact   >),
    Cue       (TextElement     <Cue       >),
    D         (TextElement     <D         >),
    Dir       (TextElement     <Dir       >),
    Em        (TextElement     <Em        >),
    End       (TextElement     <End       >),
    FullName  (TextElement     <FullName  >),
    Head      (ContainerElement<Head      >),
    Note      (TextElement     <Note      >),
    Open      (TextElement     <Open      >),
    P         (TextElement     <P         >),
    PageBreak (EmptyElement    <PageBreak >),
    Screenplay(ContainerElement<Screenplay>),
    Series    (TextElement     <Series    >),
    Slug      (TextElement     <Slug      >),
    Title     (TextElement     <Title     >),
}

/// Data type for a sequence of elements
pub type ElementList = Vec<ElementType>;

impl ElementType {
    /// Returns true for container elements
    pub fn has_children(&self) -> bool {
        match self {
            ElementType::Authors   (_) |
            ElementType::Body      (_) |
            ElementType::Head      (_) |
            ElementType::Screenplay(_) => true,
            _ => false,
        }
    }

    /// If the element is a container, returns a mutable reference to
    /// the children
    pub fn children_mut(&mut self) -> Option<&mut ElementList> {
        match self {
            ElementType::Authors(elem) => Some(&mut elem.children),
            ElementType::Body(elem) => Some(&mut elem.children),
            ElementType::Head(elem) => Some(&mut elem.children),
            ElementType::Screenplay(elem) =>  Some(&mut elem.children),
            _ => None,
        }
    }

    /// Extract the children through a mutable reference 
    pub fn drain_children(&mut self) -> Option<ElementList> {
        match self {
            ElementType::Authors(elem) => {
                Some(elem.children.drain(..).collect())
            },
            ElementType::Body(elem) => {
                Some(elem.children.drain(..).collect())
            },
            ElementType::Head(elem) => {
                Some(elem.children.drain(..).collect())
            },
            ElementType::Screenplay(elem) =>  {
                Some(elem.children.drain(..).collect())
            },
            _ => None,
        }
    }

    /// Return an into iterator for the element
    pub fn into_iter(self) -> ElementIntoIterator {
        ElementIntoIterator {
            children: vec![self],
            parent: None,
        }
    }

    /// If the element has a padding_before attribute, return its value
    pub fn get_padding_before(&self) -> Option<i32> {
        match self {
            ElementType::Act       (elem) => Some(elem.attributes.padding_before),
            ElementType::Authors   (_) => None,
            ElementType::Body      (_) => None,
            ElementType::Br        (_) => None,
            ElementType::Trans     (elem) => Some(elem.attributes.padding_before),
            ElementType::Contact   (_) => None,
            ElementType::Cue       (elem) => Some(elem.attributes.padding_before),
            ElementType::D         (elem) => Some(elem.attributes.padding_before),
            ElementType::Dir       (elem) => Some(elem.attributes.padding_before),
            ElementType::Em        (_) => None,
            ElementType::End       (elem) => Some(elem.attributes.padding_before),
            ElementType::FullName  (_) => None,
            ElementType::Head      (_) => None,
            ElementType::Note      (_) => None,
            ElementType::Open      (elem) => Some(elem.attributes.padding_before),
            ElementType::P         (elem) => Some(elem.attributes.padding_before),
            ElementType::PageBreak (_) => None,
            ElementType::Screenplay(_) => None,
            ElementType::Series    (elem) => Some(elem.attributes.padding_before),
            ElementType::Slug      (elem) => Some(elem.attributes.padding_before),
            ElementType::Title     (elem) => Some(elem.attributes.padding_before),
        }
    }

    /// If the element has a padding_after attribute, return its value
    pub fn get_padding_after(&self) -> Option<usize> {
        match self {
            ElementType::Act       (elem) => Some(elem.attributes.padding_after),
            ElementType::Authors   (_) => None,
            ElementType::Body      (_) => None,
            ElementType::Br        (_) => None,
            ElementType::Trans     (elem) => Some(elem.attributes.padding_after),
            ElementType::Contact   (_) => None,
            ElementType::Cue       (elem) => Some(elem.attributes.padding_after),
            ElementType::D         (elem) => Some(elem.attributes.padding_after),
            ElementType::Dir       (elem) => Some(elem.attributes.padding_after),
            ElementType::Em        (_) => None,
            ElementType::End       (elem) => Some(elem.attributes.padding_after),
            ElementType::FullName  (_) => None,
            ElementType::Head      (_) => None,
            ElementType::Note      (_) => None,
            ElementType::Open      (elem) => Some(elem.attributes.padding_after),
            ElementType::P         (elem) => Some(elem.attributes.padding_after),
            ElementType::PageBreak (_) => None,
            ElementType::Screenplay(_) => None,
            ElementType::Series    (elem) => Some(elem.attributes.padding_after),
            ElementType::Slug      (elem) => Some(elem.attributes.padding_after),
            ElementType::Title     (elem) => Some(elem.attributes.padding_after),
        }
    }
}

/// Into iterator for an element (flattens the children)
#[derive(Debug)]
pub struct ElementIntoIterator {
    children: ElementList,
    parent: Option<Box<ElementIntoIterator>>,
}

impl Iterator for ElementIntoIterator {
    type Item = ElementType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.children.is_empty() {
             match self.parent.take() {
                 Some(parent) => {
                     // continue with the parent node
                     *self = *parent;
                     self.next()
                 },
                 None => None,
             }
        } else {
            let mut elem = self.children.remove(0);

            if elem.has_children() {
                // start iterating over the child trees
                *self = ElementIntoIterator {
                    children: elem.drain_children().unwrap(),
                    parent: Some(Box::new(mem::take(self))),
                };
            }

            Some(elem)
        }
    }
}

impl Default for ElementIntoIterator {
    fn default() -> Self {
        ElementIntoIterator { children: Vec::new(), parent: None }
    }
}

/// Break options
#[derive(Debug, Clone, PartialEq)]
pub enum BreakType {
    None,
    Mandatory,
    Forbidden(usize),
    Atomic(usize),
    Disposable(usize),
    Point(BreakPoint),
    List(BreakPointList),
}

/// Candidate break point
#[derive(Debug, Clone, PartialEq)]
pub struct BreakPoint {
    /// Token list index
    pub token_index: usize,
    /// If the break point is chosen, discard the corresponding token
    /// if this flag is true.
    pub discard_flag: bool,
    /// Line number in which the break point appears
    pub line_no: usize,
}

/// Data type for a sequence of break points
pub type BreakPointList = Vec<BreakPoint>;

/// Scene number setting
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Numbering {
    /// No scene numbers
    None = 0,
    /// Scene number on the left side of the page only
    Left = 1,
    /// Scene number on the right side of the page only
    Right = 2,
    /// Scene number on the both sides of the page
    Full = 3,
}

impl From<&str> for Numbering {
    fn from(s: &str) -> Self {
        match s {
            "full" => Numbering::Full,
            "right" => Numbering::Right,
            "left" => Numbering::Left,
            _ => Numbering::None,
        }
    }
}

// generic elements

/// Generic container element contains only other elements, no text
#[derive(Debug)]
pub struct ContainerElement<Attributes> {
    /// Parameter struct
    pub attributes: Attributes,
    /// Sequence of child elements
    pub children: ElementList,
}

impl<Attributes> ContainerElement<Attributes> {
    pub fn new(attributes: Attributes) -> Self {
        Self {
            attributes: attributes,
            children: Vec::new(),
        }
    }
}

/// Generic empty element contains only attributes, no content
#[derive(Debug)]
pub struct EmptyElement<Attributes> {
    /// Parameter struct
    pub attributes: Attributes,
    /// Break type for this element
    pub break_info: BreakType,
}

impl<Attributes> EmptyElement<Attributes> {
    pub fn new(attributes: Attributes) -> Self {
        Self {
            attributes: attributes,
            break_info: BreakType::None,
        }
    }
}

/// Generic text element contains mixed content, and footnote elements
/// are set aside
#[derive(Debug)]
pub struct TextElement<Attributes> {
    pub attributes: Attributes,
    pub tokens: TokenList,
    pub break_info: BreakType,
    pub at_scene_end: bool,
}

impl<Attributes> TextElement<Attributes> {
    pub fn new(attributes: Attributes) -> Self {
        Self {
            attributes: attributes,
            tokens: Vec::new(),
            break_info: BreakType::None,
            at_scene_end: false,
        }
    }
}

// screenplay elements

/// Act beginning
///
/// # Examples
///
/// Act names should be all caps and underlined, but that will not
/// happen automatically.  Use the following:
///
/// ```xml
/// <act><em>ACT ONE</em></act>
/// ```
///
/// Output:
///
/// <pre width="100%" style="text-align: center;"><ins>ACT ONE</ins></pre>
///
/// Act numbering is not connected to the name of the act; it is used
/// to control whether or not to page break before the new act.  That
/// way you can name the acts whatever you want, such as:
///
/// ```xml
/// <act><em>TEASER</em></act>
/// ```
///
/// Output:
///
/// <pre width="100%" style="text-align: center;"><ins>TEASER</ins></pre>
#[derive(Debug)]
pub struct Act {
    /// Act number.  This is an internal sequence number that does not
    /// appear in any output.  In particular, it but does not
    /// necessarily match any numbering in the act titles.
    pub number: i32,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Container for a sequence of authors
#[derive(Debug)]
pub struct Authors {}

/// Document body
#[derive(Debug)]
pub struct Body {}

/// Mandatory line break
#[derive(Debug)]
pub struct Br {}

/// Contact information
///
/// Contact information flows into a block half the width of the page,
/// but it is intended to be used with line breaks.
///
/// # Examples
///
/// ```xml
/// <contact>MATCHLOCK PRESS<br/>P.O.\ Box 90606<br/>Brooklyn, NY 11209</contact>
/// ```
///
/// Output:
///
/// <pre>
/// MATCHLOCK PRESS
/// P.O. Box 90606
/// Brooklyn, NY 11209
/// </pre>
#[derive(Debug)]
pub struct Contact {
    /// Left edge of the page
    pub left_margin: usize,
    /// Page center
    pub right_margin: usize,
}

/// Character cue
///
/// Character cues should be in all caps.  There is no built-in
/// handling for offscreen dialogue and voice overs.  Use "(O.S.)" and
/// "(V.O.)" respectively.  As per modern practice, Batyr will
/// automatically add "(CONT'D)" to the character cue at the top of
/// the next page if the dialogue is broken across the page break.  It
/// is _not_ added automatically to dialogue spoken by the same
/// character that is broken by stage direction.
///
/// # Examples
///
/// ```xml
/// <cue>GEORGE</cue>
/// <dir>in tears</dir>
/// <d>
///   You're hurting my sore ear.
/// </d>
/// ```
///
/// Output:
///
/// <pre>
///                                    GEORGE
///                           (in tears)
///                    You're hurting my sore ear.
/// </pre>
#[derive(Debug)]
pub struct Cue {
    /// Column number to begin typing at
    pub tab_stop: usize,
    /// Break point information for D or Dir elements immediately
    /// following the Cue
    pub train: Vec<BreakType>,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

impl TextElement<Cue> {
    /// Counts lines for the cue and its train based on break point
    /// information only
    pub fn count_lines(&self) -> usize {
        let mut line_count: usize = 1;

        for break_info in self.attributes.train.iter() {
            match break_info {
                BreakType::None => (),
                BreakType::Mandatory => (),
                BreakType::Forbidden(h) => line_count += h,
                BreakType::Atomic(h) => line_count += h,
                BreakType::Disposable(h) => line_count += h,
                BreakType::Point(break_point) => {
                    line_count += break_point.line_no;
                },
                BreakType::List(break_points) => {
                    let n = break_points.len();
                    line_count += break_points[n - 1].line_no;
                },
            }
        }

        line_count
    }

    /// Selects a break point given the number of lines remaining in
    /// the page
    pub fn select_break(&self, lines_remaining: i32) -> (i32, BreakType) {
        if lines_remaining < 2 { // Don't orphan the character cue.
            return (-1, BreakType::None);
        }

        let total_height = self.count_lines();
        let n = self.attributes.train.len();
        
        if total_height as i32 <= lines_remaining { // entire dialogue fits
            return (n as i32, BreakType::None);
        }

        // We need to add "(MORE)" at the bottom of the page.
        let mut line_count = 1; // 1 for the character cue
        let mut prev_break_list_index: i32 = -1;
        let mut prev_break_info: BreakType = BreakType::None;
        
        for i in 0..n {
            match &self.attributes.train[i] {
                BreakType::None => (),
                BreakType::Mandatory => {
                    return (i as i32, BreakType::Mandatory);
                },
                BreakType::Forbidden(h) => {
                    if i == n - 1 { // last break
                        if (line_count + h) as i32 > lines_remaining {
                            return (prev_break_list_index, prev_break_info);
                        }
                    } else {
                        // plus 1 for (MORE)
                        if (line_count + h + 1) as i32 > lines_remaining {
                            return (prev_break_list_index, prev_break_info);
                        }
                    }

                    line_count += h;
                },
                BreakType::Atomic(h) => {
                    if i == n - 1 { // last break
                        if (line_count + h) as i32 > lines_remaining {
                            return (prev_break_list_index, prev_break_info);
                        }
                    } else {
                        // plus 1 for (MORE)
                        if (line_count + h + 1) as i32 > lines_remaining {
                            return (prev_break_list_index, prev_break_info);
                        }
                    }

                    prev_break_info = BreakType::Atomic(*h);
                    prev_break_list_index = i as i32;

                    line_count += h;
                },
                BreakType::Disposable(h) => {
                    if i == n - 1 { // last break
                        if (line_count + h) as i32 > lines_remaining {
                            return (prev_break_list_index, prev_break_info);
                        }
                    } else {
                        // plus 1 for (MORE)
                        if (line_count + h + 1) as i32 > lines_remaining {
                            return (prev_break_list_index, prev_break_info);
                        }
                    }

                    prev_break_info = BreakType::Disposable(*h);
                    prev_break_list_index = i as i32;

                    line_count += h;
                },
                BreakType::Point(break_point) => {
                    if i == n - 1 { // last break
                        if (line_count + break_point.line_no) as i32
                            > lines_remaining
                        {
                            return (prev_break_list_index, prev_break_info);
                        }
                    } else {
                        // plus 1 for (MORE)
                        if (line_count + break_point.line_no + 1) as i32
                            > lines_remaining
                        {
                            return (prev_break_list_index, prev_break_info);
                        }
                    }

                    prev_break_info = BreakType::Point(break_point.clone());
                    prev_break_list_index = i as i32;
                },
                BreakType::List(break_points) => {
                    let m = break_points.len();

                    for j in 0..m {
                        let break_point = &break_points[j];

                        // plus 1 for (MORE)
                        if (line_count + break_point.line_no + 1) as i32
                            > lines_remaining
                        {
                            if prev_break_info == BreakType::None
                                || prev_break_list_index < 0
                            {
                                return (-1, BreakType::None);
                                
                            } else {
                                return (prev_break_list_index, prev_break_info);
                            }
                        }

                        prev_break_info = BreakType::Point(break_point.clone());
                        prev_break_list_index = i as i32;
                    }

                    line_count += break_points[m - 1].line_no;
                },
            }
        }
        
        (-1, BreakType::None)
    }
}

/// Dialogue
///
/// For an example, see [`Cue`].
#[derive(Debug)]
pub struct D {
    /// Number of spaces to indent (default to 0)
    pub indent: usize,
    /// Narrow column left margin
    pub left_margin: usize,
    /// Narrow column right margin
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Personal direction
///
/// For an example, see [`Cue`].
#[derive(Debug)]
pub struct Dir {
    /// Narrow column left margin
    pub left_margin: usize,
    /// Narrow column right margin
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Emphasis
#[derive(Debug)]
pub struct Em {}

/// End of sequence
///
/// # Examples
///
/// ```xml
/// <end><em>END OF ACT ONE</em></end>
/// ```
///
/// Output:
///
/// <pre style="text-align: center;"><ins>END OF ACT ONE</ins></pre>
#[derive(Debug)]
pub struct End {
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Author's name
#[derive(Debug)]
pub struct FullName {}

/// Document head
#[derive(Debug)]
pub struct Head {}

/// Scene opening
///
/// # Examples
///
/// ```xml
/// <open>FADE IN:</open>
/// ```
///
/// Output:
///
/// <pre>FADE IN:</pre>
#[derive(Debug)]
pub struct Open {
    /// Column number to begin typing at
    pub tab_stop: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Title page note
#[derive(Debug)]
pub struct Note {}

/// Stage direction
#[derive(Debug)]
pub struct P {
    /// Number of spaces to indent (default to 0)
    pub indent: usize,
    /// Full-width column left margin
    pub left_margin: usize,
    /// Full-width column right margin
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

impl TextElement<P> {
    pub fn count_lines(&self) -> usize {
        match &self.break_info {
            BreakType::None => 0,
            BreakType::Mandatory => 0,
            BreakType::Forbidden(n) => *n,
            BreakType::Atomic(n) => *n,
            BreakType::Disposable(n) => *n,
            BreakType::Point(break_point) => break_point.line_no,
            BreakType::List(break_points) => {
                if let Some(break_point) = break_points.last() {
                    break_point.line_no
                } else {
                    0
                }
            },
        }
    }

    pub fn select_break(&self, lines_remaining: i32) -> BreakType {
        let total_height = self.count_lines();

        if total_height as i32 <= lines_remaining { // entire paragraph fits
            return BreakType::None;
        }

        let mut prev_break_info: BreakType = BreakType::None;

        match &self.break_info {
            BreakType::Atomic(n) => {
                if *n as i32 <= lines_remaining {
                    if prev_break_info == BreakType::None {
                        return BreakType::Mandatory;
                    } else {
                        return prev_break_info;
                    }
                } else {
                    return BreakType::Mandatory;
                }
            },
            BreakType::List(break_points) => {
                for break_point in break_points.iter() {
                    if break_point.line_no as i32 > lines_remaining {
                        if prev_break_info == BreakType::None {
                            return BreakType::Mandatory;
                        } else {
                            return prev_break_info;
                        }
                    }

                    prev_break_info = BreakType::Point(break_point.clone());
                }
            },
            _ => (),
        }

        BreakType::Mandatory
    }
}

/// Mandatory page break
#[derive(Debug)]
pub struct PageBreak {}

/// Document root
#[derive(Debug)]
pub struct Screenplay {
    numbering: Numbering,
}

impl ContainerElement<Screenplay> {
    pub fn body(&mut self) -> Option<&mut ContainerElement<Body>> {
        for child in self.children.iter_mut() {
            match child {
                ElementType::Body(elem) => {
                    return Some(elem);
                },
                _ => {},
            }
        }

        None
    }
}

/// Series name
#[derive(Debug)]
pub struct Series {
    /// Narrow column left margin
    pub left_margin: usize,
    /// Narrow column right margin 
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Slug line
///
/// No special formatting.  Use hyphens to separate the parts, and
/// parentheses where necessary.  The period after "INT" or "EXT"
/// should be followed by a backslash so only one space will be
/// printed.
/// 
/// # Examples
///
/// ```xml
/// <slug>EXT.\ OFFICE BUILDING - CLOSE ANGLE - ENTRANCE - DAY</slug>
/// ```
///
/// Output:
///
/// <pre>EXT. OFFICE BUILDING - CLOSE ANGLE - ENTRANCE - DAY</pre>
#[derive(Debug)]
pub struct Slug {
    /// Scene number
    pub number: i32,
    /// Scene number addition
    pub addition: Option<char>,
    /// Break point information
    pub train: Vec<BreakType>,
    /// Full-width column left margin
    pub left_margin: usize,
    /// Full-width column right margin
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
    /// Numbering setting
    pub numbering: Numbering,
}

impl TextElement<Slug> {
    /// Counts the number of lines until the first valid break
    /// following the Slug element
    pub fn lines_to_first_break(&self) -> usize {
        let mut line_count: usize = 1;

        if let BreakType::Forbidden(n) = self.break_info {
            line_count += n + self.attributes.padding_after;
        }

        for break_info in self.attributes.train.iter() {
            match break_info {
                BreakType::None => (),
                BreakType::Mandatory => {
                    return line_count;
                },
                BreakType::Forbidden(h) => {
                    line_count += h;
                },
                BreakType::Atomic(h) => {
                    return line_count + h;
                },
                BreakType::Disposable(_) => {
                    return line_count;
                },
                BreakType::Point(break_point) => {
                    return line_count + break_point.line_no;
                },
                BreakType::List(break_points) => {
                    if let Some(break_point) = break_points.first() {
                        return line_count + break_point.line_no;
                    }
                },
            }
        }

        line_count
    }
}

/// Document title
#[derive(Debug)]
pub struct Title {
    /// Narrow column left margin
    pub left_margin: usize,
    /// Narrow column left margin
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

/// Scene transition
///
/// # Examples
///
/// To transition from one scene to another:
///
/// ```xml
/// <trans>CUT TO:</trans>
/// ```
///
/// Output:
///
/// <pre style="text-align: right;">CUT TO:</pre>
///
/// At the end of a sequence:
///
/// ```xml
/// <trans>FADE OUT.</trans>
/// ```
///
/// Output:
///
/// <pre style="text-align: right;">FADE OUT.</pre>
#[derive(Debug)]
pub struct Trans {
    /// Column number to begin typing at
    pub tab_stop: usize,
    /// If the transition text is longer than <tt>right_margin -
    /// tab_stop + 1</tt>, hang the text from the right margin.
    pub right_margin: usize,
    /// Number of blank lines preceding
    pub padding_before: i32,
    /// Number of blank lines following
    pub padding_after: usize,
}

