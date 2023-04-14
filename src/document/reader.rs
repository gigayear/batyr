// Batyr Document Reader
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

//! Reads a screenplay into memory
//!
//! This module reads screenplays that are encoded in XML.  The driver
//! is a pushdown automaton that accepts valid instances of the
//! [screenplay schema].  Let _M_ = (Σ, Γ, _Q_, Δ, s, _F_) where
//!
//! * Σ, the **tape alphabet**, consists of the complete set of XML
//!   elements, plus the blank symbol β,
//!
//! * Γ, the **stack alphabet**, contains the parsed data from each
//!   XML element wrapped in a [`State`] struct, plus the blank symbol
//!   γ,
//!
//! * _Q_, the set of **states**, contains the parametrized [`State`]
//!   structs, plus the initial state _s_,
//!
//! * Δ, the set of **transitions**, is specified by the grammar
//!   described in the [manuscript schema],
//!
//! * _s_, the **initial state**, is the state of the machine before
//!   the first start tag or after the last end tag,
//!
//! * _F_, the set of **final states**, contains only _s_, because the
//!   stack is only empty in state _s_ (due to the fact that a valid
//!   XML document contains a single top-level element).
//!
//! Data is collected for each element while it is open, and the
//! collected data is then pushed to an [`ElementType`] variant when
//! the state is popped off the stack.  Parent states have to do
//! something with the result.  Container elements store children for
//! later traversal.  Text elements extract the tokens and add them to
//! their own token lists, discarding the children.  The output is an
//! [`ElementType`] hierarchy with an [`ElementType::Screenplay`] at
//! the root.
//!
//! [screenplay schema]: <http://www.matchlock.com/batyr/screenplay.xsd>

use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::name::QName;

use std::str;

use crate::document::*;
use crate::text;
use crate::text::parser::Parser;

#[macro_use]
mod macros;

/// Stack alphabet
#[derive(Debug)]
pub enum State {
    Act       (TextElement     <Act       >),
    Authors   (ContainerElement<Authors   >),
    Body      (ContainerElement<Body      >),
    Br        (EmptyElement    <Br        >),
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
    Trans     (TextElement     <Trans     >),
}

impl State {
    fn on_enter(&self) {}

    fn on_exit(self) -> ElementType {
        match self {
            State::Act(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                elem.break_info = BreakType::Atomic(1);
                ElementType::Act(elem)
            },
            State::Authors(elem) => {
                ElementType::Authors(elem)
            },
            State::Body(elem) => {
                ElementType::Body(elem)
            },
            State::Br(mut elem) => {
                elem.break_info = BreakType::Disposable(1);
                ElementType::Br(elem)
            },
            State::Contact(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Contact(elem)
            },
            State::Cue(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                elem.break_info = BreakType::Forbidden(1);
                ElementType::Cue(elem)
            },
            State::D(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                State::remove_leading_eos(&mut elem.tokens);

                if elem.attributes.indent > 0 {
                    elem.tokens.insert(0, TokenType::Space(
                        Token::from(elem.attributes.indent)
                    ));
                }
                
                let w = elem.attributes.right_margin
                    - elem.attributes.left_margin + 1;

                let b = State::find_break_points(&elem.tokens[..], w);

                if b.len() == 1 {
                    elem.break_info = BreakType::Atomic(b[0].line_no);

                } else if b.len() > 1 {
                    elem.break_info = BreakType::List(b);
                }

                ElementType::D(elem)
            },
            State::Dir(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);

                let w = elem.attributes.right_margin
                    - elem.attributes.left_margin + 1;

                let n = text::count_lines(&elem.tokens[..], w);

                elem.break_info = BreakType::Forbidden(n);
                
                ElementType::Dir(elem)
            },
            State::Em(elem) => {
                ElementType::Em(elem)
            },
            State::End(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                elem.break_info = BreakType::Atomic(1);
                ElementType::End(elem)
            },
            State::FullName(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::FullName(elem)
            },
            State::Head(elem) => {
                ElementType::Head(elem)
            },
            State::Note(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Note(elem)
            },
            State::Open(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                elem.break_info = BreakType::Atomic(1);
                ElementType::Open(elem)
            },
            State::P(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                State::remove_leading_eos(&mut elem.tokens);

                if elem.attributes.indent > 0 {
                    elem.tokens.insert(0, TokenType::Space(
                        Token::from(elem.attributes.indent)
                    ));
                }
                
                let w = elem.attributes.right_margin
                    - elem.attributes.left_margin + 1;

                let b = State::find_break_points(&elem.tokens[..], w);

                if b.len() == 1 {
                    elem.break_info = BreakType::Atomic(b[0].line_no);

                } else if b.len() > 1 {
                    elem.break_info = BreakType::List(b);
                }

                ElementType::P(elem)
            },
            State::PageBreak(mut elem) => {
                elem.break_info = BreakType::Mandatory;
                ElementType::PageBreak(elem)
            },
            State::Screenplay(elem) => {
                ElementType::Screenplay(elem)
            },
            State::Series(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                elem.break_info = BreakType::Atomic(1);
                ElementType::Series(elem)
            },
            State::Slug(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);

                let w = elem.attributes.right_margin
                    - elem.attributes.left_margin + 1;

                let n = text::count_lines(&elem.tokens[..], w);
                
                elem.break_info = BreakType::Forbidden(n);

                ElementType::Slug(elem)
            },
            State::Title(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);

                let w = elem.attributes.right_margin
                    - elem.attributes.left_margin + 1;

                let n = text::count_lines(&elem.tokens[..], w);
                
                elem.break_info = BreakType::Atomic(n);

                ElementType::Title(elem)
            },
            State::Trans(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                elem.break_info = BreakType::Atomic(1);
                ElementType::Trans(elem)
            },
        }
    }

    fn on_pause(&self) {}

    fn on_resume(mut self, child: ElementType) -> Self {
        match self {
            State::Act(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Authors(ref mut elem) => {
                elem.children.push(child);
            },
            State::Body(ref mut elem) => {
                elem.children.push(child);
            },
            State::Br(_) => (),
            State::Contact(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Cue(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::D(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Dir(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Em(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::End(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::FullName(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Head(ref mut elem) => {
                elem.children.push(child);
            },
            State::Note(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Open(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::P(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::PageBreak(_) => (),
            State::Screenplay(ref mut elem) => {
                elem.children.push(child);
            },
            State::Series(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Slug(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Title(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Trans(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
        }

        self
    }

    fn resume_text_element<T>(elem: &mut TextElement<T>, child: ElementType) {
        match child {
            ElementType::Br(_) => {
                let token = Token {
                    data: LineBreakData {},
                    dpy: Default::default(),
                    frm: FormatFlags::MLB,
                };
                elem.tokens.push(TokenType::LineBreak(token));
            },
            ElementType::Em(child) => {
                elem.tokens.extend(child.tokens.into_iter());
            },
            _ => {},
        }
    }

    fn trim_whitespace(tokens: &mut TokenList) {
        if let Some(TokenType::Space(_)) = tokens.first() {
            tokens.remove(0);
        }

        if let Some(TokenType::Space(_)) = tokens.last() {
            tokens.pop();
        }
    }

    fn remove_leading_eos(tokens: &mut TokenList) {
        if let Some(TokenType::Punct(token)) = tokens.first_mut() {
            token.frm.remove(FormatFlags::EOS);
        }
    }
    
    fn find_break_points(tokens: &[TokenType], line_length: usize)
                   -> BreakPointList
    {
        let mut break_points: BreakPointList = Vec::new();
        let mut n: usize = 1; // line count
        let mut x: usize = 0; // char count
        let mut at_end_of_sentence: bool = false;

        for (i, token) in tokens.iter().enumerate() {
            let frm = token.format_flags();

            if at_end_of_sentence {
                if let TokenType::Punct(_) = token {
                    at_end_of_sentence = false;
                }
            }
            
            if frm.intersects(FormatFlags::MLB) {
                break_points.push(BreakPoint {
                    token_index: i + 1,
                    discard_flag: true,
                    line_no: n,
                });
                
                n += 1;
                x = 0;
                at_end_of_sentence = false;
            
            } else if frm.intersects(FormatFlags::EOS)
                && !frm.intersects(FormatFlags::DLB)
            {
                x += token.length();
                at_end_of_sentence = true;

            } else if frm.intersects(FormatFlags::DLB) {
                if at_end_of_sentence || frm.intersects(FormatFlags::EOS) {
                    break_points.push(BreakPoint {
                        token_index: i + 1,
                        discard_flag: frm.intersects(FormatFlags::DOB),
                        line_no: n,
                    });

                    at_end_of_sentence = false;
                }

                if !text::next_word_fits(tokens, line_length, i, x) {
                    n += 1;
                    x = 0;
                    
                } else {
                    x += token.length();
                }

            } else {
                x += token.length();
            }
        }

        if x >= line_length {
            n += 1;
        }
        
        break_points.push(BreakPoint {
            token_index: tokens.len(),
            discard_flag: false,
            line_no: n,
        });
        
        break_points
    }
}

/// Input driver
///
/// Accumulates a hierarchy of [`ElementType`] variants.
pub struct Reader<'a> {
    xml_reader: quick_xml::Reader<&'a [u8]>,
    stack: Vec<State>,
    next_act_no: i32,
    next_scene_no: i32,
    numbering: Numbering,
    /// Document root
    pub root: Option<ElementType>,
}

impl<'a> Reader<'a> {
    /// Construct a new reader from an XML string
    ///
    /// # Examples
    ///
    /// ```
    /// use batyr::document::reader::Reader;
    /// let reader = Reader::new("<em>Ulysses</em>");
    /// assert!(reader.root.is_none());
    /// ```
    pub fn new(xml_string: &'a str) -> Self {
        Reader {
            xml_reader: quick_xml::Reader::from_str(xml_string),
            stack: Vec::with_capacity(16),
            next_act_no: 1,
            next_scene_no: 1,
            numbering: Numbering::None,
            root: None,
        }
    }

    /// Push a state onto the stack
    fn push(&mut self, next: State) {
        if let Some(prev) = self.stack.last() {
            prev.on_pause();
        }

        next.on_enter();
        self.stack.push(next);
    }

    /// Pop a state off the stack
    fn pop(&mut self) {
        if let Some(prev) = self.stack.pop() {
            let elem = prev.on_exit();

            if let Some(next) = self.stack.pop() {
                self.stack.push(next.on_resume(elem));

            } else {
                self.root = Some(elem);
            }
        }
    }

    /// Process XML events
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::document::reader::Reader;
    /// let reader = Reader::new("<em>Ulysses</em>");
    /// let root = reader.run();
    /// assert!(root.is_some());
    /// ```
    pub fn run(mut self) -> Option<ElementType> {
        loop {
            match self.xml_reader.read_event().unwrap() {
                Event::Start(ref event) => {
                    match event.local_name().into_inner() {
                        b"act" => {
                            let number = self.next_act_no;
                            self.next_act_no += 1;

                            let elem = TextElement::new(Act {
                                number: number,
                                padding_before: if number == 1 {
                                    0
                                } else {
                                    -1
                                },
                                padding_after: 1,
                            });

                            self.push(State::Act(elem));
                        },
                        b"authors" => {
                            let elem = ContainerElement::new(Authors {});
                            self.push(State::Authors(elem));
                        },
                        b"body" => {
                            let elem = ContainerElement::new(Body {});
                            self.push(State::Body(elem));
                        },
                        b"contact" => {
                            let elem = TextElement::new(Contact {
                                left_margin: CONTACT_BEGIN,
                                right_margin: CONTACT_END,
                            });
                            self.push(State::Contact(elem));
                        },
                        b"cue" => {
                            let elem = TextElement::new(Cue {
                                tab_stop: CUE_BEGIN,
                                train: Vec::new(),
                                padding_before: 1,
                                padding_after: 0,
                            });

                            self.push(State::Cue(elem));
                        },
                        b"d" => {
                            let indent
                                = numeric_attr!(event, b"indent", usize)
                                .unwrap_or(0);

                            let elem = TextElement::new(D {
                                indent: indent,
                                left_margin: D_BEGIN,
                                right_margin: D_END,
                                padding_before: 0,
                                padding_after: 0,
                            });
                            
                            self.push(State::D(elem));
                        },
                        b"dir" => {
                            let elem = TextElement::new(Dir {
                                left_margin: DIR_BEGIN,
                                right_margin: DIR_END,
                                padding_before: 0,
                                padding_after: 0,
                            });
                            self.push(State::Dir(elem));
                        },
                        b"em" => {
                            let elem = TextElement::new(Em {});
                            self.push(State::Em(elem));
                        },
                        b"end" => {
                            let elem = TextElement::new(End {
                                padding_before: 1,
                                padding_after: 0,
                            });
                            self.push(State::End(elem));
                        },
                        b"fullName" => {
                            let elem = TextElement::new(FullName {});
                            self.push(State::FullName(elem));
                        },
                        b"head" => {
                            let elem = ContainerElement::new(Head {});
                            self.push(State::Head(elem));
                        },
                        b"note" => {
                            let elem = TextElement::new(Note {});
                            self.push(State::Note(elem));
                        },
                        b"open" => {
                            let elem = TextElement::new(Open {
                                tab_stop: P_BEGIN,
                                padding_before: 0,
                                padding_after: 1,
                            });
                            self.push(State::Open(elem));
                        },
                        b"p" => {
                            let indent = numeric_attr!(event, b"indent", usize)
                                .unwrap_or(0);

                            let elem = TextElement::new(P {
                                indent: indent,
                                left_margin: P_BEGIN,
                                right_margin: P_END,
                                padding_before: 1,
                                padding_after: 1,
                            });

                            self.push(State::P(elem));
                        },
                        b"screenplay" => {
                            let numbering = enum_attr!(
                                event, b"numbering", Numbering,
                                |x| Numbering::from(x)
                            ).unwrap_or(Numbering::None);
                            
                            let elem = ContainerElement::new(Screenplay {
                                numbering: numbering,
                            });

                            self.numbering = elem.attributes.numbering;
                            self.push(State::Screenplay(elem));
                        },
                        b"series" => {
                            let elem = TextElement::new(Series {
                                left_margin: LEFT_MARGIN + 2 * INDENT,
                                right_margin: RIGHT_MARGIN - 2 * INDENT,
                                padding_before: 0,
                                padding_after: 1,
                            });
                            self.push(State::Series(elem));
                        },
                        b"slug" => {
                            let number;
                            
	                    if let Some(n)
                                = numeric_attr!(event, b"number", i32)
                            {
	                        number = n;
                                self.next_scene_no = number + 1;

	                    } else {
                                number = self.next_scene_no;
                                self.next_scene_no += 1;
	                    }

                            let mut padding_before: i32 = 2;
                            
                            if let Some(State::Body(parent)) = self.stack.last() {
                                match parent.children.last() {
                                    Some(ElementType::Open(_)) => {
                                        padding_before = 1;
                                    },
                                    _ => (),
                                }
                            }
                            
                            let elem = TextElement::new(Slug {
                                number: number,
                                addition: char_attr!(event, b"addition"),
                                train: Vec::new(),
                                left_margin: P_BEGIN,
                                right_margin: P_END,
                                padding_before: padding_before,
                                padding_after: 1,
                                numbering: self.numbering,
                            });
                            
                            self.push(State::Slug(elem));
                        },
                        b"title" => {
                            let elem = TextElement::new(Title {
                                left_margin: LEFT_MARGIN + 2 * INDENT,
                                right_margin: RIGHT_MARGIN - 2 * INDENT,
                                padding_before: 0,
                                padding_after: 1,
                            });
                            self.push(State::Title(elem));
                        },
                        b"trans" => {
                            let elem = TextElement::new(Trans {
                                tab_stop: TRANS_BEGIN,
                                right_margin: TRANS_END,
                                padding_before: 1,
                                padding_after: 1,
                            });
                            self.push(State::Trans(elem));
                        },
                        _ => (),
                    }
                },
                Event::End(_) => self.pop(),
	        Event::Empty(ref event) => {
                    match event.local_name().into_inner() {
                        b"br" => {
                            self.push(State::Br(EmptyElement::new(Br {})));
                            self.pop();
                        },
                        b"pageBreak" => {
                            self.push(State::PageBreak(
                                EmptyElement::new(PageBreak {})
                            ));
                            self.pop();
                        },
                        _ => (),
                    }
                },
	        Event::Text(ref event) => {
                    match self.stack.pop() {
                        Some(State::Act(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Act(elem));
                        },
                        Some(State::Contact(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Contact(elem));
                        },
                        Some(State::Cue(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Cue(elem));
                        },
                        Some(State::D(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::D(elem));
                        },
                        Some(State::Dir(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Dir(elem));
                        },
                        Some(State::Em(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          DisplayFlags::EM);
                            self.stack.push(State::Em(elem));
                        },
                        Some(State::End(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::End(elem));
                        },
                        Some(State::FullName(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::FullName(elem));
                        },
                        Some(State::Note(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Note(elem));
                        },
                        Some(State::Open(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Open(elem));
                        },
                        Some(State::P(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::P(elem));
                        },
                        Some(State::Series(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Series(elem));
                        },
                        Some(State::Slug(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Slug(elem));
                        },
                        Some(State::Title(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Title(elem));
                        },
                        Some(State::Trans(mut elem)) => {
                            elem.tokens = self.parse_text(event, elem.tokens,
                                                          Default::default());
                            self.stack.push(State::Trans(elem));
                        },
                        Some(state) => self.stack.push(state),
                        None => (),
                    }
                },
	        Event::Comment(_) => (), // ignore comments
	        Event::CData(_) => (), // not handled
	        Event::Decl(_) => (), // ignore declaration
	        Event::PI(_) => (), // not handled
	        Event::DocType(_) => (), // not handled
	        Event::Eof => break,
            }
        }
        
        // post-processing
        match &mut self.root {
            Some(ElementType::Screenplay(ref mut root)) => {
                if let Some(body) = root.body() {
                    build_trains(body);
                    mark_scene_endings(body);
                }
            },
            _ => (),
        }
        
        self.root
    }

    fn parse_text(&mut self, event: &BytesText, tokens: TokenList,
                  dpy: DisplayFlags)
        -> TokenList
    {
        let text = event.unescape().unwrap();
        let mut parser = Parser::new(&text, tokens, dpy);
        parser = parser.run();
        parser.get_tokens()
    }
}

fn build_trains(body: &mut ContainerElement<Body>) {
    let n = body.children.len();
    let mut trains: Vec<(usize, Vec<BreakType>)> = Vec::new();
    
    for i in 0 .. n - 1 {
        match &body.children[i] {
            ElementType::Cue(_) => {
                let j = trains.len();
                
                trains.push((i, Vec::new()));
                
                for k in i + 1 .. n {
                    match &mut body.children[k] {
                        ElementType::D(elem) => {
                            trains[j].1.push(elem.break_info.clone());
                        },
                        ElementType::Dir(elem) => {
                            trains[j].1.push(elem.break_info.clone());
                        },
                        _ => break,
                    }
                }
            },
            _ => (),
        }
    }

    for (i, train) in trains.iter_mut() {
        match &mut body.children[*i] {
            ElementType::Cue(elem) => {
                elem.attributes.train = train.to_vec();
            },
            _ => (),
        }
    }

    for i in 0 .. n - 1 {
        match &body.children[i] {
            ElementType::Slug(_) => {
                let j = trains.len();
                
                trains.push((i, Vec::new()));

                for k in i + 1 .. n {
                    match &mut body.children[k] {
                        ElementType::Cue(elem) => {
                            trains[j].1.push(elem.break_info.clone());
                            trains[j].1.extend_from_slice(&elem.attributes.train[..]);
                            break;
                        },
                        ElementType::P(elem) => {
                            trains[j].1.push(elem.break_info.clone());
                            break;
                        },
                        _ => break,
                    }
                }
            },
            _ => (),
        }
    }

    for (i, train) in trains {
        match &mut body.children[i] {
            ElementType::Slug(elem) => {
                elem.attributes.train = train;
            },
            _ => (),
        }
    }
}

fn mark_scene_endings(body: &mut ContainerElement<Body>) {
    let n = body.children.len();
    
    for i in 1..n {
        match &body.children[i] {
            ElementType::Slug(_) => {
                for j in i - 1 ..= 0 {
                    match &mut body.children[j] {
                        ElementType::Act(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Cue(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::D(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Dir(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Em(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::End(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::FullName(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Open(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::P(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Series(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Slug(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Title(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        ElementType::Trans(elem) => {
                            elem.at_scene_end = true;
                            break;
                        },
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }

    // If a D or Dir is at_scene_end, propagate the at_scene_end flag
    // to the preceding Cue.  If a P element at the end of a scene is
    // immediately preceded by a Slug, propagate the at_scene_end flag
    // to the Slug.
    for i in 1..n {
        match &body.children[i] {
            ElementType::D(d_elem) => {
                if d_elem.at_scene_end {
                    for j in i - 1 ..= 0 {
                        match &mut body.children[j] {
                            ElementType::Cue(cue_elem) => {
                                cue_elem.at_scene_end = true;
                                break;
                            },
                            ElementType::D(_) => (),
                            ElementType::Dir(_) => (),
                            _ => break,
                        }
                    }
                }
            },
            ElementType::Dir(dir_elem) => {
                if dir_elem.at_scene_end {
                    for j in i - 1 ..= 0 {
                        match &mut body.children[j] {
                            ElementType::Cue(cue_elem) => {
                                cue_elem.at_scene_end = true;
                                break;
                            },
                            ElementType::D(_) => (),
                            ElementType::Dir(_) => (),
                            _ => break,
                        }
                    }
                }
            },
            ElementType::P(p_elem) => {
                if p_elem.at_scene_end {
                    match &mut body.children[i - 1] {
                        ElementType::Slug(slug_elem) => {
                            slug_elem.at_scene_end = true;
                        },
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }
}
