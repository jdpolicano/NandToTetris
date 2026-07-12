use std::fmt;

/// Lexical tokens produced from Hack assembly source.
///
/// The tokenizer only classifies source text by shape. It does not decide
/// whether an identifier is being used as a dest, comp, jump, label, or
/// A-instruction symbol; those grammar roles are assigned by the parser.
#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Number(&'a str),     // digit-only literal
    Identifier(&'a str), // symbol-like text or mnemonic
    SemiColon,           // ";"
    OpenParen,           // "("
    CloseParen,          // ")"
    Plus,                // "+"
    Minus,               // "-"
    Amp,                 // "&"
    Pipe,                // "|"
    Eq,                  // "="
    At,                  // "@"
    Newline,             // "\n"
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(txt) => write!(f, "'{}'", txt),
            Self::Identifier(txt) => write!(f, "'{}'", txt),
            Self::SemiColon => write!(f, "';'"),
            Self::OpenParen => write!(f, "'('"),
            Self::CloseParen => write!(f, "')'"),
            Self::Plus => write!(f, "'+'"),
            Self::Minus => write!(f, "'-'"),
            Self::Amp => write!(f, "'&'"),
            Self::Pipe => write!(f, "'|'"),
            Self::Eq => write!(f, "'='"),
            Self::At => write!(f, "'@'"),
            Self::Newline => write!(f, "'\\n'"),
        }
    }
}

impl<'a> TryFrom<char> for Token<'a> {
    type Error = ();
    fn try_from(value: char) -> Result<Self, ()> {
        match value {
            ';' => Ok(Self::SemiColon),
            '(' => Ok(Self::OpenParen),
            ')' => Ok(Self::CloseParen),
            '+' => Ok(Self::Plus),
            '-' => Ok(Self::Minus),
            '&' => Ok(Self::Amp),
            '|' => Ok(Self::Pipe),
            '=' => Ok(Self::Eq),
            '@' => Ok(Self::At),
            '\n' => Ok(Self::Newline),
            _ => Err(()),
        }
    }
}

impl<'a> From<&'a str> for Token<'a> {
    fn from(value: &'a str) -> Self {
        if value.chars().all(|c| c.is_ascii_digit()) {
            Self::Number(value)
        } else {
            Self::Identifier(value)
        }
    }
}

pub struct Tokenizer<'a> {
    stream: &'a str,
    origin: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// Creates a tokenizer that borrows slices from the original input.
    pub fn new(stream: &'a str) -> Self {
        Self {
            stream,
            origin: stream,
        }
    }

    /// Scans a non-punctuation run and classifies it as a number or identifier.
    fn scan_str(&mut self) -> Option<Token<'a>> {
        for (idx, c) in self.stream.char_indices() {
            if c.is_whitespace() || self.is_special_token(c) {
                let toke = Token::from(&self.stream[0..idx]);
                self.stream = &self.stream[idx..];
                return Some(toke);
            }
        }
        let toke = Token::from(&self.stream[0..]);
        self.stream = &self.stream[self.stream.len()..];
        Some(toke)
    }

    fn on_white_space(&self) -> bool {
        self.stream
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_whitespace() && c != '\n')
    }

    fn on_comment(&self) -> bool {
        // Find the byte index of the 2nd character ahead (0-indexed position 2)
        let byte_end = 2.min(self.stream.len());
        matches!(&self.stream[0..byte_end], "//")
    }

    fn should_skip(&self) -> bool {
        self.on_comment() || self.on_white_space()
    }

    fn skip_comment_text(&mut self) {
        let skip_to_idx = self
            .stream
            .find('\n') // Finds the byte index of '\n'
            .unwrap_or(self.stream.len());
        self.stream = &self.stream[skip_to_idx..];
    }

    fn skip_to_start(&mut self) {
        while self.should_skip() {
            if self.on_comment() {
                self.skip_comment_text();
            } else {
                // It's whitespace, consume exactly one character
                if let Some(c) = self.stream.chars().next() {
                    self.stream = &self.stream[c.len_utf8()..];
                }
            }
        }
    }

    fn is_special_token(&self, c: char) -> bool {
        Token::try_from(c).is_ok()
    }

    pub fn reset(&mut self) {
        self.stream = self.origin;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    /// Returns the next token, skipping spaces, tabs, and comments.
    ///
    /// Newlines are preserved because the parser uses them as instruction
    /// boundaries.
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_to_start();

        if self.stream.is_empty() {
            return None;
        }

        let c = self
            .stream
            .chars()
            .next()
            .expect("len check ensures this exists");

        match Token::try_from(c) {
            Ok(token) => {
                self.stream = &self.stream[c.len_utf8()..];
                Some(token)
            }
            Err(()) => self.scan_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token<'_>> {
        let mut tokens = Vec::new();

        for token in Tokenizer::new(input) {
            tokens.push(token);
        }

        tokens
    }

    #[test]
    fn emits_newline_tokens() {
        assert_eq!(
            tokens("@2\nD=A\n"),
            vec![
                Token::At,
                Token::Number("2"),
                Token::Newline,
                Token::Identifier("D"),
                Token::Eq,
                Token::Identifier("A"),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn keeps_newline_after_comment() {
        assert_eq!(
            tokens("@2 // comment\nD=A"),
            vec![
                Token::At,
                Token::Number("2"),
                Token::Newline,
                Token::Identifier("D"),
                Token::Eq,
                Token::Identifier("A"),
            ]
        );
    }

    #[test]
    fn classifies_a_number() {
        assert_eq!(tokens("@123"), vec![Token::At, Token::Number("123")]);
    }

    #[test]
    fn classifies_a_symbol() {
        assert_eq!(tokens("@i"), vec![Token::At, Token::Identifier("i")]);
    }

    #[test]
    fn classifies_label_symbol() {
        assert_eq!(
            tokens("(END_GT)"),
            vec![
                Token::OpenParen,
                Token::Identifier("END_GT"),
                Token::CloseParen,
            ]
        );
    }

    #[test]
    fn classifies_c_instruction_parts() {
        assert_eq!(
            tokens("D=D+A"),
            vec![
                Token::Identifier("D"),
                Token::Eq,
                Token::Identifier("D"),
                Token::Plus,
                Token::Identifier("A"),
            ]
        );
    }
}
