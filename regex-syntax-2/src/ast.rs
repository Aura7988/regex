// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt;

/// An error that occurred while parsing a regular expression into an abstract
/// syntax tree.
///
/// Note that note all ASTs represents a valid regular expression. For example,
/// an AST is constructed without error for `\p{Quux}`, but `Quux` is not a
/// valid Unicode property name. That particular error is reported when
/// translating an AST to the high-level intermediate representation (`HIR`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstError {
    /// The span of this error.
    pub span: Span,
    /// The kind of error.
    pub kind: AstErrorKind,
}

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstErrorKind {
    /// An invalid escape sequence was found in a character class set.
    ClassIllegal,
    /// An opening `[` was found with no corresponding closing `]`.
    ClassUnclosed,
    /// An opening `{` was found with no corresponding closing `}`.
    CountedRepetitionUnclosed,
    /// An empty decimal number was given where one was expected.
    DecimalEmpty,
    /// An invalid decimal number was given where one was expected.
    DecimalInvalid,
    /// A bracketed hex literal was empty.
    EscapeHexEmpty,
    /// A bracketed hex literal did not correspond to a Unicode scalar value.
    EscapeHexInvalid,
    /// An invalid hexadecimal digit was found.
    EscapeHexInvalidDigit {
        /// The invalid digit (i.e., not [0-9a-zA-Z]).
        c: char,
    },
    /// EOF was found before an escape sequence was completed.
    EscapeUnexpectedEof,
    /// An unrecognized escape sequence.
    EscapeUnrecognized {
        /// The unrecognized escape.
        c: char,
    },
    /// A flag was used twice, e.g., `i-i`.
    FlagDuplicate {
        /// The duplicate flag.
        flag: char,
        /// The position of the original flag. The error position
        /// points to the duplicate flag.
        original: Span,
    },
    /// The negation operator was used twice, e.g., `-i-s`.
    FlagRepeatedNegation {
        /// The position of the original negation operator. The error position
        /// points to the duplicate negation operator.
        original: Span,
    },
    /// Expected a flag but got EOF, e.g., `(?`.
    FlagUnexpectedEof,
    /// Unrecognized flag, e.g., `a`.
    FlagUnrecognized {
        /// The unrecognized flag.
        flag: char,
    },
    /// An empty group, e.g., `()`.
    GroupEmpty,
    /// A capture group name is empty, e.g., `(?P<>abc)`.
    GroupNameEmpty,
    /// An invalid character was seen for a capture group name. This includes
    /// errors where the first character is a digit (even though subsequent
    /// characters are allowed to be digits).
    GroupNameInvalid {
        /// The invalid character. This may be a digit if it's the first
        /// character in the name.
        c: char,
    },
    /// A closing `>` could not be found for a capture group name.
    GroupNameUnexpectedEof,
    /// An unclosed group, e.g., `(ab`.
    ///
    /// The span of this error corresponds to the unclosed parenthesis.
    GroupUnclosed,
    /// An unopened group, e.g., `ab)`.
    GroupUnopened,
    /// The nest limit was exceeded. The limit stored here is the limit
    /// configured in the parser.
    NestLimitExceeded(u32),
    /// When octal support is disabled, this error is produced when an octal
    /// escape is used. The octal escape is assumed to be an invocation of
    /// a backreference, which is the common case.
    UnsupportedBackreference,
    /// When syntax similar to PCRE's look-around is used, this error is
    /// returned. Some example syntaxes that are rejected include, but are
    /// not necessarily limited to, `(?=re)`, `(?!re)`, `(?<=re)` and
    /// `(?<!re)`. Note that all of these syntaxes are otherwise invalid; this
    /// error is used to improve the user experience.
    UnsupportedLookAround,
}

impl error::Error for AstError {
    fn description(&self) -> &str {
        use self::AstErrorKind::*;
        match self.kind {
            ClassIllegal => "illegal item found in character class",
            ClassUnclosed => "unclosed character class",
            CountedRepetitionUnclosed => "unclosed counted repetition",
            DecimalEmpty => "empty decimal literal",
            DecimalInvalid => "invalid decimal literal",
            EscapeHexEmpty => "empty hexadecimal literal",
            EscapeHexInvalid => "invalid hexadecimal literal",
            EscapeHexInvalidDigit{..} => "invalid hexadecimal digit",
            EscapeUnexpectedEof => "unexpected eof (escape sequence)",
            EscapeUnrecognized{..} => "unrecognized escape sequence",
            FlagDuplicate{..} => "duplicate flag",
            FlagRepeatedNegation{..} => "repeated negation",
            FlagUnexpectedEof => "unexpected eof (flag)",
            FlagUnrecognized{..} => "unrecognized flag",
            GroupEmpty => "empty group",
            GroupNameEmpty => "empty capture group name",
            GroupNameInvalid{..} => "invalid capture group name",
            GroupNameUnexpectedEof => "unclosed capture group name",
            GroupUnclosed => "unclosed group",
            GroupUnopened => "unopened group",
            NestLimitExceeded(_) => "nest limit exceeded",
            UnsupportedBackreference => "backreferences are not supported",
            UnsupportedLookAround => "look-around is not supported",
        }
    }
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstErrorKind::*;
        match self.kind {
            ClassIllegal => {
                write!(f, "illegal item found in character class")
            }
            ClassUnclosed => {
                write!(f, "unclosed character class")
            }
            CountedRepetitionUnclosed => {
                write!(f, "unclosed counted repetition")
            }
            DecimalEmpty => {
                write!(f, "decimal literal empty")
            }
            DecimalInvalid => {
                write!(f, "decimal literal invalid")
            }
            EscapeHexEmpty => {
                write!(f, "hexadecimal literal empty")
            }
            EscapeHexInvalid => {
                write!(f, "hexadecimal literal is not a Unicode scalar value")
            }
            EscapeHexInvalidDigit { c } => {
                write!(f, "invalid hexadecimal digit '{}'", c)
            }
            EscapeUnexpectedEof => {
                write!(f, "incomplete escape sequence, \
                           reached end of pattern prematurely")
            }
            EscapeUnrecognized { c } => {
                write!(f, "unrecognized escape sequence '\\{}'", c)
            }
            FlagDuplicate { flag, .. } => {
                write!(f, "duplicate flag '{}'", flag)
            }
            FlagRepeatedNegation{..} => {
                write!(f, "flag negation operator repeated")
            }
            FlagUnexpectedEof => {
                write!(f, "expected flag but got end of regex")
            }
            FlagUnrecognized { flag } => {
                write!(f, "unrecognized flag '{}'", flag)
            }
            GroupEmpty => {
                write!(f, "empty group")
            }
            GroupNameEmpty => {
                write!(f, "empty capture group name")
            }
            GroupNameInvalid{ c } => {
                write!(f, "invalid capture group character '{}'", c)
            }
            GroupNameUnexpectedEof => {
                write!(f, "unclosed capture group name")
            }
            GroupUnclosed => {
                write!(f, "unclosed group")
            }
            GroupUnopened => {
                write!(f, "unopened group")
            }
            NestLimitExceeded(limit) => {
                write!(f, "exceed the maximum number of \
                           nested parentheses/brackets ({})", limit)
            }
            UnsupportedBackreference => {
                write!(f, "backreferences are not supported")
            }
            UnsupportedLookAround => {
                write!(f, "look-around (including look-ahead and look-behind) \
                           is not supported")
            }
        }
    }
}

/// Span represents the position information of a single AST item.
///
/// All span positions are absolute byte offsets that can be used on the
/// original regular expression that was parsed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Span {
    /// The start byte offset.
    pub start: Position,
    /// The end byte offset.
    pub end: Position,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Span({:?}, {:?})", self.start, self.end)
    }
}

/// A single position in a regular expression.
///
/// A position encodes one half of a span, and include the byte offset, line
/// number and column number.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Position {
    /// The absolute offset of this position, starting at `0` from the
    /// beginning of the regular expression pattern string.
    pub offset: usize,
    /// The line number, starting at `1`.
    pub line: usize,
    /// The approximate column number, starting at `1`.
    pub column: usize,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Position(o: {:?}, l: {:?}, c: {:?})",
            self.offset, self.line, self.column)
    }
}

impl Span {
    /// Create a new span with the given positions.
    pub fn new(start: Position, end: Position) -> Span {
        Span { start: start, end: end }
    }

    /// Create a new span using the given position as the start and end.
    pub fn splat(pos: Position) -> Span {
        Span::new(pos, pos)
    }

    /// Create a new span by replacing the starting the position with the one
    /// given.
    pub fn with_start(self, pos: Position) -> Span {
        Span { start: pos, ..self }
    }

    /// Create a new span by replacing the ending the position with the one
    /// given.
    pub fn with_end(self, pos: Position) -> Span {
        Span { end: pos, ..self }
    }
}

impl Position {
    /// Create a new position with the given information.
    ///
    /// `offset` is the absolute offset of the position, starting at `0` from
    /// the beginning of the regular expression pattern string.
    ///
    /// `line` is the line number, starting at `1`.
    ///
    /// `column` is the approximate column number, starting at `1`.
    pub fn new(offset: usize, line: usize, column: usize) -> Position {
        Position { offset: offset, line: line, column: column }
    }
}

/// An abstract syntax tree for a singular expression along with comments
/// found.
///
/// Comments are not stored in the tree itself to avoid complexity. Each
/// comment contains a span of precisely where it occurred in the original
/// regular expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstWithComments {
    /// The actual ast.
    pub ast: Ast,
    /// All comments found in the original regular expression.
    pub comments: Vec<AstComment>,
}

/// A comment from a regular expression with an associated span.
///
/// A regular expression can only contain comments when the `x` flag is
/// enabled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstComment {
    /// The span of this comment, including the beginning `#` and ending `\n`.
    pub span: Span,
    /// The comment text, starting with the first character following the `#`
    /// and ending with the last character preceding the `\n`.
    pub comment: String,
}

/// An abstract syntax tree for a single regular expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ast {
    /// An empty regex that matches everything.
    Empty(Span),
    /// A set of flags, e.g., `(?is)`.
    Flags(AstSetFlags),
    /// A single character literal, which includes escape sequences.
    Literal(AstLiteral),
    /// The "any character" class.
    Dot(Span),
    /// A single zero-width assertion.
    Assertion(AstAssertion),
    /// A single character class. This includes all forms of character classes
    /// except for `.`. e.g., `\d`, `\pN`, `[a-z]` and `[[:alpha:]]`.
    Class(AstClass),
    /// A repetition operator applied to an arbitrary regular expression.
    Repetition(AstRepetition),
    /// A grouped regular expression.
    Group(AstGroup),
    /// An alternation of regular expressions.
    Alternation(AstAlternation),
    /// A concatenation of regular expressions.
    Concat(AstConcat),
}

impl Ast {
    /// Return the span of this abstract syntax tree.
    pub fn span(&self) -> &Span {
        match *self {
            Ast::Empty(ref span) => span,
            Ast::Literal(ref x) => &x.span,
            Ast::Dot(ref span) => span,
            Ast::Class(ref x) => x.span(),
            Ast::Assertion(ref x) => &x.span,
            Ast::Repetition(ref x) => &x.span,
            Ast::Group(ref x) => &x.span,
            Ast::Flags(ref x) => &x.span,
            Ast::Alternation(ref x) => &x.span,
            Ast::Concat(ref x) => &x.span,
        }
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ast::Empty(_) => Ok(()),
            Ast::Flags(ref x) => x.fmt(f),
            Ast::Literal(ref x) => x.fmt(f),
            Ast::Dot(_) => '.'.fmt(f),
            Ast::Assertion(ref x) => x.fmt(f),
            Ast::Class(ref x) => x.fmt(f),
            Ast::Repetition(ref x) => x.fmt(f),
            Ast::Group(ref x) => x.fmt(f),
            Ast::Alternation(ref x) => x.fmt(f),
            Ast::Concat(ref x) => x.fmt(f),
        }
    }
}

/// An alternation of regular expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstAlternation {
    /// The span of this alternation.
    pub span: Span,
    /// The alternate regular expressions.
    pub asts: Vec<Ast>,
}

impl AstAlternation {
    /// Return this alternation as an AST.
    ///
    /// If this alternation contains zero ASTs, then Ast::Empty is
    /// returned. If this alternation contains exactly 1 AST, then the
    /// corresponding AST is returned. Otherwise, Ast::Alternation is returned.
    pub fn into_ast(mut self) -> Ast {
        match self.asts.len() {
            0 => Ast::Empty(self.span),
            1 => self.asts.pop().unwrap(),
            _ => Ast::Alternation(self),
        }
    }
}

impl fmt::Display for AstAlternation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for x in &self.asts {
            if !first {
                try!('|'.fmt(f));
            }
            first = false;
            try!(x.fmt(f));
        }
        Ok(())
    }
}

/// A concatenation of regular expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstConcat {
    /// The span of this concatenation.
    pub span: Span,
    /// The concatenation regular expressions.
    pub asts: Vec<Ast>,
}

impl AstConcat {
    /// Return this concatenation as an AST.
    ///
    /// If this concatenation contains zero ASTs, then Ast::Empty is
    /// returned. If this concatenation contains exactly 1 AST, then the
    /// corresponding AST is returned. Otherwise, Ast::Concat is returned.
    pub fn into_ast(mut self) -> Ast {
        match self.asts.len() {
            0 => Ast::Empty(self.span),
            1 => self.asts.pop().unwrap(),
            _ => Ast::Concat(self),
        }
    }
}

impl fmt::Display for AstConcat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in &self.asts {
            try!(x.fmt(f));
        }
        Ok(())
    }
}

/// A single literal expression.
///
/// A literal corresponds to a single Unicode scalar value. Literals may be
/// represented in their literal form, e.g., `a` or in their escaped form,
/// e.g., `\x61`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstLiteral {
    /// The span of this literal.
    pub span: Span,
    /// The kind of this literal.
    pub kind: AstLiteralKind,
    /// The Unicode scalar value corresponding to this literal.
    pub c: char,
}

impl fmt::Display for AstLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstLiteralKind::*;

        match self.kind {
            Verbatim => self.c.fmt(f),
            Special(ref x) => x.fmt(f),
            Punctuation => write!(f, r"\{}", self.c),
            Octal => write!(f, r"\{:o}", self.c as u32),
            HexFixed(AstHexLiteralKind::X) => {
                write!(f, r"\x{:02X}", self.c as u32)
            }
            HexFixed(AstHexLiteralKind::UnicodeShort) => {
                write!(f, r"\u{:04X}", self.c as u32)
            }
            HexFixed(AstHexLiteralKind::UnicodeLong) => {
                write!(f, r"\U{:08X}", self.c as u32)
            }
            HexBrace(AstHexLiteralKind::X) => {
                write!(f, r"\x{{{:X}}}", self.c as u32)
            }
            HexBrace(AstHexLiteralKind::UnicodeShort) => {
                write!(f, r"\u{{{:X}}}", self.c as u32)
            }
            HexBrace(AstHexLiteralKind::UnicodeLong) => {
                write!(f, r"\U{{{:X}}}", self.c as u32)
            }
        }
    }
}

/// The kind of a single literal expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstLiteralKind {
    /// The literal is written verbatim, e.g., `a` or `☃`.
    Verbatim,
    /// The literal is written as a specially recognized escape, e.g., `\f`
    /// or `\n`.
    Special(AstSpecialLiteralKind),
    /// The literal is written as an escape because it is punctuation, e.g.,
    /// `\*` or `\[`.
    Punctuation,
    /// The literal is written as an octal escape, e.g., `\141`.
    Octal,
    /// The literal is written as a hex code with a fixed number of digits
    /// depending on the type of the escape, e.g., `\x61` or or `\u0061` or
    /// `\U00000061`.
    HexFixed(AstHexLiteralKind),
    /// The literal is written as a hex code with a bracketed number of
    /// digits. The only restriction is that the bracketed hex code must refer
    /// to a valid Unicode scalar value.
    HexBrace(AstHexLiteralKind),
}

/// The type of a special literal.
///
/// A special literal is a special escape sequence recognized by the regex
/// parser, e.g., `\f` or `\n`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstSpecialLiteralKind {
    /// Bell, spelled `\a` (`\x07`).
    Bell,
    /// Form feed, spelled `\f` (`\x0C`).
    FormFeed,
    /// Tab, spelled `\t` (`\x09`).
    Tab,
    /// Line feed, spelled `\n` (`\x0A`).
    LineFeed,
    /// Carriage return, spelled `\r` (`\x0D`).
    CarriageReturn,
    /// Vertical tab, spelled `\v` (`\x0B`).
    VerticalTab,
    /// Space, spelled `\ ` (`\x20`). Note that this can only appear when
    /// parsing in verbose mode.
    Space,
}

impl fmt::Display for AstSpecialLiteralKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstSpecialLiteralKind::*;

        match *self {
            Bell => r"\a".fmt(f),
            FormFeed => r"\f".fmt(f),
            Tab => r"\t".fmt(f),
            LineFeed => r"\n".fmt(f),
            CarriageReturn => r"\r".fmt(f),
            VerticalTab => r"\v".fmt(f),
            Space => r"\ ".fmt(f),
        }
    }
}

/// The type of a Unicode hex literal.
///
/// Note that all variants behave the same when used with brackets. They only
/// differ when used without brackets in the number of hex digits that must
/// follow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstHexLiteralKind {
    /// A `\x` prefix. When used without brackets, this form is limited to
    /// two digits.
    X,
    /// A `\u` prefix. When used without brackets, this form is limited to
    /// four digits.
    UnicodeShort,
    /// A `\U` prefix. When used without brackets, this form is limited to
    /// eight digits.
    UnicodeLong,
}

impl AstHexLiteralKind {
    /// The number of digits that must be used with this literal form when
    /// used without brackets. When used with brackets, there is no
    /// restriction on the number of digits.
    pub fn digits(&self) -> u32 {
        match *self {
            AstHexLiteralKind::X => 2,
            AstHexLiteralKind::UnicodeShort => 4,
            AstHexLiteralKind::UnicodeLong => 8,
        }
    }
}

/// A single character class expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClass {
    /// A perl character class, e.g., `\d` or `\W`.
    Perl(AstClassPerl),
    /// A Unicode character class, e.g., `\pL` or `\p{Greek}`.
    Unicode(AstClassUnicode),
    /// A character class set, which may contain zero or more character ranges
    /// and/or zero or more nested classes. e.g., `[a-zA-Z\pL]`.
    Set(AstClassSet),
}

impl AstClass {
    /// Return the span of this character class.
    pub fn span(&self) -> &Span {
        match *self {
            AstClass::Perl(ref x) => &x.span,
            AstClass::Unicode(ref x) => &x.span,
            AstClass::Set(ref x) => &x.span,
        }
    }
}

impl fmt::Display for AstClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstClass::Perl(ref x) => x.fmt(f),
            AstClass::Unicode(ref x) => x.fmt(f),
            AstClass::Set(ref x) => x.fmt(f),
        }
    }
}

/// A Perl character class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassPerl {
    /// The span of this class.
    pub span: Span,
    /// The kind of Perl class.
    pub kind: AstClassPerlKind,
    /// Whether the class is negated or not. e.g., `\d` is not negated but
    /// `\D` is.
    pub negated: bool,
}

impl fmt::Display for AstClassPerl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            AstClassPerlKind::Digit if self.negated => r"\D".fmt(f),
            AstClassPerlKind::Digit => r"\d".fmt(f),
            AstClassPerlKind::Space if self.negated => r"\S".fmt(f),
            AstClassPerlKind::Space => r"\s".fmt(f),
            AstClassPerlKind::Word if self.negated => r"\W".fmt(f),
            AstClassPerlKind::Word => r"\w".fmt(f),
        }
    }
}

/// The available Perl character classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClassPerlKind {
    /// Decimal numbers.
    Digit,
    /// Whitespace.
    Space,
    /// Word characters.
    Word,
}

/// An ASCII character class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassAscii {
    /// The span of this class.
    pub span: Span,
    /// The kind of ASCII class.
    pub kind: AstClassAsciiKind,
    /// Whether the class is negated or not. e.g., `[[:alpha:]]` is not negated
    /// but `[[:^alpha:]]` is.
    pub negated: bool,
}

impl fmt::Display for AstClassAscii {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstClassAsciiKind::*;

        match self.kind {
            Alnum if self.negated => "[:^alnum:]".fmt(f),
            Alnum => "[:alnum:]".fmt(f),
            Alpha if self.negated => "[:^alpha:]".fmt(f),
            Alpha => "[:alpha:]".fmt(f),
            Ascii if self.negated => "[:^ascii:]".fmt(f),
            Ascii => "[:ascii:]".fmt(f),
            Blank if self.negated => "[:^blank:]".fmt(f),
            Blank => "[:blank:]".fmt(f),
            Cntrl if self.negated => "[:^cntrl:]".fmt(f),
            Cntrl => "[:cntrl:]".fmt(f),
            Digit if self.negated => "[:^digit:]".fmt(f),
            Digit => "[:digit:]".fmt(f),
            Graph if self.negated => "[:^graph:]".fmt(f),
            Graph => "[:graph:]".fmt(f),
            Lower if self.negated => "[:^lower:]".fmt(f),
            Lower => "[:lower:]".fmt(f),
            Print if self.negated => "[:^print:]".fmt(f),
            Print => "[:print:]".fmt(f),
            Punct if self.negated => "[:^punct:]".fmt(f),
            Punct => "[:punct:]".fmt(f),
            Space if self.negated => "[:^space:]".fmt(f),
            Space => "[:space:]".fmt(f),
            Upper if self.negated => "[:^upper:]".fmt(f),
            Upper => "[:upper:]".fmt(f),
            Word if self.negated => "[:^word:]".fmt(f),
            Word => "[:word:]".fmt(f),
            Xdigit if self.negated => "[:^xdigit:]".fmt(f),
            Xdigit => "[:xdigit:]".fmt(f),
        }
    }
}

/// The available ASCII character classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClassAsciiKind {
    /// `[0-9A-Za-z]`
    Alnum,
    /// `[A-Za-z]`
    Alpha,
    /// `[\x00-\x7F]`
    Ascii,
    /// `[ \t]`
    Blank,
    /// `[\x00-\x1F\x7F]`
    Cntrl,
    /// `[0-9]`
    Digit,
    /// `[!-~]`
    Graph,
    /// `[a-z]`
    Lower,
    /// `[ -~]`
    Print,
    /// `[!-/:-@\[-`{-~]`
    Punct,
    /// `[\t\n\v\f\r ]`
    Space,
    /// `[A-Z]`
    Upper,
    /// `[0-9A-Za-z_]`
    Word,
    /// `[0-9A-Fa-f]`
    Xdigit,
}

impl AstClassAsciiKind {
    /// Return the corresponding AstClassAsciiKind variant for the given name.
    ///
    /// The name given should correspond to the lowercase version of the
    /// variant name. e.g., `cntrl` is the name for `AstClassAsciiKind::Cntrl`.
    ///
    /// If no variant with the corresponding name exists, then `None` is
    /// returned.
    pub fn from_name(name: &str) -> Option<AstClassAsciiKind> {
        use self::AstClassAsciiKind::*;
        match name {
            "alnum" => Some(Alnum),
            "alpha" => Some(Alpha),
            "ascii" => Some(Ascii),
            "blank" => Some(Blank),
            "cntrl" => Some(Cntrl),
            "digit" => Some(Digit),
            "graph" => Some(Graph),
            "lower" => Some(Lower),
            "print" => Some(Print),
            "punct" => Some(Punct),
            "space" => Some(Space),
            "upper" => Some(Upper),
            "word" => Some(Word),
            "xdigit" => Some(Xdigit),
            _ => None,
        }
    }
}

/// A Unicode character class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassUnicode {
    /// The span of this class.
    pub span: Span,
    /// Whether this class is negated or not.
    ///
    /// Note: be careful when using this attribute. This specifically refers
    /// to whether the class is written as `\p` or `\P`, where the former
    /// is `negated = true`. However, it also possible to write something like
    /// `\P{scx!=Katakana}` which is actually equivalent to
    /// `\p{scx=Katakana}` and is therefore not actually negated even though
    /// `negated = true` here. To test whether this class is truly negated
    /// or not, use the `is_negated` method.
    pub negated: bool,
    /// The kind of Unicode class.
    pub kind: AstClassUnicodeKind,
}

impl AstClassUnicode {
    /// Returns true if this class has been negated.
    ///
    /// Note that this takes the Unicode op into account, if it's present.
    /// e.g., `is_negated` for `\P{scx!=Katakana}` will return `false`.
    pub fn is_negated(&self) -> bool {
        match self.kind {
            AstClassUnicodeKind::NamedValue {
                op: AstClassUnicodeOpKind::NotEqual, ..
            } => !self.negated,
            _ => self.negated,
        }
    }
}

impl fmt::Display for AstClassUnicode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.negated {
            write!(f, r"\P{}", self.kind)
        } else {
            write!(f, r"\p{}", self.kind)
        }
    }
}

/// The available forms of Unicode character classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClassUnicodeKind {
    /// A one letter abbreviated class, e.g., `\pN`.
    OneLetter(char),
    /// A binary property, general category or script. The string may be
    /// empty.
    Named(String),
    /// A property name and an associated value.
    NamedValue {
        /// The type of Unicode op used to associate `name` with `value`.
        op: AstClassUnicodeOpKind,
        /// The property name (which may be empty).
        name: String,
        /// The property value (which may be empty).
        value: String,
    },
}

impl fmt::Display for AstClassUnicodeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstClassUnicodeKind::*;

        match *self {
            OneLetter(c) => c.fmt(f),
            Named(ref x) => write!(f, "{{{}}}", x),
            NamedValue { ref op, ref name, ref value } => {
                write!(f, "{{{}{}{}}}", name, op, value)
            }
        }
    }
}

/// The type of op used in a Unicode character class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClassUnicodeOpKind {
    /// A property set to a specific value, e.g., `\p{scx=Katakana}`.
    Equal,
    /// A property set to a specific value using a colon, e.g.,
    /// `\p{scx:Katakana}`.
    Colon,
    /// A property that isn't a particular value, e.g., `\p{scx!=Katakana}`.
    NotEqual,
}

impl AstClassUnicodeOpKind {
    /// Whether the op is an equality op or not.
    pub fn is_equal(&self) -> bool {
        match *self {
            AstClassUnicodeOpKind::Equal|AstClassUnicodeOpKind::Colon => true,
            _ => false,
        }
    }
}

impl fmt::Display for AstClassUnicodeOpKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstClassUnicodeOpKind::*;

        match *self {
            Equal => '='.fmt(f),
            Colon => ':'.fmt(f),
            NotEqual => "!=".fmt(f),
        }
    }
}

/// A Unicode character class set, e.g., `[a-z0-9]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassSet {
    /// The span of this class.
    pub span: Span,
    /// Whether this class is negated or not. e.g., `[a]` is not negated but
    /// `[^a]` is.
    pub negated: bool,
    /// The top-level op of this set.
    pub op: AstClassSetOp,
}

impl fmt::Display for AstClassSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.negated {
            write!(f, r"[^{}]", self.op)
        } else {
            write!(f, r"[{}]", self.op)
        }
    }
}

/// An operation inside a character class set.
///
/// An operation is either a union of many things, or a binary operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClassSetOp {
    /// A union of items in a class. A union may contain a single item.
    Union(AstClassSetUnion),
    /// A single binary operation (i.e., &&, -- or ~~).
    BinaryOp(AstClassSetBinaryOp),
}

impl AstClassSetOp {
    /// Return the span of this character class set operation.
    pub fn span(&self) -> &Span {
        match *self {
            AstClassSetOp::Union(ref x) => &x.span,
            AstClassSetOp::BinaryOp(ref x) => &x.span,
        }
    }
}

impl fmt::Display for AstClassSetOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstClassSetOp::Union(ref x) => x.fmt(f),
            AstClassSetOp::BinaryOp(ref x) => x.fmt(f),
        }
    }
}

/// A union of items inside a character class set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassSetUnion {
    /// The span of the items in this operation. e.g., the `a-z0-9` in
    /// `[^a-z0-9]`
    pub span: Span,
    /// The sequence of items that make up this union.
    pub items: Vec<AstClassSetItem>,
}

impl AstClassSetUnion {
    /// Push a new item in this union.
    ///
    /// The ending position of this union's span is updated to the ending
    /// position of the span of the item given. If the union is empty, then
    /// the starting position of this union is set to the starting position
    /// of this item.
    ///
    /// In other words, if you only use this method to add items to a union
    /// and you set the spans on each item correctly, then you should never
    /// need to adjust the span of the union directly.
    pub fn push(&mut self, item: AstClassSetItem) {
        if self.items.is_empty() {
            self.span.start = item.span().start;
        }
        self.span.end = item.span().end;
        self.items.push(item);
    }
}

impl fmt::Display for AstClassSetUnion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in &self.items {
            try!(x.fmt(f));
        }
        Ok(())
    }
}

/// A single component of a character class set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstClassSetItem {
    /// A single literal.
    Literal(AstLiteral),
    /// A range between two literals.
    Range(AstClassSetRange),
    /// An ASCII character class, e.g., `[:alnum:]` or `[:punct:]`.
    Ascii(AstClassAscii),
    /// A nested character class.
    Class(Box<AstClass>),
}

impl AstClassSetItem {
    /// Return the span of this character class set item.
    pub fn span(&self) -> &Span {
        match *self {
            AstClassSetItem::Literal(ref x) => &x.span,
            AstClassSetItem::Range(ref x) => &x.span,
            AstClassSetItem::Ascii(ref x) => &x.span,
            AstClassSetItem::Class(ref x) => x.span(),
        }
    }
}

impl fmt::Display for AstClassSetItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstClassSetItem::Literal(ref x) => x.fmt(f),
            AstClassSetItem::Range(ref x) => x.fmt(f),
            AstClassSetItem::Ascii(ref x) => x.fmt(f),
            AstClassSetItem::Class(ref x) => x.fmt(f),
        }
    }
}

/// A single character class range in a set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassSetRange {
    /// The span of this range.
    pub span: Span,
    /// The start of this range.
    pub start: AstLiteral,
    /// The end of this range.
    pub end: AstLiteral,
}

impl fmt::Display for AstClassSetRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// A Unicode character class set operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstClassSetBinaryOp {
    /// The span of this operation. e.g., the `a-z--[h-p]` in `[a-z--h-p]`.
    pub span: Span,
    /// The type of this set operation.
    pub kind: AstClassSetBinaryOpKind,
    /// The left hand side of the operation.
    pub lhs: Box<AstClassSetOp>,
    /// The right hand side of the operation.
    pub rhs: Box<AstClassSetOp>,
}

impl fmt::Display for AstClassSetBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}", self.lhs, self.kind, self.rhs)
    }
}

/// The type of a Unicode character class set operation.
///
/// Note that this doesn't explicitly represent union since there is no
/// explicit union operator. Concatenation inside a character class corresponds
/// to the union operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AstClassSetBinaryOpKind {
    /// The intersection of two sets, e.g., `\pN&&[a-z]`.
    Intersection,
    /// The difference of two sets, e.g., `\pN--[0-9]`.
    Difference,
    /// The symmetric difference of two sets. The symmetric difference is the
    /// set of elements belonging to one but not both sets.
    /// e.g., `[\pL~~[:ascii:]]`.
    SymmetricDifference,
}

impl fmt::Display for AstClassSetBinaryOpKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstClassSetBinaryOpKind::Intersection => "&&".fmt(f),
            AstClassSetBinaryOpKind::Difference => "--".fmt(f),
            AstClassSetBinaryOpKind::SymmetricDifference => "~~".fmt(f),
        }
    }
}

/// A single zero-width assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstAssertion {
    /// The span of this assertion.
    pub span: Span,
    /// The assertion kind, e.g., `\b` or `^`.
    pub kind: AstAssertionKind,
}

impl fmt::Display for AstAssertion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

/// An assertion kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstAssertionKind {
    /// `^`
    StartLine,
    /// `$`
    EndLine,
    /// `\A`
    StartText,
    /// `\z`
    EndText,
    /// `\b`
    WordBoundary,
    /// `\B`
    NotWordBoundary,
}

impl fmt::Display for AstAssertionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstAssertionKind::StartLine => '^'.fmt(f),
            AstAssertionKind::EndLine => '$'.fmt(f),
            AstAssertionKind::StartText => r"\A".fmt(f),
            AstAssertionKind::EndText => r"\z".fmt(f),
            AstAssertionKind::WordBoundary => r"\b".fmt(f),
            AstAssertionKind::NotWordBoundary => r"\B".fmt(f),
        }
    }
}

/// A repetition operation applied to a regular expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstRepetition {
    /// The span of this operation.
    pub span: Span,
    /// The actual operation.
    pub op: AstRepetitionOp,
    /// Whether this operation was applied greedily or not.
    pub greedy: bool,
    /// The regular expression under repetition.
    pub ast: Box<Ast>,
}

impl fmt::Display for AstRepetition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op.kind {
            AstRepetitionKind::ZeroOrOne => {
                if self.greedy {
                    write!(f, "{}?", self.ast)
                } else {
                    write!(f, "{}??", self.ast)
                }
            }
            AstRepetitionKind::ZeroOrMore => {
                if self.greedy {
                    write!(f, "{}*", self.ast)
                } else {
                    write!(f, "{}*?", self.ast)
                }
            }
            AstRepetitionKind::OneOrMore => {
                if self.greedy {
                    write!(f, "{}+", self.ast)
                } else {
                    write!(f, "{}+?", self.ast)
                }
            }
            AstRepetitionKind::Range(ref x) => {
                if self.greedy {
                    write!(f, "{}{}", self.ast, x)
                } else {
                    write!(f, "{}{}?", self.ast, x)
                }
            }
        }
    }
}

/// The repetition operator itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstRepetitionOp {
    /// The span of this operator. This includes things like `+`, `*?` and
    /// `{m,n}`.
    pub span: Span,
    /// The type of operation.
    pub kind: AstRepetitionKind,
}

/// The kind of a repetition operator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstRepetitionKind {
    /// `?`
    ZeroOrOne,
    /// `*`
    ZeroOrMore,
    /// `+`
    OneOrMore,
    /// `{m,n}`
    Range(AstRepetitionRange),
}

/// A range repetition operator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstRepetitionRange {
    /// `{m}`
    Exactly(u32),
    /// `{m,}`
    AtLeast(u32),
    /// `{m,n}`
    Bounded(u32, u32),
}

impl fmt::Display for AstRepetitionRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstRepetitionRange::Exactly(x) => write!(f, "{{{}}}", x),
            AstRepetitionRange::AtLeast(x) => write!(f, "{{{},}}", x),
            AstRepetitionRange::Bounded(x, y) => write!(f, "{{{},{}}}", x, y),
        }
    }
}

/// A grouped regular expression.
///
/// This includes both capturing and non-capturing groups. This does **not**
/// include flag-only groups like `(?is)`, but does contain any group that
/// contains a sub-expression, e.g., `(a)`, `(?P<name>a)`, `(?:a)` and
/// `(?is:a)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstGroup {
    /// The span of this group.
    pub span: Span,
    /// The kind of this group.
    pub kind: AstGroupKind,
    /// The regular expression in this group.
    pub ast: Box<Ast>,
}

impl AstGroup {
    /// If this group is non-capturing, then this returns the (possibly empty)
    /// set of flags. Otherwise, `None` is returned.
    pub fn flags(&self) -> Option<&AstFlags> {
        match self.kind {
            AstGroupKind::NonCapturing(ref flags) => Some(flags),
            _ => None,
        }
    }
}

impl fmt::Display for AstGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            AstGroupKind::CaptureIndex => {
                write!(f, "({})", self.ast)
            }
            AstGroupKind::CaptureName(ref x) => {
                write!(f, "(?P<{}>{})", x, self.ast)
            }
            AstGroupKind::NonCapturing(ref x) => {
                write!(f, "(?{}:{})", x, self.ast)
            }
        }
    }
}

/// The kind of a group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstGroupKind {
    /// `(a)`
    CaptureIndex,
    /// `(?P<name>a)`
    CaptureName(AstCaptureName),
    /// `(?:a)` and `(?i:a)`
    NonCapturing(AstFlags),
}

/// A capture name.
///
/// This corresponds to the name itself between the angle brackets in, e.g.,
/// `(?P<foo>expr)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstCaptureName {
    /// The span of this capture name.
    pub span: Span,
    /// The capture name.
    pub name: String,
}

impl fmt::Display for AstCaptureName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

/// A group of flags that is not applied to a particular regular expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstSetFlags {
    /// The span of these flags, including the grouping parentheses.
    pub span: Span,
    /// The actual sequence of flags.
    pub flags: AstFlags,
}

impl fmt::Display for AstSetFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(?{})", self.flags)
    }
}

/// A group of flags.
///
/// This corresponds only to the sequence of flags themselves, e.g., `is-u`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstFlags {
    /// The span of this group of flags.
    pub span: Span,
    /// A sequence of flag items. Each item is either a flag or a negation
    /// operator.
    pub items: Vec<AstFlagsItem>,
}

impl AstFlags {
    /// Add the given item to this sequence of flags.
    ///
    /// If the item was added successfully, then `None` is returned. If the
    /// given item is a duplicate, then `Some(i)` is returned, where
    /// `items[i].kind == item.kind`.
    pub fn add_item(&mut self, item: AstFlagsItem) -> Option<usize> {
        for (i, x) in self.items.iter().enumerate() {
            if x.kind == item.kind {
                return Some(i);
            }
        }
        self.items.push(item);
        None
    }

    /// Returns the state of the given flag in this set.
    ///
    /// If the given flag is in the set but is negated, then `Some(false)` is
    /// returned.
    ///
    /// If the given flag is in the set and is not negated, then `Some(true)`
    /// is returned.
    ///
    /// Otherwise, `None` is returned.
    pub fn flag_state(&self, flag: AstFlag) -> Option<bool> {
        let mut negated = false;
        for x in &self.items {
            match x.kind {
                AstFlagsItemKind::Negation => {
                    negated = true;
                }
                AstFlagsItemKind::Flag(ref xflag) if xflag == &flag => {
                    return Some(!negated);
                }
                _ => {}
            }
        }
        None
    }
}

impl fmt::Display for AstFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in &self.items {
            try!(item.fmt(f));
        }
        Ok(())
    }
}

/// A single item in a group of flags.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstFlagsItem {
    /// The span of this item.
    pub span: Span,
    /// The kind of this item.
    pub kind: AstFlagsItemKind,
}

impl fmt::Display for AstFlagsItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

/// The kind of an item in a group of flags.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstFlagsItemKind {
    /// A negation operator applied to all subsequent flags in the enclosing
    /// group.
    Negation,
    /// A single flag in a group.
    Flag(AstFlag),
}

impl AstFlagsItemKind {
    /// Returns true if and only if this item is a negation operator.
    pub fn is_negation(&self) -> bool {
        match *self {
            AstFlagsItemKind::Negation => true,
            _ => false,
        }
    }
}

impl fmt::Display for AstFlagsItemKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AstFlagsItemKind::Negation => "-".fmt(f),
            AstFlagsItemKind::Flag(ref flag) => flag.fmt(f),
        }
    }
}

/// A single flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AstFlag {
    /// `i`
    CaseInsensitive,
    /// `m`
    MultiLine,
    /// `s`
    DotMatchesNewLine,
    /// `U`
    SwapGreed,
    /// `u`
    Unicode,
    /// `x`
    IgnoreWhitespace,
}

impl fmt::Display for AstFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AstFlag::*;
        match *self {
            CaseInsensitive => write!(f, "i"),
            MultiLine => write!(f, "m"),
            DotMatchesNewLine => write!(f, "s"),
            SwapGreed => write!(f, "U"),
            Unicode => write!(f, "u"),
            IgnoreWhitespace => write!(f, "x"),
        }
    }
}

#[cfg(test)]
mod tests {
    use parse::ParserBuilder;

    fn roundtrip(given: &str) {
        roundtrip_with(|b| b, given);
    }

    fn roundtrip_with<F>(mut f: F, given: &str)
        where F: FnMut(&mut ParserBuilder) -> &mut ParserBuilder
    {
        let mut builder = ParserBuilder::new();
        f(&mut builder);
        let ast = builder.build(given).parse().unwrap();
        assert_eq!(format!("{}", ast), given);
    }

    #[test]
    fn print_literal() {
        roundtrip("a");
        roundtrip(r"\[");
        roundtrip_with(|b| b.octal(true), r"\141");
        roundtrip(r"\x61");
        roundtrip(r"\x7F");
        roundtrip(r"\u0061");
        roundtrip(r"\U00000061");
        roundtrip(r"\x{61}");
        roundtrip(r"\x{7F}");
        roundtrip(r"\u{61}");
        roundtrip(r"\U{61}");

        roundtrip(r"\a");
        roundtrip(r"\f");
        roundtrip(r"\t");
        roundtrip(r"\n");
        roundtrip(r"\r");
        roundtrip(r"\v");
        roundtrip(r"(?x)\ ");
    }

    #[test]
    fn print_dot() {
        roundtrip(".");
    }

    #[test]
    fn print_concat() {
        roundtrip("ab");
        roundtrip("abcde");
        roundtrip("a(bcd)ef");
    }

    #[test]
    fn print_alternation() {
        roundtrip("a|b");
        roundtrip("a|b|c|d|e");
        roundtrip("a(b|c|d)|e|f");
    }

    #[test]
    fn print_assertion() {
        roundtrip(r"^");
        roundtrip(r"$");
        roundtrip(r"\A");
        roundtrip(r"\z");
        roundtrip(r"\b");
        roundtrip(r"\B");
    }

    #[test]
    fn print_repetition() {
        roundtrip("a?");
        roundtrip("a??");
        roundtrip("a*");
        roundtrip("a*?");
        roundtrip("a+");
        roundtrip("a+?");
        roundtrip("a{5}");
        roundtrip("a{5}?");
        roundtrip("a{5,}");
        roundtrip("a{5,}?");
        roundtrip("a{5,10}");
        roundtrip("a{5,10}?");
    }

    #[test]
    fn print_flags() {
        roundtrip("(?i)");
        roundtrip("(?-i)");
        roundtrip("(?s-i)");
        roundtrip("(?-si)");
        roundtrip("(?siUmux)");
    }

    #[test]
    fn print_group() {
        roundtrip("(?i:a)");
        roundtrip("(?P<foo>a)");
        roundtrip("(a)");
    }

    #[test]
    fn print_class() {
        roundtrip(r"[abc]");
        roundtrip(r"[a-z]");
        roundtrip(r"[^a-z]");
        roundtrip(r"[a-z0-9]");
        roundtrip(r"[-a-z0-9]");
        roundtrip(r"[-a-z0-9]");
        roundtrip(r"[a-z0-9---]");
        roundtrip(r"[a-z&&m-n]");
        roundtrip(r"[a-z--m-n]");
        roundtrip(r"[a-z~~m-n]");
        roundtrip(r"[a-z[0-9]]");
        roundtrip(r"[a-z[^0-9]]");

        roundtrip(r"\d");
        roundtrip(r"\D");
        roundtrip(r"\s");
        roundtrip(r"\S");
        roundtrip(r"\w");
        roundtrip(r"\W");

        roundtrip(r"[[:alnum:]]");
        roundtrip(r"[[:^alnum:]]");
        roundtrip(r"[[:alpha:]]");
        roundtrip(r"[[:^alpha:]]");
        roundtrip(r"[[:ascii:]]");
        roundtrip(r"[[:^ascii:]]");
        roundtrip(r"[[:blank:]]");
        roundtrip(r"[[:^blank:]]");
        roundtrip(r"[[:cntrl:]]");
        roundtrip(r"[[:^cntrl:]]");
        roundtrip(r"[[:digit:]]");
        roundtrip(r"[[:^digit:]]");
        roundtrip(r"[[:graph:]]");
        roundtrip(r"[[:^graph:]]");
        roundtrip(r"[[:lower:]]");
        roundtrip(r"[[:^lower:]]");
        roundtrip(r"[[:print:]]");
        roundtrip(r"[[:^print:]]");
        roundtrip(r"[[:punct:]]");
        roundtrip(r"[[:^punct:]]");
        roundtrip(r"[[:space:]]");
        roundtrip(r"[[:^space:]]");
        roundtrip(r"[[:upper:]]");
        roundtrip(r"[[:^upper:]]");
        roundtrip(r"[[:word:]]");
        roundtrip(r"[[:^word:]]");
        roundtrip(r"[[:xdigit:]]");
        roundtrip(r"[[:^xdigit:]]");

        roundtrip(r"\pL");
        roundtrip(r"\PL");
        roundtrip(r"\p{L}");
        roundtrip(r"\P{L}");
        roundtrip(r"\p{X=Y}");
        roundtrip(r"\P{X=Y}");
        roundtrip(r"\p{X:Y}");
        roundtrip(r"\P{X:Y}");
        roundtrip(r"\p{X!=Y}");
        roundtrip(r"\P{X!=Y}");
    }
}
