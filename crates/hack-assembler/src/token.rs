use hack_source::PositionTracker;
use logos::Logos;

/// Lexical tokens produced from Hack assembly source.
///
/// The tokenizer only classifies source text by shape. It does not decide
/// whether an identifier is being used as a dest, comp, jump, label, or
/// A-instruction symbol; those grammar roles are assigned by the parser.
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(extras = PositionTracker)]
#[logos(skip r#"[^\S\n]"#)]
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
pub enum Token<'a> {
    // any number, the parser will do overflow checking.
    #[regex("[0-9]+", |lex| lex.slice(), priority = 3)]
    Number(&'a str),
    // Any sequence of characters that are a valid asm symbol. The parser will figure out based on context if this is valid.
    #[regex("[_.$:a-zA-Z0-9]+", |lex| lex.slice())]
    Identifier(&'a str),
    #[token(";")]
    SemiColon,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("!")]
    Not,
    #[token("=")]
    Eq,
    #[token("@")]
    At,
    #[token("\n")]
    Newline,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn tokenizer_never_panics_for_arbitrary_bounded_text(
            input in prop::collection::vec(any::<char>(), 0..=2048)
                .prop_map(|characters| characters.into_iter().collect::<String>())
        ) {
            let _: Vec<_> = Token::lexer(&input).collect();
        }
    }

    fn tokens(input: &str) -> Result<Vec<Token<'_>>, ()> {
        let mut tokens = Vec::new();

        for token in Token::lexer(input) {
            tokens.push(token?);
        }

        Ok(tokens)
    }

    #[test]
    fn emits_newline_tokens() -> Result<(), ()> {
        assert_eq!(
            tokens("@2\nD=A\n")?,
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
        Ok(())
    }

    #[test]
    fn keeps_newline_after_comment() -> Result<(), ()> {
        assert_eq!(
            tokens("@2 // comment\nD=A")?,
            vec![
                Token::At,
                Token::Number("2"),
                Token::Newline,
                Token::Identifier("D"),
                Token::Eq,
                Token::Identifier("A"),
            ]
        );
        Ok(())
    }

    #[test]
    fn classifies_a_number() -> Result<(), ()> {
        assert_eq!(tokens("@123")?, vec![Token::At, Token::Number("123")]);
        Ok(())
    }

    #[test]
    fn classifies_a_symbol() -> Result<(), ()> {
        assert_eq!(tokens("@i")?, vec![Token::At, Token::Identifier("i")]);
        Ok(())
    }

    #[test]
    fn classifies_label_symbol() -> Result<(), ()> {
        assert_eq!(
            tokens("(END_GT)")?,
            vec![
                Token::OpenParen,
                Token::Identifier("END_GT"),
                Token::CloseParen,
            ]
        );
        Ok(())
    }

    #[test]
    fn classifies_c_instruction_parts() -> Result<(), ()> {
        assert_eq!(
            tokens("D=D+A")?,
            vec![
                Token::Identifier("D"),
                Token::Eq,
                Token::Identifier("D"),
                Token::Plus,
                Token::Identifier("A"),
            ]
        );
        Ok(())
    }
}
