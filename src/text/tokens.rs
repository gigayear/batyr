// Batyr Text Tokens
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

//! Text tokens
//!
//! The text parser will collapse contiguous whitespace to a single
//! space character (<tt>U+0020</tt>) in all cases but one.
//! Typewriter style requires two spaces after a full-stop character:
//! an exclamation mark (<tt>U+0021</tt>), a period (<tt>U+002e</tt>),
//! a colon (<tt>U+003a</tt>), or a question mark (<tt>U+003f</tt>).

use bitflags::bitflags;

use std::iter::repeat;

/// Generic token data wrapper
#[derive(Debug, Clone)]
pub struct Token<Data> {
    /// Generic storage buffer
    pub data: Data,
    /// Changes to the display state are handled in Postscript.
    pub dpy: DisplayFlags,
    /// Format flags affect line breaking.
    pub frm: FormatFlags,
}

impl<Data> Token<Data> {
    /// Construct a new token
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::text::tokens::*;
    /// let token = Token::new(WordData::from("foo"),
    ///                        DisplayFlags::EM,
    ///                        Default::default());
    /// assert_eq!(token.data.text, r"foo");
    /// ```
    pub fn new(data: Data, dpy: DisplayFlags, frm: FormatFlags) -> Self {
        Self {
            data: data,
            dpy: dpy,
            frm: frm,
        }
    }
}

/// Token types
#[derive(Debug, Clone)]
pub enum TokenType {
    Close(Token<CloseData>),
    LineBreak(Token<LineBreakData>),
    Open(Token<OpenData>),
    Punct(Token<PunctData>),
    Space(Token<SpaceData>),
    Symbol(Token<SymbolData>),
    Word(Token<WordData>),
}

impl TokenType {
    /// Retrieves the token length from the associated generic token
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::text::tokens::*;
    /// let token = TokenType::Word(Token::new(WordData::from("foo"),
    ///                                        DisplayFlags::EM,
    ///                                        Default::default()));
    /// assert_eq!(token.length(), 3);
    /// ```
    pub fn length(&self) -> usize {
        match self {
            TokenType::Close    (token) => token.data.text.chars().count(),
            TokenType::LineBreak(_    ) => 0,
            TokenType::Open     (token) => token.data.text.chars().count(),
            TokenType::Punct    (token) => token.data.text.chars().count(),
            TokenType::Space    (token) => token.data.text.chars().count(),
            TokenType::Symbol   (token) => token.data.text.chars().count(),
            TokenType::Word     (token) => token.data.text.chars().count(),
        }
    }

    /// Retrieves the text from the associated generic token
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::text::tokens::*;
    /// let token = TokenType::Word(Token::new(WordData::from("foo"),
    ///                                        DisplayFlags::EM,
    ///                                        Default::default()));
    /// assert_eq!(token.text(), r"foo");
    /// ```
    pub fn text(&self) -> String {
        match self {
            TokenType::Close    (token) => token.data.text.clone(),
            TokenType::LineBreak(_    ) => String::new(),
            TokenType::Open     (token) => token.data.text.clone(),
            TokenType::Punct    (token) => token.data.text.clone(),
            TokenType::Space    (token) => token.data.text.clone(),
            TokenType::Symbol   (token) => token.data.text.clone(),
            TokenType::Word     (token) => token.data.text.clone(),
        }
    }

    /// Retrieves the display flags from the associated generic token
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::text::tokens::*;
    /// let token = TokenType::Word(Token::new(WordData::from("foo"),
    ///                                        DisplayFlags::EM,
    ///                                        Default::default()));
    /// assert!(token.display_flags().intersects(DisplayFlags::EM));
    /// ```
    pub fn display_flags(&self) -> DisplayFlags {
        match self {
            TokenType::Close    (token) => token.dpy,
            TokenType::LineBreak(token) => token.dpy,
            TokenType::Open     (token) => token.dpy,
            TokenType::Punct    (token) => token.dpy,
            TokenType::Space    (token) => token.dpy,
            TokenType::Symbol   (token) => token.dpy,
            TokenType::Word     (token) => token.dpy,
        }
    }

    /// Retrieves the display flags from the associated generic token
    ///
    /// # Examples
    ///
    /// ```
    /// # use batyr::text::tokens::*;
    /// let token = TokenType::Word(Token::new(WordData::from("foo"),
    ///                                        DisplayFlags::EM,
    ///                                        Default::default()));
    /// assert!(token.format_flags().is_empty());
    /// ```
    pub fn format_flags(&self) -> FormatFlags {
        match self {
            TokenType::Close    (token) => token.frm,
            TokenType::LineBreak(token) => token.frm,
            TokenType::Open     (token) => token.frm,
            TokenType::Punct    (token) => token.frm,
            TokenType::Space    (token) => token.frm,
            TokenType::Symbol   (token) => token.frm,
            TokenType::Word     (token) => token.frm,
        }
    }
}

/// Token accumulator
pub type TokenList = Vec<TokenType>;

bitflags! {
    /// Display feature selection
    #[derive(Default)]
    pub struct DisplayFlags: u32 {
        const EM    = 0b00000001; // emphasis
    }
}

impl DisplayFlags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

bitflags! {
    /// Format feature selection
    #[derive(Default)]
    pub struct FormatFlags: u32 {
        const FS    = 0b00000001; // full stop
        const DLB   = 0b00000010; // discretionary line break
        const MLB   = 0b00000100; // mandatory line break
        const DOB   = 0b00001000; // discard-on-break
        const EOS   = 0b00010000; // end-of-sentence
    }
}

impl FormatFlags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

/// End-of-group characters
///
/// | Glyph | UTF-8 Code      | Description                                | Latin-9 Equivalent             |
/// | ----- | --------------- | ------------------------------------------ | ------------------------------ |
/// | )     | <tt>U+0029</tt> | Right parenthesis                          | <tt>0x29</tt>                  |
/// | ]     | <tt>U+005d</tt> | Right square bracket                       | <tt>0x5d</tt>                  |
/// | }     | <tt>U+007d</tt> | Right curly bracket                        | <tt>0x7d</tt>                  |
/// | »     | <tt>U+00bb</tt> | Right-pointing double angle quotation mark | <tt>0xbb</tt>                  |
/// | ’     | <tt>U+2019</tt> | Right single quotation mark                | <tt>0x27</tt> (Apostrophe)     |
/// | ”     | <tt>U+201d</tt> | Right double quotation mark                | <tt>0x22</tt> (Quotation mark) |
#[derive(Debug, Clone)]
pub struct CloseData {
    /// Stores one end-of-group character
    pub text: String,
}

/// Escape characters
///
/// | Glyph | UTF-8 Code      | Description    | Latin-9 Equivalent |
/// | ----- | --------------- | -------------- | ------------------ |
/// |       | <tt>U+0009</tt> | Horizontal tab | <tt>0x09</tt>      |
/// |       | <tt>U+000a</tt> | Line feed      | <tt>0x0a</tt>      |
/// |       | <tt>U+0020</tt> | Space          | <tt>0x20</tt>      |
/// | \     | <tt>U+005c</tt> | Backslash      | <tt>0x5c</tt>      |
#[derive(Debug, Clone)]
pub struct EscapeData {
    /// Stores a space character or a backslash
    pub text: String,
    pub count: u32,
}

/// A unit struct signalling a mandatory line break
#[derive(Debug, Clone)]
pub struct LineBreakData {}

/// Start-of-group characters
///
/// | Glyph | UTF-8 Code      | Description                               | Latin-9 Equivalent             |
/// | ----- | --------------- | ----------------------------------------- | ------------------------------ |
/// | (     | <tt>U+0028</tt> | Left parenthesis                          | <tt>0x28</tt>                  |
/// | [     | <tt>U+005b</tt> | Left square bracket                       | <tt>0x5b</tt>                  |
/// | {     | <tt>U+007b</tt> | Left curly bracket                        | <tt>0x7b</tt>                  |
/// | «     | <tt>U+00ab</tt> | Left-pointing double angle quotation mark | <tt>0xab</tt>                  |
/// | ‘     | <tt>U+2018</tt> | Left single quotation mark                | <tt>0x27</tt> (Apostrophe)     |
/// | “     | <tt>U+201c</tt> | Left double quotation mark                | <tt>0x22</tt> (Quotation mark) |
#[derive(Debug, Clone)]
pub struct OpenData {
    /// Stores one start-of-group character
    pub text: String,
}

/// Punctuation characters
///
/// | Glyph | UTF-8 Code      | Description               | Latin-9 Equivalent |
/// | ----- | --------------- | ------------------------- | ------------------ |
/// | !     | <tt>U+0021</tt> | Exclamation mark          | <tt>0x21</tt>      |
/// | '     | <tt>U+0027</tt> | Apostrophe                | <tt>0x27</tt>      |
/// | ,     | <tt>U+002c</tt> | Comma                     | <tt>0x2c</tt>      |
/// | -     | <tt>U+002d</tt> | Hyphen-minus              | <tt>0x2c</tt>      |
/// | .     | <tt>U+002e</tt> | Full stop                 | <tt>0x2e</tt>      |
/// | :     | <tt>U+003a</tt> | Colon                     | <tt>0x3a</tt>      |
/// | ;     | <tt>U+003b</tt> | Semicolon                 | <tt>0x3b</tt>      |
/// | ?     | <tt>U+003f</tt> | Question mark             | <tt>0x3f</tt>      |
/// | ¡     | <tt>U+00a1</tt> | Inverted exclamation mark | <tt>0x3f</tt>      |
/// | ¿     | <tt>U+00bf</tt> | Inverted question mark    | <tt>0xbf</tt>      |
/// | –     | <tt>U+2013</tt> | En-dash                   | <tt>0x2c</tt>      |
/// | —     | <tt>U+2014</tt> | Em-dash                   | <tt>0x2c2c</tt>    |
/// | …     | <tt>U+2026</tt> | Horizontal ellipsis       | <tt>0x272727</tt>  |
#[derive(Debug, Clone)]
pub struct PunctData {
    pub text: String,
}

/// Space characters
///
/// | Glyph | UTF-8 Code      | Description    | Latin-9 Equivalent |
/// | ----- | --------------- | -------------- | ------------------ |
/// |       | <tt>U+0009</tt> | Horizontal tab | <tt>0x09</tt>      |
/// |       | <tt>U+000a</tt> | Line feed      | <tt>0x0a</tt>      |
/// |       | <tt>U+0020</tt> | Space          | <tt>0x20</tt>      |
#[derive(Debug, Clone)]
pub struct SpaceData {
    /// Stores any number of space characters.
    pub text: String,
}

impl From<usize> for SpaceData {
    fn from(n: usize) -> Self {
        Self {
	    text: repeat(' ').take(n).collect::<String>(),
        }
    }
}

/// Symbol characters
///
/// | Glyph | UTF-8 Code      | Description         | Latin-9 Equivalent |
/// | ----- | --------------- | ------------------- | ------------------ |
/// | "     | <tt>U+0022</tt> | Quotation mark      | <tt>0x22</tt>      |
/// | #     | <tt>U+0023</tt> | Number sign         | <tt>0x23</tt>      |
/// | $     | <tt>U+0024</tt> | Dollar sign         | <tt>0x24</tt>      |
/// | %     | <tt>U+0025</tt> | Percent sign        | <tt>0x25</tt>      |
/// | &     | <tt>U+0026</tt> | Ampersand           | <tt>0x26</tt>      |
/// | *     | <tt>U+002a</tt> | Asterisk            | <tt>0x2a</tt>      |
/// | +     | <tt>U+002b</tt> | Plus sign           | <tt>0x2b</tt>      |
/// | /     | <tt>U+002f</tt> | Slash               | <tt>0x2f</tt>      |
/// | <     | <tt>U+003c</tt> | Less-than sign      | <tt>0x3c</tt>      |
/// | =     | <tt>U+003d</tt> | Equal sign          | <tt>0x3d</tt>      |
/// | >     | <tt>U+003e</tt> | Greater-than sign   | <tt>0x3e</tt>      |
/// | @     | <tt>U+0040</tt> | At sign             | <tt>0x40</tt>      |
/// | \     | <tt>U+005c</tt> | Backslash           | <tt>0x5c</tt>      |
/// | ^     | <tt>U+005e</tt> | Circumflex accent   | <tt>0x5e</tt>      |
/// | _     | <tt>U+005f</tt> | Low line            | <tt>0x5f</tt>      |
/// | `     | <tt>U+0060</tt> | Grave accent        | <tt>0x60</tt>      |
/// | \|    | <tt>U+007c</tt> | Vertical bar        | <tt>0x7c</tt>      |
/// | ~     | <tt>U+007e</tt> | Tilde               | <tt>0x7e</tt>      |
/// | ¢     | <tt>U+00a2</tt> | Cent sign           | <tt>0xa2</tt>      |
/// | £     | <tt>U+00a3</tt> | Pound sign          | <tt>0xa3</tt>      |
/// | ¥     | <tt>U+00a5</tt> | Yen sign            | <tt>0xa5</tt>      |
/// | §     | <tt>U+00a7</tt> | Section sign        | <tt>0xa7</tt>      |
/// | ©     | <tt>U+00a9</tt> | Copyright sign      | <tt>0xa9</tt>      |
/// | ¬     | <tt>U+00ac</tt> | Not sign            | <tt>0xac</tt>      |
/// | ®     | <tt>U+00ae</tt> | Registered sign     | <tt>0xae</tt>      |
/// | ¯     | <tt>U+00af</tt> | Macron              | <tt>0xaf</tt>      |
/// | °     | <tt>U+00b0</tt> | Degree sign         | <tt>0xb0</tt>      |
/// | ±     | <tt>U+00b1</tt> | Plus-minus sign     | <tt>0xb1</tt>      |
/// | ¶     | <tt>U+00b6</tt> | Pilcrow sign        | <tt>0xb6</tt>      |
/// | ·     | <tt>U+00b7</tt> | Middle dot          | <tt>0xb7</tt>      |
/// | ×     | <tt>U+00d7</tt> | Multiplication sign | <tt>0xd7</tt>      |
/// | ÷     | <tt>U+00f7</tt> | Division sign       | <tt>0xb6</tt>      |
/// | €     | <tt>U+20ac</tt> | Euro sign           | <tt>0xa4</tt>      |
#[derive(Debug, Clone)]
pub struct SymbolData {
    /// Stores one symbol character
    pub text: String,
}

impl From<String> for SymbolData {
    fn from(text: String) -> Self {
        Self {
	    text: text,
        }
    }
}

/// Word characters
///
/// | Glyph | UTF-8 Code      | Description                            | Latin-9 Equivalent |
/// | ----- | --------------- | -------------------------------------- | ------------------ |
/// | 0     | <tt>U+0030</tt> | Digit Zero                             | <tt>0x30</tt>      |
/// | 1     | <tt>U+0031</tt> | Digit One                              | <tt>0x31</tt>      |
/// | 2     | <tt>U+0032</tt> | Digit Two                              | <tt>0x32</tt>      |
/// | 3     | <tt>U+0033</tt> | Digit Three                            | <tt>0x33</tt>      |
/// | 4     | <tt>U+0034</tt> | Digit Four                             | <tt>0x34</tt>      |
/// | 5     | <tt>U+0035</tt> | Digit Five                             | <tt>0x35</tt>      |
/// | 6     | <tt>U+0036</tt> | Digit Six                              | <tt>0x36</tt>      |
/// | 7     | <tt>U+0037</tt> | Digit Seven                            | <tt>0x37</tt>      |
/// | 8     | <tt>U+0038</tt> | Digit Eight                            | <tt>0x38</tt>      |
/// | 9     | <tt>U+0039</tt> | Digit Nine                             | <tt>0x39</tt>      |
/// | A     | <tt>U+0041</tt> | Latin capital letter A                 | <tt>0x41</tt>      |
/// | B     | <tt>U+0042</tt> | Latin capital letter B                 | <tt>0x42</tt>      |
/// | C     | <tt>U+0043</tt> | Latin capital letter C                 | <tt>0x43</tt>      |
/// | D     | <tt>U+0044</tt> | Latin capital letter D                 | <tt>0x44</tt>      |
/// | E     | <tt>U+0045</tt> | Latin capital letter E                 | <tt>0x45</tt>      |
/// | F     | <tt>U+0046</tt> | Latin capital letter F                 | <tt>0x46</tt>      |
/// | G     | <tt>U+0047</tt> | Latin capital letter G                 | <tt>0x47</tt>      |
/// | H     | <tt>U+0048</tt> | Latin capital letter H                 | <tt>0x48</tt>      |
/// | I     | <tt>U+0049</tt> | Latin capital letter I                 | <tt>0x49</tt>      |
/// | J     | <tt>U+004a</tt> | Latin capital letter J                 | <tt>0x4a</tt>      |
/// | K     | <tt>U+004b</tt> | Latin capital letter K                 | <tt>0x4b</tt>      |
/// | L     | <tt>U+004c</tt> | Latin capital letter L                 | <tt>0x4c</tt>      |
/// | M     | <tt>U+004d</tt> | Latin capital letter M                 | <tt>0x4d</tt>      |
/// | N     | <tt>U+004e</tt> | Latin capital letter N                 | <tt>0x4e</tt>      |
/// | O     | <tt>U+004f</tt> | Latin capital letter O                 | <tt>0x4f</tt>      |
/// | P     | <tt>U+0050</tt> | Latin capital letter P                 | <tt>0x50</tt>      |
/// | Q     | <tt>U+0051</tt> | Latin capital letter Q                 | <tt>0x51</tt>      |
/// | R     | <tt>U+0052</tt> | Latin capital letter R                 | <tt>0x52</tt>      |
/// | S     | <tt>U+0053</tt> | Latin capital letter S                 | <tt>0x53</tt>      |
/// | T     | <tt>U+0054</tt> | Latin capital letter T                 | <tt>0x54</tt>      |
/// | U     | <tt>U+0055</tt> | Latin capital letter U                 | <tt>0x55</tt>      |
/// | V     | <tt>U+0056</tt> | Latin capital letter V                 | <tt>0x56</tt>      |
/// | W     | <tt>U+0057</tt> | Latin capital letter W                 | <tt>0x57</tt>      |
/// | X     | <tt>U+0058</tt> | Latin capital letter X                 | <tt>0x58</tt>      |
/// | Y     | <tt>U+0059</tt> | Latin capital letter Y                 | <tt>0x59</tt>      |
/// | Z     | <tt>U+005a</tt> | Latin capital letter Z                 | <tt>0x5a</tt>      |
/// | a     | <tt>U+0061</tt> | Latin small letter A                   | <tt>0x61</tt>      |
/// | b     | <tt>U+0062</tt> | Latin small letter B                   | <tt>0x62</tt>      |
/// | c     | <tt>U+0063</tt> | Latin small letter C                   | <tt>0x63</tt>      |
/// | d     | <tt>U+0064</tt> | Latin small letter D                   | <tt>0x64</tt>      |
/// | e     | <tt>U+0065</tt> | Latin small letter E                   | <tt>0x65</tt>      |
/// | f     | <tt>U+0066</tt> | Latin small letter F                   | <tt>0x66</tt>      |
/// | g     | <tt>U+0067</tt> | Latin small letter G                   | <tt>0x67</tt>      |
/// | h     | <tt>U+0068</tt> | Latin small letter H                   | <tt>0x68</tt>      |
/// | i     | <tt>U+0069</tt> | Latin small letter I                   | <tt>0x69</tt>      |
/// | j     | <tt>U+006a</tt> | Latin small letter J                   | <tt>0x6a</tt>      |
/// | k     | <tt>U+006b</tt> | Latin small letter K                   | <tt>0x6b</tt>      |
/// | l     | <tt>U+006c</tt> | Latin small letter L                   | <tt>0x6c</tt>      |
/// | m     | <tt>U+006d</tt> | Latin small letter M                   | <tt>0x6d</tt>      |
/// | n     | <tt>U+006e</tt> | Latin small letter N                   | <tt>0x6e</tt>      |
/// | o     | <tt>U+006f</tt> | Latin small letter O                   | <tt>0x6f</tt>      |
/// | p     | <tt>U+0070</tt> | Latin small letter P                   | <tt>0x70</tt>      |
/// | q     | <tt>U+0071</tt> | Latin small letter Q                   | <tt>0x71</tt>      |
/// | r     | <tt>U+0072</tt> | Latin small letter R                   | <tt>0x72</tt>      |
/// | s     | <tt>U+0073</tt> | Latin small letter S                   | <tt>0x73</tt>      |
/// | t     | <tt>U+0074</tt> | Latin small letter T                   | <tt>0x74</tt>      |
/// | u     | <tt>U+0075</tt> | Latin small letter U                   | <tt>0x75</tt>      |
/// | v     | <tt>U+0076</tt> | Latin small letter V                   | <tt>0x76</tt>      |
/// | w     | <tt>U+0077</tt> | Latin small letter W                   | <tt>0x77</tt>      |
/// | x     | <tt>U+0078</tt> | Latin small letter X                   | <tt>0x78</tt>      |
/// | y     | <tt>U+0079</tt> | Latin small letter Y                   | <tt>0x79</tt>      |
/// | z     | <tt>U+007a</tt> | Latin small letter Z                   | <tt>0x7a</tt>      |
/// | ª     | <tt>U+00aa</tt> | Feminine ordinal indicator             | <tt>0xaa</tt>      |
/// | ²     | <tt>U+00b2</tt> | Superscript two                        | <tt>0xb2</tt>      |
/// | ³     | <tt>U+00b3</tt> | Superscript three                      | <tt>0xb3</tt>      |
/// | µ     | <tt>U+00b5</tt> | Micro sign                             | <tt>0xb5</tt>      |
/// | ¹     | <tt>U+00b9</tt> | Superscript one                        | <tt>0xb9</tt>      |
/// | º     | <tt>U+00ba</tt> | Masculine ordinal indicator            | <tt>0xba</tt>      |
/// | À     | <tt>U+00c0</tt> |	Latin capital letter A with grave      | <tt>0xc0</tt>      |
/// | Á     | <tt>U+00c1</tt> |	Latin capital letter A with acute      | <tt>0xc1</tt>      |
/// | Â     | <tt>U+00c2</tt> |	Latin capital letter A with circumflex | <tt>0xc2</tt>      |
/// | Ã     | <tt>U+00c3</tt> |	Latin capital letter A with tilde      | <tt>0xc3</tt>      |
/// | Ä     | <tt>U+00c4</tt> |	Latin capital letter A with diaeresis  | <tt>0xc4</tt>      |
/// | Å     | <tt>U+00c5</tt> |	Latin capital letter A with ring above | <tt>0xc5</tt>      |
/// | Æ     | <tt>U+00c6</tt> |	Latin capital letter Æ                 | <tt>0xc6</tt>      |
/// | Ç     | <tt>U+00c7</tt> |	Latin capital letter C with cedilla    | <tt>0xc7</tt>      |
/// | È     | <tt>U+00c8</tt> |	Latin capital letter E with grave      | <tt>0xc8</tt>      |
/// | É     | <tt>U+00c9</tt> |	Latin capital letter E with acute      | <tt>0xc9</tt>      |
/// | Ê     | <tt>U+00ca</tt> |	Latin capital letter E with circumflex | <tt>0xca</tt>      |
/// | Ë     | <tt>U+00cb</tt> |	Latin capital letter E with diaeresis  | <tt>0xcb</tt>      |
/// | Ì     | <tt>U+00cc</tt> |	Latin capital letter I with grave      | <tt>0xcc</tt>      |
/// | Í     | <tt>U+00cd</tt> |	Latin capital letter I with acute      | <tt>0xcd</tt>      |
/// | Î     | <tt>U+00ce</tt> |	Latin capital letter I with circumflex | <tt>0xce</tt>      |
/// | Ï     | <tt>U+00cf</tt> |	Latin capital letter I with diaeresis  | <tt>0xcf</tt>      |
/// | Ð     | <tt>U+00d0</tt> |	Latin capital letter Eth               | <tt>0xd0</tt>      |
/// | Ñ     | <tt>U+00d1</tt> |	Latin capital letter N with tilde      | <tt>0xd1</tt>      |
/// | Ò     | <tt>U+00d2</tt> |	Latin capital letter O with grave      | <tt>0xd2</tt>      |
/// | Ó     | <tt>U+00d3</tt> |	Latin capital letter O with acute      | <tt>0xd3</tt>      |
/// | Ô     | <tt>U+00d4</tt> |	Latin capital letter O with circumflex | <tt>0xd4</tt>      |
/// | Õ     | <tt>U+00d5</tt> |	Latin capital letter O with tilde      | <tt>0xd5</tt>      |
/// | Ö     | <tt>U+00d6</tt> |	Latin capital letter O with diaeresis  | <tt>0xd6</tt>      |
/// | Ø     | <tt>U+00d8</tt> |	Latin capital letter O with stroke     | <tt>0xd8</tt>      |
/// | Ù     | <tt>U+00d9</tt> |	Latin capital letter U with grave      | <tt>0xd9</tt>      |
/// | Ú     | <tt>U+00da</tt> |	Latin capital letter U with acute      | <tt>0xda</tt>      |
/// | Û     | <tt>U+00db</tt> |	Latin capital letter U with circumflex | <tt>0xdb</tt>      |
/// | Ü     | <tt>U+00dc</tt> |	Latin capital letter U with diaeresis  | <tt>0xdc</tt>      |
/// | Ý     | <tt>U+00dd</tt> |	Latin capital letter Y with acute      | <tt>0xdd</tt>      |
/// | Þ     | <tt>U+00de</tt> |	Latin capital letter Thorn             | <tt>0xde</tt>      |
/// | ß     | <tt>U+00df</tt> |	Latin small letter sharp S             | <tt>0xdf</tt>      |
/// | à     | <tt>U+00e0</tt> |	Latin small letter A with grave        | <tt>0xe0</tt>      |
/// | á     | <tt>U+00e1</tt> |	Latin small letter A with acute        | <tt>0xe1</tt>      |
/// | â     | <tt>U+00e2</tt> |	Latin small letter A with circumflex   | <tt>0xe2</tt>      |
/// | ã     | <tt>U+00e3</tt> |	Latin small letter A with tilde        | <tt>0xe3</tt>      |
/// | ä     | <tt>U+00e4</tt> |	Latin small letter A with diaeresis    | <tt>0xe4</tt>      |
/// | å     | <tt>U+00e5</tt> |	Latin small letter A with ring above   | <tt>0xe5</tt>      |
/// | æ     | <tt>U+00e6</tt> |	Latin small letter Æ                   | <tt>0xe6</tt>      |
/// | ç     | <tt>U+00e7</tt> |	Latin small letter C with cedilla      | <tt>0xe7</tt>      |
/// | è     | <tt>U+00e8</tt> |	Latin small letter E with grave        | <tt>0xe8</tt>      |
/// | é     | <tt>U+00e9</tt> |	Latin small letter E with acute        | <tt>0xe9</tt>      |
/// | ê     | <tt>U+00ea</tt> |	Latin small letter E with circumflex   | <tt>0xea</tt>      |
/// | ë     | <tt>U+00eb</tt> |	Latin small letter E with diaeresis    | <tt>0xeb</tt>      |
/// | ì     | <tt>U+00ec</tt> |	Latin small letter I with grave        | <tt>0xec</tt>      |
/// | í     | <tt>U+00ed</tt> |	Latin small letter I with acute        | <tt>0xed</tt>      |
/// | î     | <tt>U+00ee</tt> |	Latin small letter I with circumflex   | <tt>0xed</tt>      |
/// | ï     | <tt>U+00ef</tt> |	Latin small letter I with diaeresis    | <tt>0xef</tt>      |
/// | ð     | <tt>U+00f0</tt> |	Latin small letter Eth                 | <tt>0xe0</tt>      |
/// | ñ     | <tt>U+00f1</tt> |	Latin small letter N with tilde        | <tt>0xf1</tt>      |
/// | ò     | <tt>U+00f2</tt> |	Latin small letter O with grave        | <tt>0xf2</tt>      |
/// | ó     | <tt>U+00f3</tt> |	Latin small letter O with acute        | <tt>0xf3</tt>      |
/// | ô     | <tt>U+00f4</tt> |	Latin small letter O with circumflex   | <tt>0xf4</tt>      |
/// | õ     | <tt>U+00f5</tt> |	Latin small letter O with tilde        | <tt>0xf5</tt>      |
/// | ö     | <tt>U+00f6</tt> |	Latin small letter O with diaeresis    | <tt>0xf6</tt>      |
/// | ø     | <tt>U+00f8</tt> |	Latin small letter O with stroke       | <tt>0xf8</tt>      |
/// | ù     | <tt>U+00f9</tt> |	Latin small letter U with grave        | <tt>0xff</tt>      |
/// | ú     | <tt>U+00fa</tt> |	Latin small letter U with acute        | <tt>0xfa</tt>      |
/// | û     | <tt>U+00fb</tt> |	Latin small letter U with circumflex   | <tt>0xfb</tt>      |
/// | ü     | <tt>U+00fc</tt> |	Latin small letter U with diaeresis    | <tt>0xfc</tt>      |
/// | ý     | <tt>U+00fd</tt> |	Latin small letter Y with acute        | <tt>0xfd</tt>      |
/// | þ     | <tt>U+00fe</tt> |	Latin small letter Thorn               | <tt>0xfe</tt>      |
/// | ÿ     | <tt>U+00ff</tt> |	Latin small letter Y with diaresis     | <tt>0xff</tt>      |
/// | Œ     | <tt>U+0152</tt> | Latin capital ligature OE              | <tt>0xbc</tt>      |
/// | œ     | <tt>U+0153</tt> | Latin small ligature OE                | <tt>0xbd</tt>      |
/// | Š     | <tt>U+0160</tt> | Latin capital letter S with caron      | <tt>0xa6</tt>      |
/// | š     | <tt>U+0161</tt> | Latin small letter S with caron        | <tt>0xa8</tt>      |
/// | Ÿ     | <tt>U+0178</tt> | Latin capital letter Y with diaresis   | <tt>0xbe</tt>      |
/// | Ž     | <tt>U+017d</tt> | Latin capital letter Z with caron      | <tt>0xb4</tt>      |
/// | ž     | <tt>U+017e</tt> | Latin small letter Z with caron        | <tt>0xb8</tt>      |
#[derive(Debug, Clone)]
pub struct WordData {
    /// Stores one or more word characters
    pub text: String,
}

impl From<&str> for WordData {
    fn from(text: &str) -> Self {
        Self {
	    text: text.to_string(),
        }
    }
}

impl From<String> for WordData {
    fn from(text: String) -> Self {
        Self {
	    text: text,
        }
    }
}

impl From<&str> for Token<PunctData> {
    fn from(s: &str) -> Self {
        Self {
            data: PunctData {
                text: s.to_string(),
            },
            dpy: Default::default(),
            frm: match s {
                r"." | r"?" | r"!" => {
                    FormatFlags::FS | FormatFlags::EOS
                },
                r":" => {
                    FormatFlags::FS
                },
                _ => {
                    Default::default()
                }
            },
        }
    }
}

impl From<usize> for Token<SpaceData> {
    fn from(n: usize) -> Self {
        Self {
            data: SpaceData {
                text: repeat(' ').take(n).collect::<String>(),
            },
            dpy: Default::default(),
            frm: FormatFlags::DLB | FormatFlags::DOB,
        }
    }
}

impl From<&str> for Token<SymbolData> {
    fn from(s: &str) -> Self {
        Self {
            data: SymbolData {
                text: s.to_string(),
            },
            dpy: Default::default(),
            frm: Default::default(),
        }
    }
}

impl From<&str> for Token<WordData> {
    fn from(w: &str) -> Self {
        Self {
            data: WordData {
                text: w.to_string(),
            },
            dpy: Default::default(),
            frm: Default::default(),
        }
    }
}
