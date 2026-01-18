/// トークン定義
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn,
    Let,
    Return,
    Int,     // type
    IntArray, // type
    Ident(String),
    IntLit(i64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Equals,
    Dot,
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn read_number(&mut self) -> i64 {
        let mut num = String::new();
        while let Some(c) = self.current() {
            if c.is_numeric() {
                num.push(c);
                self.advance();
            } else {
                break;
            }
        }
        num.parse().unwrap_or(0)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current() {
            None => Token::Eof,
            Some('(') => {
                self.advance();
                Token::LParen
            }
            Some(')') => {
                self.advance();
                Token::RParen
            }
            Some('{') => {
                self.advance();
                Token::LBrace
            }
            Some('}') => {
                self.advance();
                Token::RBrace
            }
            Some('[') => {
                self.advance();
                Token::LBracket
            }
            Some(']') => {
                self.advance();
                Token::RBracket
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some(';') => {
                self.advance();
                Token::Semicolon
            }
            Some(':') => {
                self.advance();
                Token::Colon
            }
            Some('=') => {
                self.advance();
                Token::Equals
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some(c) if c.is_numeric() => Token::IntLit(self.read_number()),
            Some(c) if c.is_alphabetic() || c == '_' => {
                let ident = self.read_ident();
                match ident.as_str() {
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "return" => Token::Return,
                    "int" => Token::Int,
                    _ => Token::Ident(ident),
                }
            }
            Some(c) => {
                self.advance();
                Token::Ident(c.to_string())
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
