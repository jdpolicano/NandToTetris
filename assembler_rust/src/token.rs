#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Text,        // any text that is not a special token
    Comment,     // a full line of a commnet
    CloseParens, // ')'
    OpenParens,  // '('
    Eq,          // '=' for equality
    Comp,        // ';' for a comparison
    Address,     // '@'
    Newline,     // '\n'
    WhiteSpace, // ' ' or '\t' see https://doc.rust-lang.org/std/primitive.char.html#method.is_whitespace
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub t: TokenType,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(t: TokenType, start: usize, end: usize) -> Self {
        Self { t, start, end }
    }

    pub fn get_type(&self) -> TokenType {
        self.t.clone()
    }

    pub fn expect_type(self, t: TokenType) -> Result<Token, String> {
        if self.t == t {
            Ok(self)
        } else {
            Err(format!("token expected {:?} found {:?}", t, self.t))
        }
    }

    pub fn expect_one_of(self, ts: &[TokenType]) -> Result<Token, String> {
        if ts.contains(&self.t) {
            Ok(self)
        } else {
            Err(format!("token expected {:?} found {:?}", ts, self.t))
        }
    }
}

pub struct Tokenizer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    pub fn remaining(&self) -> &'a str {
        self.src
    }

    pub fn is_empty(&self) -> bool {
        self.current_slice().is_empty()
    }

    pub fn restore(&mut self) {
        self.pos = 0;
    }

    pub fn peek_token(&self) -> Option<Token> {
        let mut tokenizer = Tokenizer {
            src: self.src,
            pos: self.pos,
        };
        let token = tokenizer.next_token();
        token
    }

    pub fn get_slice(&self, start: usize, end: usize) -> &'a str {
        &self.src[start..end]
    }

    pub fn read_token(&self, token: Token) -> &'a str {
        self.get_slice(token.start, token.end)
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.is_empty() {
            return None;
        }

        let start = self.pos;

        if self.is_white_space() {
            self.advance();
            return Some(Token::new(TokenType::WhiteSpace, start, self.pos));
        }

        if self.is_newline() {
            self.advance();
            return Some(Token::new(TokenType::Newline, start, self.pos));
        }

        if self.is_address() {
            self.advance();
            return Some(Token::new(TokenType::Address, start, self.pos));
        }

        if self.is_comp() {
            self.advance();
            return Some(Token::new(TokenType::Comp, start, self.pos));
        }

        if self.is_eq() {
            self.advance();
            return Some(Token::new(TokenType::Eq, start, self.pos));
        }

        if self.is_open_parens() {
            self.advance();
            return Some(Token::new(TokenType::OpenParens, start, self.pos));
        }

        if self.is_close_parens() {
            self.advance();
            return Some(Token::new(TokenType::CloseParens, start, self.pos));
        }

        if self.is_comment() {
            return self.comment_token();
        }

        return self.text_token();
    }

    fn text_token(&mut self) -> Option<Token> {
        let start = self.pos;
        while !self.is_simple_token() && !self.is_comment() && !self.is_empty() {
            self.advance();
        }
        Some(Token::new(TokenType::Text, start, self.pos))
    }

    fn comment_token(&mut self) -> Option<Token> {
        let start = self.pos;
        while !self.is_newline() && !self.is_empty() {
            self.advance();
        }
        Some(Token::new(TokenType::Comment, start, self.pos))
    }

    /// simple tokens are any tokens that represent a single character.
    /// the benefit is that we know we can skip exactly one character to get to the
    /// beginning of the next token.
    fn is_simple_token(&self) -> bool {
        self.is_white_space()
            || self.is_newline()
            || self.is_address()
            || self.is_comp()
            || self.is_eq()
            || self.is_open_parens()
            || self.is_close_parens()
    }

    fn is_white_space(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == ' ' || c == '\t' || c == '\r';
        }
        false
    }

    fn is_comment(&self) -> bool {
        // check if the next two chars are "//" for a comment
        match self.slice_to(2) {
            Some("//") => true,
            _ => false,
        }
    }

    fn is_newline(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == '\n';
        }
        false
    }

    fn is_address(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == '@';
        }
        false
    }

    fn is_comp(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == ';';
        }
        false
    }

    fn is_eq(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == '=';
        }
        false
    }

    fn is_open_parens(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == '(';
        }
        false
    }

    fn is_close_parens(&self) -> bool {
        if let Some(c) = self.current_slice().chars().next() {
            return c == ')';
        }
        false
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_slice().chars().next() {
            self.pos += c.len_utf8();
        }
    }

    fn slice_to(&self, n: usize) -> Option<&str> {
        if let Some((idx, _)) = self.current_slice().char_indices().nth(n) {
            Some(&self.src[self.pos..self.pos + idx])
        } else {
            None
        }
    }

    fn current_slice(&self) -> &str {
        let pos = self.pos.min(self.src.len());
        &self.src[pos..]
    }
}

/// Tests for the Tokenizer
/// These tests are not exhaustive, but they should cover the basic functionality

#[cfg(test)]
mod test {
    use super::{TokenType, Tokenizer};
    #[test]
    fn test_tokenizer_space() {
        let src = "hello world";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        let toke3 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Text);
        assert_eq!(toke2.t, TokenType::WhiteSpace);
        assert_eq!(toke3.t, TokenType::Text);
        assert_eq!(tokenizer.read_token(toke1), "hello");
        assert_eq!(tokenizer.read_token(toke2), " ");
        assert_eq!(tokenizer.read_token(toke3), "world");
    }

    #[test]
    fn test_tokenizer_newline() {
        let src = "hello\nworld";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        let toke3 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Text);
        assert_eq!(toke2.t, TokenType::Newline);
        assert_eq!(toke3.t, TokenType::Text);
        assert_eq!(tokenizer.read_token(toke1), "hello");
        assert_eq!(tokenizer.read_token(toke2), "\n");
        assert_eq!(tokenizer.read_token(toke3), "world");
    }

    #[test]
    fn test_tokenizer_comment() {
        let src = "hello // world";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        let toke3 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Text);
        assert_eq!(toke2.t, TokenType::WhiteSpace);
        assert_eq!(toke3.t, TokenType::Comment);
        assert_eq!(tokenizer.read_token(toke1), "hello");
        assert_eq!(tokenizer.read_token(toke2), " ");
        assert_eq!(tokenizer.read_token(toke3), "// world");
    }

    #[test]
    fn test_tokenizer_comment_newline_multi() {
        let src = "hello // world\n// another comment\n";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        let toke3 = tokenizer.next_token().unwrap();
        let toke4 = tokenizer.next_token().unwrap();
        let toke5 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Text);
        assert_eq!(toke2.t, TokenType::WhiteSpace);
        assert_eq!(toke3.t, TokenType::Comment);
        assert_eq!(toke4.t, TokenType::Newline);
        assert_eq!(toke5.t, TokenType::Comment);
        assert_eq!(tokenizer.read_token(toke1), "hello");
        assert_eq!(tokenizer.read_token(toke2), " ");
        assert_eq!(tokenizer.read_token(toke3), "// world");
        assert_eq!(tokenizer.read_token(toke4), "\n");
        assert_eq!(tokenizer.read_token(toke5), "// another comment");
    }
    #[test]
    fn test_tokenizer_comment_newline() {
        let src = "hello // world\n";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        let toke3 = tokenizer.next_token().unwrap();
        let toke4 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Text);
        assert_eq!(toke2.t, TokenType::WhiteSpace);
        assert_eq!(toke3.t, TokenType::Comment);
        assert_eq!(toke4.t, TokenType::Newline);
        assert_eq!(tokenizer.read_token(toke1), "hello");
        assert_eq!(tokenizer.read_token(toke2), " ");
        assert_eq!(tokenizer.read_token(toke3), "// world");
        assert_eq!(tokenizer.read_token(toke4), "\n");
    }

    #[test]
    fn test_tokenizer_address() {
        let src = "@hello";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Address);
        assert_eq!(toke2.t, TokenType::Text);
        assert_eq!(tokenizer.read_token(toke1), "@");
        assert_eq!(tokenizer.read_token(toke2), "hello");
    }

    #[test]
    fn test_tokenizer_comp() {
        let src = "hello;";
        let mut tokenizer = Tokenizer::new(src);
        let toke1 = tokenizer.next_token().unwrap();
        let toke2 = tokenizer.next_token().unwrap();
        assert_eq!(toke1.t, TokenType::Text);
        assert_eq!(toke2.t, TokenType::Comp);
        assert_eq!(tokenizer.read_token(toke1), "hello");
        assert_eq!(tokenizer.read_token(toke2), ";");
    }
}
