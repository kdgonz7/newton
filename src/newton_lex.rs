//! # Newton Lexer
//!
//! This file contains the source code for the `.newton` language.
//!
//! Newton is an (experimentally) extensible programming language, with features designed to
//! allow for extending the language.
//!
//! ```ignore
//! new hello_world {
//!     conditions {
//!         any
//!         %override
//!     }
//!
//!     logic {
//!         collect as $
//!         for $ as var {
//!             ::stdout write_newline var
//!         }
//!     }
//! }
//! ```
//!

/// # Span
///
/// A span of code. These are attached to tokens for error reporting
///
/// ```
/// let span = Span::new(5, 10);
/// let str = "hello world";
///
/// assert_eq!(span.slice_and_dice(&str), "world");
/// ```
///
#[derive(Debug, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the length of the span
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// check if the span is empty, i.e. if the length is 0
    /// 
    /// this can ALSO mean that the span is a single character,
    /// so `is_empty` is a bit of a misnomer
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Peeks into a string and returns a separate string,
    /// with the content of the span
    ///
    /// ```
    /// let span = Span::new(5, 10);
    /// let str = "hello world";
    /// let slice = span.slice_and_dice(&str);
    ///
    /// assert_eq!(slice, "world");
    /// ```
    pub fn slice_and_dice(&self, string: &String) -> String {
        string[self.start..self.end].to_owned()
    }

    /// check if the span is erroneous
    ///
    /// ```
    /// let span = Span::new(5, 10);
    ///
    /// assert_eq!(span.perfect(), true);
    ///
    /// span.start = 10;
    /// span.end = 51;
    ///
    /// assert_eq!(span.perfect(), false);
    /// ```
    pub fn perfect(&self) -> bool {
        self.start <= self.end
    }

    /// check if the span is backward, i.e. if the start is greater than the end
    pub fn backward(&self) -> bool {
        self.start > self.end
    }

    /// if the span is forward.
    ///
    /// This means that the start has a destination to get to.
    pub fn forward(&self) -> bool {
        self.start < self.end
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Span: ({}, {})", self.start, self.end)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Ident,           // abc
    ReservedKeyword, // new, conditions, logic
    String,          // "abc"
    Number,          // 123
    OpenParen,       // '('
    CloseParen,      // ')'
    OpenBrace,       // '{'
    CloseBrace,      // '}'
    MemberAccess,    // '::'
    Colon,           // ':'
    SemiColon,       // ';'
    Comma,           // ','
    Dot,             // '.'
    Equal,           // '='
    Greater,         // '>'
    Less,            // '<'
    Plus,            // '+'
    Minus,           // '-'
    Multiply,        // '*'
    Divide,          // '/'
    Modulo,          // '%'
}

/// # Token
///
/// A token is essentially a compile-time optimization to allow for the parser to have an
/// easier time digesting the source code.
///
/// ```ignore
/// print "hello world"
/// ```
///
/// is equivalent to
///
/// ```ignore
/// print           :    Ident
/// "hello world"   :    String
/// ```
///
/// ```ignore
/// ::stdout write_newline var
/// ```
///
/// is equivalent to
///
/// ```ignore
/// ::            :    MemberAccess
/// stdout        :    Ident
/// write_newline :    Ident
/// var           :    Ident
/// ```
#[derive(Debug, PartialEq /* Clone */)]
pub struct Token {
    pub ty: Type,     // the token type
    pub body: String, // the embodiment of the token
    pub span: Span,   // the span of the token
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Ident => write!(f, "Ident"),
            Type::String => write!(f, "String"),
            Type::Number => write!(f, "Number"),
            Type::OpenParen => write!(f, "OpenParen"),
            Type::CloseParen => write!(f, "CloseParen"),
            Type::OpenBrace => write!(f, "OpenBrace"),
            Type::CloseBrace => write!(f, "CloseBrace"),
            Type::Colon => write!(f, "Colon"),
            Type::SemiColon => write!(f, "SemiColon"),
            Type::Comma => write!(f, "Comma"),
            Type::Dot => write!(f, "Dot"),
            Type::Equal => write!(f, "Equal"),
            Type::Greater => write!(f, "Greater"),
            Type::Less => write!(f, "Less"),
            Type::Plus => write!(f, "Plus"),
            Type::Minus => write!(f, "Minus"),
            Type::Multiply => write!(f, "Multiply"),
            Type::Divide => write!(f, "Divide"),
            Type::Modulo => write!(f, "Modulo"),
            Type::MemberAccess => write!(f, "MemberAccess"),
            Type::ReservedKeyword => write!(f, "ReservedKeyword"),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token: ({}, {})", self.ty, self.body)
    }
}

/// # Lexer
///
/// This handles the large bit of the compiling process.
#[derive(Debug, PartialEq, Clone)]
pub struct Lexer {
    pub buffer: String, // the source code
    pub pos: isize,     // the current position in the source code
}

impl Lexer {
    pub fn new(buffer: String) -> Self {
        Self { buffer, pos: -1 }
    }

    pub fn cur(&self) -> Option<char> {
        self.buffer.chars().nth(self.pos as usize)
    }

    pub fn next(&mut self) -> Option<char> {
        self.pos += 1;
        self.cur()
    }

    pub fn peek(&self) -> Option<char> {
        self.buffer.chars().nth((self.pos + 1) as usize)
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    /// turns the lexer's input stream into a list of tokens
    /// 
    /// Each token contains location information, specially for the parser to be able to
    /// find and report errors in the source code.
    /// 
    /// Still unfinished, as there are plans to include diagnostics in the error reporting,
    /// instead of panicking.
    pub fn lexeme(&mut self) -> Vec<Option<Token>> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.next() {
            if ch.is_whitespace() {
                continue;
            }

            match ch {
                'a'..='z' | 'A'..='Z' | '_' => {
                    let identifier = self.digest_ident();

                    tokens.push(identifier);
                }

                '\"' => {
                    let literal_sub = self.digest_literal();

                    tokens.push(literal_sub);
                }

                _ if (ch.is_numeric()) => {
                    let number = self.digest_number();

                    if number.is_none() {
                        panic!("weird token in number"); /* again, need diagnostics tf is this */
                    }

                    tokens.push(number);
                }

                '(' => {
                    tokens.push(Some(Token {
                        ty: Type::OpenParen,
                        body: ch.to_string(),
                        span: Span::new(self.pos as usize, self.pos as usize),
                    }));
                }

                ')' => {
                    tokens.push(Some(Token {
                        ty: Type::CloseParen,
                        body: ch.to_string(),
                        span: Span::new(self.pos as usize, self.pos as usize),
                    }));
                }

                '{' => {
                    tokens.push(Some(Token {
                        ty: Type::OpenBrace,
                        body: ch.to_string(),
                        span: Span::new(self.pos as usize, self.pos as usize),
                    }));
                }

                '}' => {
                    tokens.push(Some(Token {
                        ty: Type::CloseBrace,
                        body: ch.to_string(),
                        span: Span::new(self.pos as usize, self.pos as usize),
                    }));
                }

                ':' => {
                    let is_access = self.digest_access();
                    let access_id = self.digest_ident();

                    // if we have an access token
                    // we can now push it to the token array
                    if is_access.is_none() == false {
                        tokens.push(is_access);
                        tokens.push(access_id);
                    }
                }

                ';' => {
                    self.digest_comment();
                }

                /* ignore it otherwise */
                _ => {
                    panic!("weird token");
                }
            }
        }

        tokens
    }

    pub fn digest_comment(&mut self) {
        while let Some(ch) = self.cur() {
            if ch == '\n' {
                break;
            }

            self.advance();
        }
    }

    pub fn digest_ident(&mut self) -> Option<Token> {
        let mut ident = String::new();
        let start = self.pos;

        while let Some(ch) = self.cur() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
            } else {
                break;
            }

            self.advance(); // advances without returning
        }

        Some(Token {
            // see if it's a reserved keyword
            ty: match ident.as_str() {
                "new" => Type::ReservedKeyword,
                "conditions" => Type::ReservedKeyword,
                "logic" => Type::ReservedKeyword,
                _ => Type::Ident,
            },
            body: ident,
            span: Span::new(start as usize, self.pos as usize),
        })
    }

    /// Digests "abc"
    /// Tries to find the end quote,
    pub fn digest_literal(&mut self) -> Option<Token> {
        let mut literal = String::new();
        let start = self.pos;

        let mut escaped = false;

        literal.push('\"');

        // this revising is the result
        // of some very overestimated effort.
        //
        // from author ~ fixed now :)
        while let Some(ch) = self.next() {
            if ch == '\"' && escaped == false {
                // if char is the end quote
                self.pos += 1; // move past the end quote

                literal.push('\"');

                return Some(Token {
                    ty: Type::String,
                    body: literal,
                    span: Span::new(start as usize, self.pos as usize),
                });
            } else if ch == '\\' && escaped == false {
                escaped = true;
            } else {
                /* todo: probably add more escape sequencies. this is a toy language so i'm not too stressed about them lol */
                match escaped {
                    true => {
                        match ch {
                            'n' => {
                                literal.push('\n');
                            }
                            _ => {
                                literal.push(ch);
                            }
                        }
                        escaped = false;
                    }
                    false => {
                        literal.push(ch);
                    }
                }
            }
        }

        panic!("string was never found. he never found his buddy");
    }

    /// # Numbers
    ///
    /// `.newton` has very simple number support.
    ///
    /// All numbers are parsed as floats, but can be generally interpreted as an integer.
    pub fn digest_number(&mut self) -> Option<Token> {
        let mut number = String::new();
        let start = self.pos;

        while let Some(ch) = self.cur() {
            match ch {
                '0'..='9' | '.' | '_' => {
                    number.push(ch);
                }

                _ => {
                    panic!("weird token in number"); /* __todo__ implement diagnostics */
                }
            }

            self.advance(); // advances without returning
        }

        Some(Token {
            ty: Type::Number,
            body: number,
            span: Span::new(start as usize, self.pos as usize),
        })
    }

    pub fn digest_access(&mut self) -> Option<Token> {
        let start = self.pos;

        let should_be = self.next();

        if should_be == Some(':') {
            // this is a member access
            if self.peek().is_some() && self.next().unwrap().is_alphabetic() {
                return Some(Token {
                    ty: Type::MemberAccess,
                    body: String::from("::"),
                    span: Span::new(start as usize, self.pos as usize),
                });
            }
        } else {
            panic!("weird token, member access expects a second ':'");
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// test the span length
    #[test]
    pub fn test_simple_span() {
        let span = Span::new(5, 10);
        assert_eq!(span.len(), 5);
    }

    /// test the span peeking feature
    #[test]
    pub fn test_span_peek() {
        let span = Span::new(6, 11);
        let str = "hello world";
        let slice = span.slice_and_dice(&str.to_string());

        assert_eq!(slice, "world");
    }

    #[test]
    pub fn test_span_perfect() {
        let mut span = Span::new(5, 10);

        assert_eq!(span.forward(), true);

        span.start = 50;
        span.end = 1; // backward span?

        assert_eq!(span.forward(), false);
    }

    #[test]
    pub fn test_span_backward() {
        let mut span = Span::new(5, 10);

        assert_eq!(span.backward(), false);

        span.start = 50;
        span.end = 1; // backward span?

        assert_eq!(span.backward(), true);
    }

    #[test]
    pub fn test_lex() {
        let mut lexer =
            Lexer::new("; writes\n; basically that's what it does\n\t; so ya\n::write\nnew struct { }".to_string());

        dbg!(&lexer);

        let mut binding = lexer.lexeme();
        dbg!(&binding);

        assert_eq!(binding.len(), 5);

        let first_token = binding.get_mut(0).unwrap().as_mut().unwrap();

        assert_eq!(first_token.body, "hello");

        let second_token = binding.get_mut(1).unwrap().as_mut().unwrap();

        assert_eq!(second_token.body, "\"world");
    }
}
