use crate::error::{MinusError, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // Identifier
    Identifier(String),

    // Keywords
    Int,
    Float,
    Char,
    Bool,
    String,
    Void,
    True,
    False,
    If,
    Else,
    While,
    For,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Struct,
    Import,
    Result,
    Ok,
    Error,
    Fn,
    Let,
    Const,
    Print,

    // Operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Assign,      // =
    Equal,       // ==
    NotEqual,    // !=
    Less,        // <
    Greater,     // >
    LessEqual,   // <=
    GreaterEqual,// >=
    And,         // &&
    Or,          // ||
    Not,         // !
    Arrow,       // ->
    PlusAssign,  // +=
    MinusAssign, // -=
    StarAssign,  // *=
    SlashAssign, // /=

    // Delimiters
    LeftParen,   // (
    RightParen,  // )
    LeftBrace,   // {
    RightBrace,  // }
    LeftBracket, // [
    RightBracket,// ]
    Semicolon,   // ;
    Colon,       // :
    Comma,       // ,
    Dot,         // .

    // Special
    StringInterpolationStart, // $"
    InterpolationEnd,         // }
    InterpolationStart,       // {

    // EOF
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::IntLiteral(v) => write!(f, "{}", v),
            Token::FloatLiteral(v) => write!(f, "{}", v),
            Token::StringLiteral(v) => write!(f, "\"{}\"", v),
            Token::CharLiteral(v) => write!(f, "'{}'", v),
            Token::BoolLiteral(v) => write!(f, "{}", v),
            Token::Identifier(v) => write!(f, "{}", v),
            Token::Int => write!(f, "int"),
            Token::Float => write!(f, "float"),
            Token::Char => write!(f, "char"),
            Token::Bool => write!(f, "bool"),
            Token::String => write!(f, "string"),
            Token::Void => write!(f, "void"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::Switch => write!(f, "switch"),
            Token::Case => write!(f, "case"),
            Token::Default => write!(f, "default"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Return => write!(f, "return"),
            Token::Struct => write!(f, "struct"),
            Token::Import => write!(f, "import"),
            Token::Result => write!(f, "Result"),
            Token::Ok => write!(f, "ok"),
            Token::Error => write!(f, "error"),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Const => write!(f, "const"),
            Token::Print => write!(f, "Print"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Assign => write!(f, "="),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Arrow => write!(f, "->"),
            Token::PlusAssign => write!(f, "+="),
            Token::MinusAssign => write!(f, "-="),
            Token::StarAssign => write!(f, "*="),
            Token::SlashAssign => write!(f, "/="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::StringInterpolationStart => write!(f, "$\""),
            Token::InterpolationEnd => write!(f, "}}"),
            Token::InterpolationStart => write!(f, "{{"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace_and_comments();

        if self.pos >= self.input.len() {
            return Ok(Token::Eof);
        }

        let ch = self.input[self.pos];

        match ch {
            '"' => self.read_string(),
            '\'' => self.read_char(),
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),
            '(' => { self.advance(); Ok(Token::LeftParen) }
            ')' => { self.advance(); Ok(Token::RightParen) }
            '{' => { self.advance(); Ok(Token::LeftBrace) }
            '}' => { self.advance(); Ok(Token::RightBrace) }
            '[' => { self.advance(); Ok(Token::LeftBracket) }
            ']' => { self.advance(); Ok(Token::RightBracket) }
            ';' => { self.advance(); Ok(Token::Semicolon) }
            ':' => { self.advance(); Ok(Token::Colon) }
            ',' => { self.advance(); Ok(Token::Comma) }
            '.' => { self.advance(); Ok(Token::Dot) }
            '+' => self.read_plus(),
            '-' => self.read_minus(),
            '*' => self.read_star(),
            '/' => self.read_slash(),
            '%' => { self.advance(); Ok(Token::Percent) }
            '=' => self.read_equal(),
            '!' => self.read_not(),
            '<' => self.read_less(),
            '>' => self.read_greater(),
            '&' => self.read_and(),
            '|' => self.read_or(),
            _ => Err(MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: format!("Unexpected character: {}", ch),
            }),
        }
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            if self.input[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];

            if ch.is_whitespace() {
                self.advance();
            } else if ch == '#' {
                // Single line comment
                while self.pos < self.input.len() && self.input[self.pos] != '\n' {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> Result<Token> {
        self.advance(); // skip opening "
        let mut value = String::new();

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];

            if ch == '"' {
                self.advance(); // skip closing "
                return Ok(Token::StringLiteral(value));
            } else if ch == '\\' {
                self.advance();
                if self.pos < self.input.len() {
                    let escaped = match self.input[self.pos] {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '0' => '\0',
                        c => c,
                    };
                    value.push(escaped);
                    self.advance();
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(MinusError::LexerError {
            line: self.line,
            col: self.col,
            message: "Unterminated string".to_string(),
        })
    }

    fn read_char(&mut self) -> Result<Token> {
        self.advance(); // skip opening '
        
        if self.pos >= self.input.len() {
            return Err(MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: "Unterminated character literal".to_string(),
            });
        }

        let ch = if self.input[self.pos] == '\\' {
            self.advance();
            match self.input[self.pos] {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                '0' => '\0',
                c => c,
            }
        } else {
            self.input[self.pos]
        };

        self.advance();

        if self.pos < self.input.len() && self.input[self.pos] == '\'' {
            self.advance(); // skip closing '
            Ok(Token::CharLiteral(ch))
        } else {
            Err(MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: "Unterminated character literal".to_string(),
            })
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.pos;
        let mut is_float = false;

        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.advance();
        }

        if self.pos < self.input.len() && self.input[self.pos] == '.' {
            if self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                self.advance(); // skip .
                while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                    self.advance();
                }
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();

        if is_float {
            let value = num_str.parse::<f64>().map_err(|_| MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: format!("Invalid float literal: {}", num_str),
            })?;
            Ok(Token::FloatLiteral(value))
        } else {
            let value = num_str.parse::<i64>().map_err(|_| MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: format!("Invalid integer literal: {}", num_str),
            })?;
            Ok(Token::IntLiteral(value))
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token> {
        let start = self.pos;

        while self.pos < self.input.len()
            && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_')
        {
            self.advance();
        }

        let word: String = self.input[start..self.pos].iter().collect();

        let token = match word.as_str() {
            "int" => Token::Int,
            "float" => Token::Float,
            "char" => Token::Char,
            "bool" => Token::Bool,
            "string" => Token::String,
            "void" => Token::Void,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "switch" => Token::Switch,
            "case" => Token::Case,
            "default" => Token::Default,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "struct" => Token::Struct,
            "import" => Token::Import,
            "Result" => Token::Result,
            "ok" => Token::Ok,
            "error" => Token::Error,
            "fn" => Token::Fn,
            "let" => Token::Let,
            "const" => Token::Const,
            "Print" => Token::Print,
            _ => Token::Identifier(word),
        };

        Ok(token)
    }

    fn read_plus(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::PlusAssign)
        } else {
            Ok(Token::Plus)
        }
    }

    fn read_minus(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::MinusAssign)
        } else if self.peek() == Some('>') {
            self.advance();
            Ok(Token::Arrow)
        } else {
            Ok(Token::Minus)
        }
    }

    fn read_star(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::StarAssign)
        } else {
            Ok(Token::Star)
        }
    }

    fn read_slash(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::SlashAssign)
        } else {
            Ok(Token::Slash)
        }
    }

    fn read_equal(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::Equal)
        } else {
            Ok(Token::Assign)
        }
    }

    fn read_not(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::NotEqual)
        } else {
            Ok(Token::Not)
        }
    }

    fn read_less(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::LessEqual)
        } else {
            Ok(Token::Less)
        }
    }

    fn read_greater(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Ok(Token::GreaterEqual)
        } else {
            Ok(Token::Greater)
        }
    }

    fn read_and(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('&') {
            self.advance();
            Ok(Token::And)
        } else {
            Err(MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: "Unexpected character: &".to_string(),
            })
        }
    }

    fn read_or(&mut self) -> Result<Token> {
        self.advance();
        if self.peek() == Some('|') {
            self.advance();
            Ok(Token::Or)
        } else {
            Err(MinusError::LexerError {
                line: self.line,
                col: self.col,
                message: "Unexpected character: |".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "int x = 5;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![
            Token::Int,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::IntLiteral(5),
            Token::Semicolon,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_string_literal() {
        let input = r#""hello world""#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![
            Token::StringLiteral("hello world".to_string()),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_operators() {
        let input = "+ - * / % = == != < > <= >= && || ! -> += -= *= /=";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent,
            Token::Assign,
            Token::Equal,
            Token::NotEqual,
            Token::Less,
            Token::Greater,
            Token::LessEqual,
            Token::GreaterEqual,
            Token::And,
            Token::Or,
            Token::Not,
            Token::Arrow,
            Token::PlusAssign,
            Token::MinusAssign,
            Token::StarAssign,
            Token::SlashAssign,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_keywords() {
        let input = "if else while for switch case default break continue return struct import";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![
            Token::If,
            Token::Else,
            Token::While,
            Token::For,
            Token::Switch,
            Token::Case,
            Token::Default,
            Token::Break,
            Token::Continue,
            Token::Return,
            Token::Struct,
            Token::Import,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_comments() {
        let input = "# this is a comment\nint x = 5;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens, vec![
            Token::Int,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::IntLiteral(5),
            Token::Semicolon,
            Token::Eof,
        ]);
    }
}
