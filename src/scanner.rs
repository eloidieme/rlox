use std::collections::HashMap;

use crate::{
    error,
    token_type::{Literal, Token, TokenType},
    ErrorReporter,
};

pub struct Scanner<'a> {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line_no: usize,
    keywords: HashMap<&'a str, TokenType>,
    error_reporter: &'a mut ErrorReporter,
}

impl<'a> Scanner<'a> {
    pub fn new(source: String, error_reporter: &'a mut ErrorReporter) -> Self {
        let mut keywords: HashMap<&'a str, TokenType> = HashMap::new();

        keywords.insert("and", TokenType::And);
        keywords.insert("class", TokenType::Class);
        keywords.insert("else", TokenType::Else);
        keywords.insert("false", TokenType::False);
        keywords.insert("for", TokenType::For);
        keywords.insert("fun", TokenType::Fun);
        keywords.insert("if", TokenType::If);
        keywords.insert("nil", TokenType::Nil);
        keywords.insert("or", TokenType::Or);
        keywords.insert("print", TokenType::Print);
        keywords.insert("return", TokenType::Return);
        keywords.insert("super", TokenType::Super);
        keywords.insert("this", TokenType::This);
        keywords.insert("true", TokenType::True);
        keywords.insert("var", TokenType::Var);
        keywords.insert("while", TokenType::While);

        Scanner {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line_no: 1,
            keywords,
            error_reporter,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(token_type, text, literal));
    }

    fn next_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line_no += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.error_reporter, self.line_no, "Unterminated string.");
            return;
        }

        // for the closing '"'
        self.advance();

        let value: String = self.source[(self.start + 1)..(self.current - 1)]
            .iter()
            .collect();
        self.add_token(TokenType::String, Some(Literal::Str(value)));
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let value: String = self.source[self.start..self.current].iter().collect();
        match value.parse::<f64>() {
            Ok(num) => self.add_token(TokenType::Number, Some(Literal::Number(num))),
            Err(_) => {
                error(
                    self.error_reporter,
                    self.line_no,
                    "Invalid numeric literal.",
                );
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = self
            .keywords
            .get(text.as_str())
            .unwrap_or(&TokenType::Identifier);

        self.add_token(token_type.clone(), None);
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let next: bool = self.next_match('=');
                self.add_token(
                    if next {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    },
                    None,
                );
            }
            '=' => {
                let next: bool = self.next_match('=');
                self.add_token(
                    if next {
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    },
                    None,
                );
            }
            '<' => {
                let next: bool = self.next_match('=');
                self.add_token(
                    if next {
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    },
                    None,
                );
            }
            '>' => {
                let next: bool = self.next_match('=');
                self.add_token(
                    if next {
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    },
                    None,
                );
            }
            '/' => {
                if self.next_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            '"' => self.string(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line_no += 1,
            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    error(self.error_reporter, self.line_no, "Unexpected character.");
                }
            }
        }
    }

    pub fn scan_tokens(&mut self) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, String::new(), None));
        &self.tokens
    }
}

