#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Comment,
    Text, // arbitrary text, the parser will need to give thia meaning.
    Newline,
    WhiteSpace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
}

pub struct Tokenizer<'a> {
    pub tokens: Vec<Token>,
    pub source: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut t = Self {
            tokens: Vec::new(),
            source,
            pos: 0,
        };
        t.tokenize();
        t
    }

    pub fn tokenize(&mut self) {
        while self.pos < self.source.len() {
            let start = self.pos;
            let token_type = self.get_token_type();
            self.tokens.push(Token {
                token_type,
                start,
                end: self.pos,
            });
        }
    }

    pub fn take_tokens(self) -> Vec<Token> {
        self.tokens
    }

    fn is_done(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn get_token_type(&mut self) -> TokenType {
        if self.is_comment() {
            self.advance_comment();
            TokenType::Comment
        } else if self.is_newline() {
            self.advance();
            TokenType::Newline
        } else if self.is_whitespace() {
            self.advance();
            TokenType::WhiteSpace
        } else {
            self.advance_text();
            TokenType::Text
        }
    }

    fn is_comment(&self) -> bool {
        match self.slice_n(2) {
            "//" => true,
            _ => false,
        }
    }

    fn is_newline(&self) -> bool {
        match self.current_slice().chars().next() {
            Some('\n') => true,
            _ => false,
        }
    }

    fn is_whitespace(&self) -> bool {
        match self.current_slice().chars().next() {
            Some(c) => c == ' ' || c == '\t' || c == '\r',
            _ => false,
        }
    }

    fn advance_comment(&mut self) {
        while !self.is_done() && !self.is_newline() {
            self.advance();
        }
    }

    fn advance_text(&mut self) {
        while !self.is_done() && !self.is_comment() && !self.is_newline() && !self.is_whitespace() {
            self.advance();
        }
    }

    fn slice_n(&self, n: usize) -> &str {
        if let Some((sidx, _)) = self.current_slice().char_indices().nth(n) {
            &self.source[self.pos..self.pos + sidx]
        } else {
            ""
        }
    }

    fn current_slice(&self) -> &str {
        let start = self.pos.min(self.source.len());
        &self.source[start..]
    }

    pub fn get_token_str(&self, token: &Token) -> &str {
        &self.source[token.start..token.end]
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_slice().chars().next() {
            self.pos += c.len_utf8();
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    fn get_compare_token(tokenizer: &Tokenizer, nth: usize, expected: &str) -> Token {
        let token = &tokenizer.tokens[nth];
        let actual = tokenizer.source[token.start..token.end].to_string();
        assert_eq!(actual, expected);
        token.clone()
    }

    #[test]
    fn test_tokenize_hello() {
        let source = "Hello, World!".to_string();
        let mut tokenizer = Tokenizer::new(&source);
        tokenizer.tokenize();
        let t1 = get_compare_token(&tokenizer, 0, "Hello,");
        let t2 = get_compare_token(&tokenizer, 1, " ");
        let t3 = get_compare_token(&tokenizer, 2, "World!");
        assert!(tokenizer.tokens.len() == 3);
        assert_eq!(t1.token_type, TokenType::Text);
        assert_eq!(t2.token_type, TokenType::WhiteSpace);
        assert_eq!(t3.token_type, TokenType::Text);
    }

    #[test]
    fn test_tokenize_comment() {
        let source = "// this is a comment".to_string();
        let mut tokenizer = Tokenizer::new(&source);
        tokenizer.tokenize();
        let t1 = get_compare_token(&tokenizer, 0, "// this is a comment");
        assert!(tokenizer.tokens.len() == 1);
        assert_eq!(t1.token_type, TokenType::Comment);
    }

    #[test]
    fn test_tokenize_newline() {
        let source = "Hello,\nWorld!".to_string();
        let mut tokenizer = Tokenizer::new(&source);
        tokenizer.tokenize();
        let t1 = get_compare_token(&tokenizer, 0, "Hello,");
        let t2 = get_compare_token(&tokenizer, 1, "\n");
        let t3 = get_compare_token(&tokenizer, 2, "World!");
        assert!(tokenizer.tokens.len() == 3);
        assert_eq!(t1.token_type, TokenType::Text);
        assert_eq!(t2.token_type, TokenType::Newline);
        assert_eq!(t3.token_type, TokenType::Text);
    }

    #[test]
    fn test_tokenize_actual_example() {
        let source = "// Executes pop and push commands using the static segment.\npush constant 111\npop static 8\nsub\nadd\n".to_string();
        let mut tokenizer = Tokenizer::new(&source);
        tokenizer.tokenize();
        let t1 = get_compare_token(
            &tokenizer,
            0,
            "// Executes pop and push commands using the static segment.",
        );
        let t2 = get_compare_token(&tokenizer, 1, "\n");
        let t3 = get_compare_token(&tokenizer, 2, "push");
        let t4 = get_compare_token(&tokenizer, 3, " ");
        let t5 = get_compare_token(&tokenizer, 4, "constant");
        let t6 = get_compare_token(&tokenizer, 5, " ");
        let t7 = get_compare_token(&tokenizer, 6, "111");
        let t8 = get_compare_token(&tokenizer, 7, "\n");
        let t9 = get_compare_token(&tokenizer, 8, "pop");
        let t10 = get_compare_token(&tokenizer, 9, " ");
        let t11 = get_compare_token(&tokenizer, 10, "static");
        let t12 = get_compare_token(&tokenizer, 11, " ");
        let t13 = get_compare_token(&tokenizer, 12, "8");
        let t14 = get_compare_token(&tokenizer, 13, "\n");
        let t15 = get_compare_token(&tokenizer, 14, "sub");
        let t16 = get_compare_token(&tokenizer, 15, "\n");
        let t17 = get_compare_token(&tokenizer, 16, "add");
        let t18 = get_compare_token(&tokenizer, 17, "\n");
        assert!(tokenizer.tokens.len() == 18);
        assert_eq!(t1.token_type, TokenType::Comment);
        assert_eq!(t2.token_type, TokenType::Newline);
        assert_eq!(t3.token_type, TokenType::Text);
        assert_eq!(t4.token_type, TokenType::WhiteSpace);
        assert_eq!(t5.token_type, TokenType::Text);
        assert_eq!(t6.token_type, TokenType::WhiteSpace);
        assert_eq!(t7.token_type, TokenType::Text);
        assert_eq!(t8.token_type, TokenType::Newline);
        assert_eq!(t9.token_type, TokenType::Text);
        assert_eq!(t10.token_type, TokenType::WhiteSpace);
        assert_eq!(t11.token_type, TokenType::Text);
        assert_eq!(t12.token_type, TokenType::WhiteSpace);
        assert_eq!(t13.token_type, TokenType::Text);
        assert_eq!(t14.token_type, TokenType::Newline);
        assert_eq!(t15.token_type, TokenType::Text);
        assert_eq!(t16.token_type, TokenType::Newline);
        assert_eq!(t17.token_type, TokenType::Text);
        assert_eq!(t18.token_type, TokenType::Newline);
    }
}
