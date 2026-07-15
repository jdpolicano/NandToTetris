use std::fmt::Display;

use crate::vm::{Arithmetic, Segment};

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    Push,
    Pop,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Push => write!(f, "push"),
            Self::Pop => write!(f, "pop"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Keyword(Keyword),
    Arithmetic(Arithmetic),
    Segment(Segment),
    Number(&'a str),
    Unknown(&'a str),
    Newline,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyword(k) => write!(f, "{}", k),
            Self::Arithmetic(a) => write!(f, "{}", a),
            Self::Segment(s) => write!(f, "{}", s),
            Self::Number(num) => write!(f, "{}", num),
            Self::Unknown(word) => write!(f, "{}", word),
            Self::Newline => writeln!(f),
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

    pub fn reset(&mut self) {
        self.stream = self.origin;
    }

    fn read_until_whitespace(&mut self) -> &'a str {
        let skip_to_idx = self
            .stream
            .find(|c: char| c.is_whitespace()) // Finds the byte index of '\n'
            .unwrap_or(self.stream.len());
        let slice = &self.stream[..skip_to_idx];
        self.stream = &self.stream[skip_to_idx..];
        slice
    }

    fn on_white_space(&self) -> bool {
        self.stream
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_whitespace() && c != '\n')
    }

    fn should_skip(&self) -> bool {
        self.on_comment() || self.on_white_space()
    }

    fn on_comment(&self) -> bool {
        // Find the byte index of the 2nd character ahead (0-indexed position 2)
        let byte_end = 2.min(self.stream.len());
        matches!(&self.stream[0..byte_end], "//")
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

    fn skip_comment_text(&mut self) {
        let skip_to_idx = self
            .stream
            .find('\n') // Finds the byte index of '\n'
            .unwrap_or(self.stream.len());
        self.stream = &self.stream[skip_to_idx..];
    }

    fn scan_str(&mut self) -> Option<Token<'a>> {
        let slice = self.read_until_whitespace();
        match slice {
            "add" => Some(Token::Arithmetic(Arithmetic::Add)),
            "sub" => Some(Token::Arithmetic(Arithmetic::Sub)),
            "neg" => Some(Token::Arithmetic(Arithmetic::Neg)),
            "eq" => Some(Token::Arithmetic(Arithmetic::Eq)),
            "lt" => Some(Token::Arithmetic(Arithmetic::Lt)),
            "gt" => Some(Token::Arithmetic(Arithmetic::Gt)),
            "and" => Some(Token::Arithmetic(Arithmetic::And)),
            "or" => Some(Token::Arithmetic(Arithmetic::Or)),
            "not" => Some(Token::Arithmetic(Arithmetic::Not)),
            "push" => Some(Token::Keyword(Keyword::Push)),
            "pop" => Some(Token::Keyword(Keyword::Pop)),
            "local" => Some(Token::Segment(Segment::Local)),
            "pointer" => Some(Token::Segment(Segment::Pointer)),
            "this" => Some(Token::Segment(Segment::This)),
            "that" => Some(Token::Segment(Segment::That)),
            "argument" => Some(Token::Segment(Segment::Argument)),
            "temp" => Some(Token::Segment(Segment::Temp)),
            "constant" => Some(Token::Segment(Segment::Constant)),
            "static" => Some(Token::Segment(Segment::Static)),
            _ => {
                if slice.chars().all(|c| c.is_ascii_digit()) {
                    Some(Token::Number(slice))
                } else {
                    Some(Token::Unknown(slice))
                }
            }
        }
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

        match &self.stream[..1] {
            "\n" => {
                self.stream = &self.stream[1..];
                Some(Token::Newline)
            }
            _ => self.scan_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token<'_>> {
        Tokenizer::new(input).collect()
    }

    #[test]
    fn classifies_all_arithmetic_keywords() {
        assert_eq!(
            tokens("add sub neg eq gt lt and or not"),
            vec![
                Token::Arithmetic(Arithmetic::Add),
                Token::Arithmetic(Arithmetic::Sub),
                Token::Arithmetic(Arithmetic::Neg),
                Token::Arithmetic(Arithmetic::Eq),
                Token::Arithmetic(Arithmetic::Gt),
                Token::Arithmetic(Arithmetic::Lt),
                Token::Arithmetic(Arithmetic::And),
                Token::Arithmetic(Arithmetic::Or),
                Token::Arithmetic(Arithmetic::Not),
            ]
        );
    }

    #[test]
    fn classifies_push_and_pop_keywords() {
        assert_eq!(
            tokens("push pop"),
            vec![Token::Keyword(Keyword::Push), Token::Keyword(Keyword::Pop),]
        );
    }

    #[test]
    fn classifies_all_memory_segments() {
        assert_eq!(
            tokens("local argument this that pointer temp constant static"),
            vec![
                Token::Segment(Segment::Local),
                Token::Segment(Segment::Argument),
                Token::Segment(Segment::This),
                Token::Segment(Segment::That),
                Token::Segment(Segment::Pointer),
                Token::Segment(Segment::Temp),
                Token::Segment(Segment::Constant),
                Token::Segment(Segment::Static),
            ]
        );
    }

    #[test]
    fn classifies_a_number() {
        assert_eq!(tokens("123"), vec![Token::Number("123")]);
    }

    #[test]
    fn classifies_zero_as_a_number() {
        assert_eq!(tokens("0"), vec![Token::Number("0")]);
    }

    #[test]
    fn preserves_leading_zeroes_in_numbers() {
        assert_eq!(tokens("00123"), vec![Token::Number("00123")]);
    }

    #[test]
    fn classifies_unrecognized_words_as_unknown() {
        assert_eq!(
            tokens("multiply foo"),
            vec![Token::Unknown("multiply"), Token::Unknown("foo")]
        );
    }

    #[test]
    fn classifies_mixed_alphanumeric_tokens_as_unknown() {
        assert_eq!(
            tokens("123abc abc123"),
            vec![Token::Unknown("123abc"), Token::Unknown("abc123")]
        );
    }

    #[test]
    fn skips_spaces_and_tabs() {
        assert_eq!(
            tokens(" \t  push\tconstant   7"),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("7"),
            ]
        );
    }

    #[test]
    fn emits_newline_tokens() {
        assert_eq!(
            tokens("push constant 1\npop local 2\n"),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
                Token::Newline,
                Token::Keyword(Keyword::Pop),
                Token::Segment(Segment::Local),
                Token::Number("2"),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn emits_tokens_for_blank_lines() {
        assert_eq!(
            tokens("\n\npush constant 1\n\n"),
            vec![
                Token::Newline,
                Token::Newline,
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
                Token::Newline,
                Token::Newline,
            ]
        );
    }

    #[test]
    fn skips_an_inline_comment_but_preserves_its_newline() {
        assert_eq!(
            tokens("push constant 1 // place 1 on the stack\npop local 0"),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
                Token::Newline,
                Token::Keyword(Keyword::Pop),
                Token::Segment(Segment::Local),
                Token::Number("0"),
            ]
        );
    }

    #[test]
    fn skips_a_full_line_comment_but_preserves_its_newline() {
        assert_eq!(
            tokens("// this is a comment\npush constant 1"),
            vec![
                Token::Newline,
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
            ]
        );
    }

    #[test]
    fn skips_a_comment_at_end_of_input() {
        assert_eq!(
            tokens("push constant 1 // trailing comment"),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
            ]
        );
    }

    #[test]
    fn skips_input_containing_only_a_comment() {
        assert_eq!(tokens("// nothing to tokenize"), vec![]);
    }

    #[test]
    fn handles_windows_line_endings() {
        assert_eq!(
            tokens("push constant 1\r\npop local 2\r\n"),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
                Token::Newline,
                Token::Keyword(Keyword::Pop),
                Token::Segment(Segment::Local),
                Token::Number("2"),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn empty_input_produces_no_tokens() {
        assert_eq!(tokens(""), vec![]);
    }

    #[test]
    fn whitespace_only_input_produces_no_tokens() {
        assert_eq!(tokens(" \t\r  "), vec![]);
    }

    #[test]
    fn reset_restarts_tokenization_from_the_beginning() {
        let mut tokenizer = Tokenizer::new("push constant 7");

        assert_eq!(tokenizer.next(), Some(Token::Keyword(Keyword::Push)));
        assert_eq!(tokenizer.next(), Some(Token::Segment(Segment::Constant)));

        tokenizer.reset();

        assert_eq!(
            tokenizer.collect::<Vec<_>>(),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("7"),
            ]
        );
    }

    #[test]
    fn iterator_returns_none_repeatedly_after_exhaustion() {
        let mut tokenizer = Tokenizer::new("add");

        assert_eq!(tokenizer.next(), Some(Token::Arithmetic(Arithmetic::Add)));
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn token_slices_reference_the_original_input() {
        let input = String::from("push constant 123");
        let result = tokens(&input);

        assert_eq!(
            result,
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("123"),
            ]
        );
    }

    #[test]
    fn tokenizes_a_small_vm_program() {
        let input = "\
            // Adds two constants\n\
            push constant 7\n\
            push constant 8\n\
            add\n\
            pop temp 0\n";

        assert_eq!(
            tokens(input),
            vec![
                Token::Newline,
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("7"),
                Token::Newline,
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("8"),
                Token::Newline,
                Token::Arithmetic(Arithmetic::Add),
                Token::Newline,
                Token::Keyword(Keyword::Pop),
                Token::Segment(Segment::Temp),
                Token::Number("0"),
                Token::Newline,
            ]
        );
    }
}
