use crate::token::{Keyword, Token};
use crate::vm::{Arithmetic, Segment};
use hack_source::{LexError, SourceSpan, Spanned, SpannedLexer};
use std::{fmt, fmt::Display};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("unexpected token `{0}`")]
    UnexpectedToken(String),
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("invalid word `{0}`")]
    InvalidWord(String),
    #[error("invalid number `{0}`")]
    InvalidNumber(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{kind}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: SourceSpan,
}

type SpannedToken<'a> = Spanned<Token<'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmCommand {
    Push { segment: Segment, index: u16 },
    Pop { segment: Segment, index: u16 },
    Arithmetic(Arithmetic),
}

impl Display for VmCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Push { segment, index } => write!(f, "push {segment} {index}"),
            Self::Pop { segment, index } => write!(f, "pop {segment} {index}"),
            Self::Arithmetic(arithmetic) => write!(f, "{arithmetic}"),
        }
    }
}

pub struct Parser<'a> {
    tokenizer: SpannedLexer<'a, Token<'a>>,
    current: Option<Result<SpannedToken<'a>, LexError>>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut tokenizer = SpannedLexer::new(src);
        let current = tokenizer.next();
        Self { tokenizer, current }
    }

    pub fn parse_all(&mut self) -> Result<Vec<VmCommand>, ParseError> {
        let mut instructions = Vec::new();
        while let Some(begin) = self.next_token()? {
            match begin {
                SpannedToken {
                    value: Token::Keyword(keyword),
                    ..
                } => instructions.push(self.parse_vm_command(keyword)?),
                SpannedToken {
                    value: Token::Arithmetic(arithmetic),
                    ..
                } => instructions.push(self.parse_vm_arithmetic(arithmetic)?),
                token => return Err(self.unexpected(token)),
            }
            self.expect_line_end()?;
        }
        Ok(instructions)
    }

    fn parse_vm_command(&mut self, keyword: Keyword) -> Result<VmCommand, ParseError> {
        if let Some(next) = self.advance()? {
            match next {
                SpannedToken {
                    value: Token::Segment(segment),
                    ..
                } => {
                    let index = self.expect_index()?;
                    match keyword {
                        Keyword::Pop => Ok(VmCommand::Pop { segment, index }),
                        Keyword::Push => Ok(VmCommand::Push { segment, index }),
                    }
                }
                token => Err(self.unexpected(token)),
            }
        } else {
            Err(self.error(ParseErrorKind::UnexpectedEnd, self.eof_span()))
        }
    }

    fn parse_vm_arithmetic(&mut self, op: Arithmetic) -> Result<VmCommand, ParseError> {
        Ok(VmCommand::Arithmetic(op))
    }

    fn expect_index(&mut self) -> Result<u16, ParseError> {
        match self.advance()? {
            Some(SpannedToken {
                value: Token::Number(num_str),
                span,
            }) => {
                let index = num_str.parse::<u16>().map_err(|_| {
                    self.error(ParseErrorKind::InvalidNumber(num_str.to_string()), span)
                })?;
                Ok(index)
            }
            Some(token) => Err(self.unexpected(token)),
            None => Err(self.error(ParseErrorKind::UnexpectedEnd, self.eof_span())),
        }
    }

    fn expect_line_end(&mut self) -> Result<(), ParseError> {
        match self.advance()? {
            Some(SpannedToken {
                value: Token::Newline,
                ..
            })
            | None => Ok(()),
            Some(token) => Err(self.unexpected(token)),
        }
    }

    fn next_token(&mut self) -> Result<Option<SpannedToken<'a>>, ParseError> {
        self.skip_newlines()?;
        self.advance()
    }

    fn skip_newlines(&mut self) -> Result<(), ParseError> {
        while matches!(
            self.current.as_ref(),
            Some(Ok(SpannedToken {
                value: Token::Newline,
                ..
            }))
        ) {
            self.advance()?;
        }
        Ok(())
    }

    fn advance(&mut self) -> Result<Option<SpannedToken<'a>>, ParseError> {
        let token = self.current.take();
        self.current = self.tokenizer.next();
        match token {
            Some(Ok(token)) => Ok(Some(token)),
            None => Ok(None),
            Some(Err(error)) => Err(self.lexical_error(&error)),
        }
    }

    fn error(&self, kind: ParseErrorKind, span: SourceSpan) -> ParseError {
        ParseError { kind, span }
    }
    fn lexical_error(&self, error: &LexError) -> ParseError {
        self.error(
            ParseErrorKind::UnexpectedToken(error.text.clone()),
            error.span,
        )
    }
    fn eof_span(&self) -> SourceSpan {
        self.tokenizer.eof_span()
    }
    fn unexpected(&self, token: SpannedToken<'a>) -> ParseError {
        self.error(
            ParseErrorKind::UnexpectedToken(token.value.to_string()),
            token.span,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<Vec<VmCommand>, ParseErrorKind> {
        Parser::new(input).parse_all().map_err(|error| error.kind)
    }

    #[test]
    fn reports_unicode_aware_source_spans_across_crlf() {
        let source = "// first\r\npush 💥";
        let error = Parser::new(source).parse_all().unwrap_err();
        assert_eq!(
            error.kind,
            ParseErrorKind::UnexpectedToken("💥".to_string())
        );
        assert_eq!((error.span.start.line, error.span.start.column), (2, 6));
        assert_eq!((error.span.end.line, error.span.end.column), (2, 7));
        assert_eq!(&source[error.span.byte_range()], "💥");
    }

    #[test]
    fn reports_zero_width_eof_spans() {
        let source = "push constant";
        let error = Parser::new(source).parse_all().unwrap_err();
        assert_eq!(error.kind, ParseErrorKind::UnexpectedEnd);
        assert_eq!(error.span, SourceSpan::eof(source));
    }

    #[test]
    fn parses_push_command() {
        assert_eq!(
            parse("push constant 7"),
            Ok(vec![VmCommand::Push {
                segment: Segment::Constant,
                index: 7,
            }])
        );
    }

    #[test]
    fn parses_pop_command() {
        assert_eq!(
            parse("pop local 3"),
            Ok(vec![VmCommand::Pop {
                segment: Segment::Local,
                index: 3,
            }])
        );
    }

    #[test]
    fn parses_all_arithmetic_commands() {
        assert_eq!(
            parse("add\nsub\nneg\neq\ngt\nlt\nand\nor\nnot"),
            Ok(vec![
                VmCommand::Arithmetic(Arithmetic::Add),
                VmCommand::Arithmetic(Arithmetic::Sub),
                VmCommand::Arithmetic(Arithmetic::Neg),
                VmCommand::Arithmetic(Arithmetic::Eq),
                VmCommand::Arithmetic(Arithmetic::Gt),
                VmCommand::Arithmetic(Arithmetic::Lt),
                VmCommand::Arithmetic(Arithmetic::And),
                VmCommand::Arithmetic(Arithmetic::Or),
                VmCommand::Arithmetic(Arithmetic::Not),
            ])
        );
    }

    #[test]
    fn parses_all_segments() {
        assert_eq!(
            parse(
                "\
                push local 0\n\
                push argument 1\n\
                push this 2\n\
                push that 3\n\
                push pointer 0\n\
                push temp 4\n\
                push constant 5\n\
                push static 6"
            ),
            Ok(vec![
                VmCommand::Push {
                    segment: Segment::Local,
                    index: 0,
                },
                VmCommand::Push {
                    segment: Segment::Argument,
                    index: 1,
                },
                VmCommand::Push {
                    segment: Segment::This,
                    index: 2,
                },
                VmCommand::Push {
                    segment: Segment::That,
                    index: 3,
                },
                VmCommand::Push {
                    segment: Segment::Pointer,
                    index: 0,
                },
                VmCommand::Push {
                    segment: Segment::Temp,
                    index: 4,
                },
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 5,
                },
                VmCommand::Push {
                    segment: Segment::Static,
                    index: 6,
                },
            ])
        );
    }

    #[test]
    fn parses_multiple_commands() {
        assert_eq!(
            parse(
                "\
                push constant 7\n\
                push constant 8\n\
                add\n\
                pop temp 0"
            ),
            Ok(vec![
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 7,
                },
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 8,
                },
                VmCommand::Arithmetic(Arithmetic::Add),
                VmCommand::Pop {
                    segment: Segment::Temp,
                    index: 0,
                },
            ])
        );
    }

    #[test]
    fn skips_blank_lines() {
        assert_eq!(
            parse("\n\npush constant 1\n\n\nadd\n\n"),
            Ok(vec![
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 1,
                },
                VmCommand::Arithmetic(Arithmetic::Add),
            ])
        );
    }

    #[test]
    fn skips_comments() {
        assert_eq!(
            parse(
                "\
                // Push two values\n\
                push constant 7 // first value\n\
                push constant 8 // second value\n\
                add // add them"
            ),
            Ok(vec![
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 7,
                },
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 8,
                },
                VmCommand::Arithmetic(Arithmetic::Add),
            ])
        );
    }

    #[test]
    fn empty_input_produces_no_commands() {
        assert_eq!(parse(""), Ok(vec![]));
    }

    #[test]
    fn whitespace_and_comments_produce_no_commands() {
        assert_eq!(
            parse(
                "\
                // first comment\n\
                \n\
                // second comment"
            ),
            Ok(vec![])
        );
    }

    #[test]
    fn reports_unexpected_end_after_push() {
        assert_eq!(parse("push"), Err(ParseErrorKind::UnexpectedEnd));
    }

    #[test]
    fn reports_unexpected_end_after_pop() {
        assert_eq!(parse("pop"), Err(ParseErrorKind::UnexpectedEnd));
    }

    #[test]
    fn reports_unexpected_end_when_index_is_missing() {
        assert_eq!(parse("push constant"), Err(ParseErrorKind::UnexpectedEnd));
    }

    #[test]
    fn reports_unexpected_token_when_segment_is_missing() {
        assert_eq!(
            parse("push 7"),
            Err(ParseErrorKind::UnexpectedToken("7".to_string()))
        );
    }

    #[test]
    fn reports_unexpected_token_when_arithmetic_appears_as_segment() {
        assert_eq!(
            parse("push add 7"),
            Err(ParseErrorKind::UnexpectedToken("add".to_string()))
        );
    }

    #[test]
    fn reports_unexpected_token_when_index_is_not_a_number() {
        assert_eq!(
            parse("push constant add"),
            Err(ParseErrorKind::UnexpectedToken("add".to_string()))
        );
    }

    #[test]
    fn reports_unknown_word_at_command_position() {
        assert_eq!(
            parse("multiply"),
            Err(ParseErrorKind::UnexpectedToken("multiply".to_string()))
        );
        assert_eq!(
            parse("💥"),
            Err(ParseErrorKind::UnexpectedToken("💥".to_string()))
        );
    }

    #[test]
    fn reports_unknown_word_as_segment() {
        assert_eq!(
            parse("push banana 1"),
            Err(ParseErrorKind::UnexpectedToken("banana".to_string()))
        );
    }

    #[test]
    fn reports_unknown_word_as_index() {
        assert_eq!(
            parse("push constant banana"),
            Err(ParseErrorKind::UnexpectedToken("banana".to_string()))
        );
    }

    #[test]
    fn rejects_negative_index() {
        assert_eq!(
            parse("push constant -25"),
            Err(ParseErrorKind::UnexpectedToken("-25".to_string()))
        );
    }

    #[test]
    fn rejects_number_larger_than_u16() {
        assert_eq!(
            parse("push constant 65536"),
            Err(ParseErrorKind::InvalidNumber("65536".to_string()))
        );
    }

    #[test]
    fn accepts_maximum_u16_value() {
        assert_eq!(
            parse("push constant 65535"),
            Ok(vec![VmCommand::Push {
                segment: Segment::Constant,
                index: u16::MAX,
            }])
        );
    }

    #[test]
    fn stops_at_first_parse_error() {
        assert_eq!(
            parse(
                "\
                push constant 1\n\
                push banana 2\n\
                add"
            ),
            Err(ParseErrorKind::UnexpectedToken("banana".to_string()))
        );
    }
}
