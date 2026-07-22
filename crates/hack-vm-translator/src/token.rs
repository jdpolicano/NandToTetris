use std::fmt::Display;

use crate::vm::{Arithmetic, Segment};
use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[^\S\n]")]
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
pub enum Token<'a> {
    #[token("push", |_| Keyword::Push)]
    #[token("pop", |_| Keyword::Pop)]
    Keyword(Keyword),
    #[token("add", |_| Arithmetic::Add)]
    #[token("sub", |_| Arithmetic::Sub)]
    #[token("neg", |_| Arithmetic::Neg)]
    #[token("eq", |_| Arithmetic::Eq)]
    #[token("lt", |_| Arithmetic::Lt)]
    #[token("gt", |_| Arithmetic::Gt)]
    #[token("and", |_| Arithmetic::And)]
    #[token("or", |_| Arithmetic::Or)]
    #[token("not", |_| Arithmetic::Not)]
    Arithmetic(Arithmetic),
    #[token("local", |_| Segment::Local)]
    #[token("argument", |_| Segment::Argument)]
    #[token("this", |_| Segment::This)]
    #[token("that", |_| Segment::That)]
    #[token("pointer", |_| Segment::Pointer)]
    #[token("temp", |_| Segment::Temp)]
    #[token("constant", |_| Segment::Constant)]
    #[token("static", |_| Segment::Static)]
    Segment(Segment),
    #[regex("[0-9]+", |lex| lex.slice(), priority = 3)]
    Number(&'a str),
    #[regex(r"[^\s/]+", |lex| lex.slice(), priority = 1)]
    Unknown(&'a str),
    #[token("\n")]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token<'_>> {
        Token::lexer(input).map(Result::unwrap).collect()
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
    fn skips_an_inline_comment_without_preceding_whitespace() {
        assert_eq!(
            tokens("push constant 1// place 1 on the stack\nadd"),
            vec![
                Token::Keyword(Keyword::Push),
                Token::Segment(Segment::Constant),
                Token::Number("1"),
                Token::Newline,
                Token::Arithmetic(Arithmetic::Add),
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
    fn tokenizes_non_ascii_input_without_panicking() {
        assert_eq!(
            tokens("push 💥"),
            vec![Token::Keyword(Keyword::Push), Token::Unknown("💥")]
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
