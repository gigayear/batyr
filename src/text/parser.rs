// Batyr Text Parser
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

//! Tokenizes text element contents
//!
//! This parser is an implementation of a finite state automaton _M_ =
//! (_Q_, _q_<sub>0</sub>, _A_, Σ, δ) where
//!
//! * _Q_ = {<tt>Close</tt>, <tt>Escape</tt>, <tt>Open</tt>,
//!   <tt>Punct</tt>, <tt>Scan</tt>, <tt>Space</tt>, <tt>Symbol</tt>,
//!   <tt>Word</tt>} is the set of **states**,
//!
//! * _q_<sub>0</sub> = <tt>Scan</tt> is the **start state**,
//!
//! * _A_ = {<tt>Scan</tt>} is the set of **accepting states**,
//!
//! * Σ = {_x_|_x_ is a UTF-8 character in the Latin-9 character set}
//!   is the **input alphabet**,
//!
//! * δ, the **transition function**, is best described as a
//!   hub-and-spoke arrangement with the <tt>Scan</tt> state at the
//!   hub.  Each state besides <tt>Scan</tt> has a character class
//!   associated with it.  A character within the class causes the
//!   machine to enter the corresponding state.  The following states
//!   consume one character and then return to the <tt>Scan</tt>
//!   state:
//!
//!      * <tt>Close</tt> consumes a close quote, parenthesis,
//!        bracket, etc.
//!
//!      * <tt>Escape</tt> consumes a backslash character or a
//!        non-whitespace character.
//!
//!      * <tt>Open</tt> consumes an open quote, parenthesis, bracket,
//!        etc.
//!
//!      * <tt>Punct</tt> consumes a punctuation character.
//!
//!      * <tt>Symbol</tt> consumes a symbol character.
//!
//!   In the <tt>Word</tt> state, the machine consumes alphanumeric
//!   characters, returning to the <tt>Scan</tt> state when a
//!   non-alphanumeric character appears.  The accumulated string of
//!   characters is copied into a word token.
//!
//!   In the <tt>Space</tt> state, the machine consumes whitespace
//!   characters, returning to the <tt>Scan</tt> state when a
//!   non-whitespace character appears.  More than one whitespace
//!   character is collapsed to a single space character, except when
//!   preceded by a <tt>Punct</tt> token containing a full stop, in
//!   which case two space characters are copied into the space token.
//!   Between the space token and the preceding <tt>Punct</tt>, any
//!   number of <tt>Close</tt> tokens may appear; they are ignored
//!   with respect to the length of the space token.
//!
//!   In the <tt>Escape</tt> state, when the escaped character is a
//!   whitespace character, the machine consumes whitespace
//!   characters, returning to the <tt>Scan</tt> state when a
//!   non-whitespace character appears.  A single space character is
//!   copied into the space token.
//!
//! The parser's output is a [`TokenList`].  The tokens correspond to
//! the states, except that there is no <tt>Escape</tt> token and no
//! <tt>Scan</tt> token.  Escape characters end up as either a
//! <tt>Space</tt> token or a <tt>Symbol</tt> token, and the
//! <tt>Scan</tt> state does not produce any tokens.
//!
//! For a complete listing of the characters in each class, see the
//! corresponding [`TokenType`] variant.

use std::collections::VecDeque;

use crate::text::tokens::*;

/// Driver for parsing text element contents
pub struct Parser {
    /// The current state
    state: StateMachine,

    /// For stepping through the input string a character at a time in order
    buffer: VecDeque<char>,
}

impl Parser {
    /// Create a new parser.
    ///
    /// If the newly generated tokens are supposed to be appended to a
    /// `TokenList`, pass the partial list to this constructor so the
    /// parser can inspect the preceding tokens.
    ///
    /// Display flags are copied to every token generated by this
    /// parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use batyr::text::{parser::Parser, tokens::TokenList};
    /// let tokens: TokenList = Vec::new();
    /// let parser = Parser::new("foo bar", tokens, Default::default());
    /// ```
    pub fn new(input_string: &str, tokens: TokenList, dpy: DisplayFlags) -> Self {
        Parser {
            state: StateMachine::Scan(State::new(tokens, dpy)),
            buffer: VecDeque::from_iter(input_string.chars()),
        }
    }

    /// Consume the input string and generate tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::text::{parser::Parser, tokens::TokenList};
    /// let tokens: TokenList = Vec::new();
    /// let mut parser = Parser::new("foo bar", tokens, Default::default());
    /// parser = parser.run();
    /// assert_eq!(parser.get_tokens().len(), 3);
    /// ```
    pub fn run(mut self) -> Self {
        loop {
            if self.buffer.is_empty() {
                self.state = self.state.flush();
                break;
            }

            if let Some(ch) = self.buffer.front() {
                let consume_flag: bool;
                
                (self.state, consume_flag) = self.state.step(ch);

                if consume_flag {
                    self.buffer.pop_front();
                }
            }
        }

        self
    }

    /// Returns the tokens accumulated by the parser
    pub fn get_tokens(self) -> TokenList {
        self.state.get_tokens()
    }
}

// states

/// Generic state struct collects token data and moves it through the
/// state machine
#[derive(Debug)]
struct State<Data> {
    /// Element-specific data copied from the XML reader
    data: Data,
    /// Token accumulator
    tokens: TokenList,
    /// Counter for word tokens
    word_count: usize,
    /// Changes in the display state cause line segmentation downstream.
    dpy: DisplayFlags,
    /// Format flags are used by the line breaking algorithm, and for
    /// end-of-sentence horizontal spacing.
    frm: FormatFlags,
}

#[doc(hidden)]
#[derive(Debug)]
struct ScanData {}

impl State<ScanData> {
    fn new(tokens: TokenList, dpy: DisplayFlags) -> Self {
        State {
            data: ScanData {},
            tokens: tokens,
            word_count: 0,
            dpy: dpy,
            frm: Default::default(),
        }
    }
}

#[doc(hidden)]
impl<Data> State<Data> {
    fn at_full_stop(&self) -> bool {
        for token in self.tokens.iter().rev() {
            match token {
                TokenType::Close(_) => {},
                TokenType::Punct(token) => {
                    return token.frm.intersects(FormatFlags::FS);
                },
               _ => break,
            }
        }

        return false;
    }

    fn remove_preceding_full_stop_flag(&mut self) {
        for token in self.tokens.iter_mut().rev() {
            match token {
                TokenType::Close(_) => {},
                TokenType::Punct(token) => {
                    token.frm.remove(FormatFlags::FS | FormatFlags::EOS);
                },
               _ => break,
            }
        }
    }
}

/// Enum type for parser states
#[derive(Debug)]
enum StateMachine {
    Close(State<CloseData>),
    Escape(State<EscapeData>),
    Open(State<OpenData>),
    Punct(State<PunctData>),
    Scan(State<ScanData>),
    Space(State<SpaceData>),
    Symbol(State<SymbolData>),
    Word(State<WordData>),
}

impl StateMachine {
    /// Process the next character in the input string.
    fn step(self, ch: &char) -> (Self, bool) {
        match self {
            StateMachine::Close(mut state) => {
                match ch {
                    '\u{0029}' |    // Right parenthesis
                    '\u{005d}' |    // Right square bracket
                    '\u{007d}' |    // Right curly bracket
                    '\u{00bb}' => { // Right-pointing double angle quotation mark
                        state.data.text.push(ch.clone());
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{2019}' => { // Right single quotation mark
                        state.data.text.push_str("\'");
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{201d}' => { // Right double quotation mark
                        state.data.text.push_str("\"");
                        (StateMachine::Scan(state.into()), true)
                    },
                    _ => (StateMachine::Scan(state.into()), true),
                }
            },
            StateMachine::Escape(mut state) => {
                if state.data.count == 0 {
                    // Advance past the first character.
                    state.data.count += 1;
                    (StateMachine::Escape(state), true)

                } else if state.data.count == 1 {
                    // Match the second character.
                    match ch {
                        ch if ch.is_whitespace() => { // collapse whitespace
                            state.data.text.push(' ');
                            state.data.count += 1;
                            (StateMachine::Escape(state.into()), true)
                        },
                        '\u{005c}' => { // Backslash
                            state.data.text.push(ch.clone());
                            (StateMachine::Scan(state.into()), true)
                        },
                        _ => {
                            // Pass the unknown character through to
                            // transition function.
                            state.data.text.push(ch.clone());
                            (StateMachine::Scan(state.into()), true)
                        },
                    }
                } else {
                    // For subsequent characters, consume any
                    // remaining whitespace, but otherwise return to
                    // Scan state.
                    match ch {
                        ch if ch.is_whitespace() => {
                            state.data.count += 1;
                            (StateMachine::Escape(state.into()), true)
                        },
                        _ => {
                            // Break out of Escape state without
                            // consuming the non-whitespace character.
                            (StateMachine::Scan(state.into()), false)
                        },
                    }
                }
            },
            StateMachine::Open(mut state) => {
                match ch {
                    '\u{0028}' |    // Left parenthesis
                    '\u{005b}' |    // Left square bracket
                    '\u{007b}' |    // Left curly bracket
                    '\u{00ab}' => {  // Left-pointing double angle quotation mark
                        state.data.text.push(ch.clone());
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{2018}' => { // Left single quotation mark
                        state.data.text.push_str("\'");
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{201c}' => { // Left double quotation mark
                        state.data.text.push_str("\"");
                        (StateMachine::Scan(state.into()), true)
                    },
                    _ => (StateMachine::Scan(state.into()), true),
                }
            },
            StateMachine::Punct(mut state) => {
                match ch {
                    '\u{0027}' |    // Apostrophe
                    '\u{002c}' |    // Comma
                    '\u{003b}' |    // Semicolon
                    '\u{00a1}' |    // Inverted exclamation mark
                    '\u{00bf}' => { // Inverted question mark
                        state.data.text.push(ch.clone());
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{0021}' |    // Exclamation mark
                    '\u{002e}' |    // Full stop
                    '\u{003f}' => { // Question mark
                        state.data.text.push(ch.clone());
                        state.frm.insert(FormatFlags::FS | FormatFlags::EOS);
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{003a}' => { // Colon
                        state.data.text.push(ch.clone());
                        state.frm.insert(FormatFlags::FS);
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{002d}' => { // Hyphen-minus
                        state.data.text.push(ch.clone());
                        state.frm.insert(FormatFlags::DLB);
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{2013}' => { // En dash
                        state.data.text.push_str("-");
                        state.frm.insert(FormatFlags::EOS);
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{2014}' => { // Em dash
                        state.data.text.push_str("--");
                        state.frm.insert(FormatFlags::EOS);
                        (StateMachine::Scan(state.into()), true)
                    },
                    '\u{2026}' => { // Horizontal ellipsis
                        state.data.text.push_str("...");
                        state.frm.insert(FormatFlags::EOS);
                        (StateMachine::Scan(state.into()), true)
                    },
                    _ => (StateMachine::Scan(state.into()), true),
                }
            },
            StateMachine::Scan(state) => {
                match ch {
                    '\u{0029}' |    // Right parenthesis
                    '\u{005d}' |    // Right square bracket
                    '\u{007d}' |    // Right curly bracket
                    '\u{00bb}' |    // Right-pointing double angle quotation mark
                    '\u{2019}' |    // Right single quotation mark
                    '\u{201d}' => { // Right double quotation mark
                        (StateMachine::Close(state.into()), false)
                    },
                    '\u{005c}' => { // Backslash
                        (StateMachine::Escape(state.into()), false)
                    },
                    '\u{0028}' |    // Left parenthesis
                    '\u{005b}' |    // Left square bracket
                    '\u{007b}' |    // Left curly bracket
                    '\u{00ab}' |    // Left-pointing double angle quotation mark
                    '\u{2018}' |    // Left single quotation mark
                    '\u{201c}' => { // Left double quotation mark
                        (StateMachine::Open(state.into()), false)
                    },
                    '\u{0021}' |    // Exclamation mark
                    '\u{0027}' |    // Apostrophe
                    '\u{002c}' |    // Comma
                    '\u{002d}' |    // Hyphen-minus
                    '\u{002e}' |    // Full stop
                    '\u{003a}' |    // Colon
                    '\u{003b}' |    // Semicolon
                    '\u{003f}' |    // Question mark
                    '\u{00a1}' |    // Inverted exclamation mark
                    '\u{00bf}' |    // Inverted question mark
                    '\u{2013}' |    // En dash
                    '\u{2014}' |    // Em dash
                    '\u{2026}' => { // Horizontal ellipsis
                        (StateMachine::Punct(state.into()), false)
                    },
                    ch if ch.is_whitespace() => {
                        (StateMachine::Space(state.into()), false)
                    },
                    '\u{0022}' |    // Quotation mark
                    '\u{0023}' |    // Number sign
                    '\u{0024}' |    // Dollar sign
                    '\u{0025}' |    // Percent sign
                    '\u{0026}' |    // Ampersand
                    '\u{002a}' |    // Asterisk
                    '\u{002b}' |    // Plus sign
                    '\u{002f}' |    // Slash
                    '\u{003c}' |    // Less-than sign
                    '\u{003d}' |    // Equal sign
                    '\u{003e}' |    // Greater-than sign
                    '\u{0040}' |    // At sign
                    '\u{005e}' |    // Circumflex accent
                    '\u{005f}' |    // Low line
                    '\u{0060}' |    // Grave accent
                    '\u{007c}' |    // Vertical bar
                    '\u{007e}' |    // Tilde
                    '\u{00a2}' |    // Cent sign
                    '\u{00a3}' |    // Pound sign
                    '\u{00a5}' |    // Yen sign
                    '\u{00a7}' |    // Section sign
                    '\u{00a9}' |    // Copyright sign
                    '\u{00ac}' |    // Not sign
                    '\u{00ae}' |    // Registered sign
                    '\u{00af}' |    // Macron
                    '\u{00b0}' |    // Degree sign
                    '\u{00b1}' |    // Plus-minus sign
                    '\u{00b6}' |    // Pilcrow sign
                    '\u{00b7}' |    // Middle dot
                    '\u{00d7}' |    // Multiplication sign
                    '\u{00f7}' |    // Division sign
                    '\u{20ac}' => { // Euro sign
                        (StateMachine::Symbol(state.into()), false)
                    },
                    ch if ch.is_alphanumeric() => {
                        (StateMachine::Word(state.into()), false)
                    },
                    _ => (StateMachine::Scan(state), true),
                }
            },
            StateMachine::Space(mut state) => {
                match ch {
                    ch if ch.is_whitespace() => {
                        state.data.text.push(ch.clone());
                        (StateMachine::Space(state), true)
                    },
                    _ => (StateMachine::Scan(state.into()), false),
                }
            },
            StateMachine::Symbol(mut state) => {
                match ch {
                    '\u{0022}' |    // Quotation mark
                    '\u{0023}' |    // Number sign
                    '\u{0024}' |    // Dollar sign
                    '\u{0025}' |    // Percent sign
                    '\u{0026}' |    // Ampersand
                    '\u{002a}' |    // Asterisk
                    '\u{002b}' |    // Plus sign
                    '\u{002f}' |    // Slash
                    '\u{003c}' |    // Less-than sign
                    '\u{003d}' |    // Equal sign
                    '\u{003e}' |    // Greater-than sign
                    '\u{0040}' |    // At sign
                    '\u{005e}' |    // Circumflex accent
                    '\u{005f}' |    // Low line
                    '\u{0060}' |    // Grave accent
                    '\u{007c}' |    // Vertical bar
                    '\u{007e}' |    // Tilde
                    '\u{00a2}' |    // Cent sign
                    '\u{00a3}' |    // Pound sign
                    '\u{00a5}' |    // Yen sign
                    '\u{00a7}' |    // Section sign
                    '\u{00a9}' |    // Copyright sign
                    '\u{00ac}' |    // Not sign
                    '\u{00ae}' |    // Registered sign
                    '\u{00af}' |    // Macron
                    '\u{00b0}' |    // Degree sign
                    '\u{00b1}' |    // Plus-minus sign
                    '\u{00b6}' |    // Pilcrow sign
                    '\u{00b7}' |    // Middle dot
                    '\u{00d7}' |    // Multiplication sign
                    '\u{00f7}' |    // Division sign
                    '\u{20ac}' => { // Euro sign
                        state.data.text.push(ch.clone());
                        (StateMachine::Scan(state.into()), true)
                    },
                    _ => (StateMachine::Scan(state.into()), true),
                }
            },
            StateMachine::Word(mut state) => {
                match ch {
                    ch if ch.is_alphanumeric() => {
                        state.data.text.push(ch.clone());
                        (StateMachine::Word(state), true)
                    },
                    _ => (StateMachine::Scan(state.into()), false),
                }
            },
        }
    }

    /// Force the machine into `Scan` state.
    ///
    /// If we reach the end of the input buffer in a non-`Scan` state,
    /// there may be token data left in the state.  Make sure it gets
    /// converted to tokens.
    fn flush(self) -> Self {
        match self {
            StateMachine::Close(state) => StateMachine::Scan(state.into()),
            StateMachine::Escape(state) => StateMachine::Scan(state.into()),
            StateMachine::Open(state) => StateMachine::Scan(state.into()),
            StateMachine::Punct(state) => StateMachine::Scan(state.into()),
            StateMachine::Scan(state) => StateMachine::Scan(state),
            StateMachine::Space(state) => StateMachine::Scan(state.into()),
            StateMachine::Symbol(state) => StateMachine::Scan(state.into()),
            StateMachine::Word(state) => StateMachine::Scan(state.into()),
        }
    }

    /// Provide access to the token list through the enum wrapper.
    fn get_tokens(self) -> TokenList {
        match self {
            StateMachine::Close(state) => state.tokens,
            StateMachine::Escape(state) => state.tokens,
            StateMachine::Open(state) => state.tokens,
            StateMachine::Punct(state) => state.tokens,
            StateMachine::Scan(state) => state.tokens,
            StateMachine::Space(state) => state.tokens,
            StateMachine::Symbol(state) => state.tokens,
            StateMachine::Word(state) => state.tokens,
        }
    }

    /// Provide access to the word count through the enum wrapper.
    #[allow(dead_code)]
    fn word_count(&self) -> usize {
        match self {
            StateMachine::Close(state) => state.word_count,
            StateMachine::Escape(state) => state.word_count,
            StateMachine::Open(state) => state.word_count,
            StateMachine::Punct(state) => state.word_count,
            StateMachine::Scan(state) => state.word_count,
            StateMachine::Space(state) => state.word_count,
            StateMachine::Symbol(state) => state.word_count,
            StateMachine::Word(state) => state.word_count,
        }
    }
}

// transitions

impl From<State<CloseData>> for State<ScanData> {
    fn from(mut state: State<CloseData>) -> State<ScanData> {
        state.tokens.push(TokenType::Close(Token::<CloseData> {
            data: state.data,
            dpy: state.dpy,
            frm: state.frm,
        }));

        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<CloseData> {
    fn from(state: State<ScanData>) -> State<CloseData> {
        State {
            data: CloseData {
                text: String::new(),
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<EscapeData>> for State<ScanData> {
    fn from(mut state: State<EscapeData>) -> Self {
        match state.data.text.as_str() {
            "\\" => {
                state.tokens.push(TokenType::Symbol(Token {
                    data: SymbolData {
                        text: "\\".to_string(),
                    },
                    dpy: state.dpy,
                    frm: state.frm,
                }));
            },
            " " => {
                state.remove_preceding_full_stop_flag();
                state.tokens.push(TokenType::Space(Token {
                    data: SpaceData {
                        text: " ".to_string(),
                    },
                    dpy: state.dpy,
                    frm: state.frm | FormatFlags::DLB | FormatFlags::DOB,
                }));
            },
            ch => {
                eprintln!("Ignoring unknown escape sequence '\\{}'", ch);
            },
        }

        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<EscapeData> {
    fn from(state: State<ScanData>) -> State<EscapeData> {
        State {
            data: EscapeData {
                text: String::new(),
                count: 0,
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<OpenData>> for State<ScanData> {
    fn from(mut state: State<OpenData>) -> State<ScanData> {
        state.tokens.push(TokenType::Open(Token::<OpenData> {
            data: state.data,
            dpy: state.dpy,
            frm: state.frm,
        }));

        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<OpenData> {
    fn from(state: State<ScanData>) -> State<OpenData> {
        State {
            data: OpenData {
                text: String::new(),
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<PunctData>> for State<ScanData> {
    fn from(mut state: State<PunctData>) -> State<ScanData> {
        state.tokens.push(TokenType::Punct(Token::<PunctData> {
            data: state.data,
            dpy: state.dpy,
            frm: state.frm,
        }));

        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<PunctData> {
    fn from(state: State<ScanData>) -> State<PunctData> {
        State {
            data: PunctData {
                text: String::new(),
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<SpaceData>> for State<ScanData> {
    fn from(mut state: State<SpaceData>) -> State<ScanData> {
        let mut discard_flag: bool = false;
        
        if let Some(token) = state.tokens.last() {
            if token.format_flags().intersects(FormatFlags::MLB) {
                discard_flag = true;
            }
        }
            
        if !discard_flag {
            let text = if state.at_full_stop() {
                "  ".to_string()
            } else {
                " ".to_string()
            };

            state.tokens.push(TokenType::Space(Token::<SpaceData> {
                data: SpaceData {
                    text: text,
                },
                dpy: state.dpy,
                frm: state.frm | FormatFlags::DLB | FormatFlags::DOB,
            }));
        }
        
        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<SpaceData> {
    fn from(state: State<ScanData>) -> State<SpaceData> {
        State {
            data: SpaceData {
                text: String::new(),
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<SymbolData>> for State<ScanData> {
    fn from(mut state: State<SymbolData>) -> State<ScanData> {
        state.tokens.push(TokenType::Symbol(Token::<SymbolData> {
            data: state.data,
            dpy: state.dpy,
            frm: state.frm,
        }));

        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<SymbolData> {
    fn from(state: State<ScanData>) -> State<SymbolData> {
        State {
            data: SymbolData {
                text: String::new(),
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<WordData>> for State<ScanData> {
    fn from(mut state: State<WordData>) -> State<ScanData> {
        state.tokens.push(TokenType::Word(Token::<WordData> {
            data: state.data,
            dpy: state.dpy,
            frm: state.frm,
        }));

        State {
            data: ScanData {},
            tokens: state.tokens,
            word_count: state.word_count + 1,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}

impl From<State<ScanData>> for State<WordData> {
    fn from(state: State<ScanData>) -> State<WordData> {
        State {
            data: WordData {
                text: String::new(),
            },
            tokens: state.tokens,
            word_count: state.word_count,
            dpy: state.dpy,
            frm: Default::default(),
        }
    }
}
