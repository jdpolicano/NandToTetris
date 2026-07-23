use crate::instruction::{
    AValue, Comp, Dest, Instruction, InstructionError, Jump, predefined_symbol, validate_symbol,
};
use crate::token::Token;
use hack_source::SourceSpan;
use logos::{Lexer, Logos};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("unexpected token `{0}`")]
    UnexpectedToken(String),
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("invalid symbol `{0}`")]
    InvalidSymbol(String),
    #[error("invalid number `{0}`")]
    InvalidNumber(String),
    #[error("invalid destination `{0}`")]
    InvalidDest(String),
    #[error("invalid computation `{0}`")]
    InvalidComp(String),
    #[error("invalid jump `{0}`")]
    InvalidJump(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{kind}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
struct SpannedToken<'a> {
    token: Token<'a>,
    span: SourceSpan,
}

#[derive(Debug)]
struct LexError {
    text: String,
    span: SourceSpan,
}

pub struct Parser<'a> {
    source: &'a str,
    tokenizer: Lexer<'a, Token<'a>>,
    current: Option<Result<SpannedToken<'a>, LexError>>,
}

impl<'a> Parser<'a> {
    /// Creates a parser over the provided Hack assembly source.
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Token::lexer(input);
        let current = next_spanned(&mut tokenizer, input);

        Self {
            source: input,
            tokenizer,
            current,
        }
    }

    /// Parses the full input into instructions, skipping blank lines.
    pub fn parse_all(&mut self) -> Result<Vec<Instruction>, ParseError> {
        let mut instructions = Vec::new();

        self.skip_to_valid_position()?;
        while self.current.is_some() {
            instructions.push(self.parse_instruction()?);
            self.skip_to_valid_position()?;
        }

        Ok(instructions)
    }

    /// Consumes the first token of an instruction and dispatches to the
    /// grammar-specific parser for the rest of that instruction.
    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        match self.advance()? {
            Some(SpannedToken {
                token: Token::At, ..
            }) => self.parse_a_instruction(),
            Some(SpannedToken {
                token: Token::OpenParen,
                ..
            }) => self.parse_label(),
            Some(token) => self.parse_c_instruction(token),
            None => Err(self.error(ParseErrorKind::UnexpectedEnd, self.eof_span())),
        }
    }

    fn parse_a_instruction(&mut self) -> Result<Instruction, ParseError> {
        let value = match self.advance()? {
            Some(
                token @ SpannedToken {
                    token: Token::Number(_),
                    ..
                },
            ) => {
                let Token::Number(value) = token.token else {
                    unreachable!()
                };
                parse_a_number(value).map_err(|kind| self.error(kind, token.span))?
            }
            Some(
                token @ SpannedToken {
                    token: Token::Identifier(_),
                    ..
                },
            ) => {
                let Token::Identifier(value) = token.token else {
                    unreachable!()
                };
                parse_a_symbol(value).map_err(|kind| self.error(kind, token.span))?
            }
            Some(token) => return Err(self.unexpected(token)),
            None => return Err(self.error(ParseErrorKind::UnexpectedEnd, self.eof_span())),
        };

        self.expect_newline_or_end()?;
        Ok(Instruction::A(value))
    }

    fn parse_label(&mut self) -> Result<Instruction, ParseError> {
        let (label, label_span) = match self.advance()? {
            Some(SpannedToken {
                token: Token::Identifier(value),
                span,
            }) => (value, span),
            Some(token) => return Err(self.unexpected(token)),
            None => return Err(self.error(ParseErrorKind::UnexpectedEnd, self.eof_span())),
        };

        self.expect(Token::CloseParen)?;
        self.expect_newline_or_end()?;
        validate_symbol(label).map_err(|error| self.error(invalid_symbol(error), label_span))?;
        Ok(Instruction::Label(label.to_string()))
    }

    fn parse_c_instruction(
        &mut self,
        first_token: SpannedToken<'a>,
    ) -> Result<Instruction, ParseError> {
        let first = self.read_component_until_control(first_token)?;

        match self.peek()? {
            Some(SpannedToken {
                token: Token::Eq, ..
            }) => self.parse_assignment(&first),
            Some(SpannedToken {
                token: Token::SemiColon,
                ..
            }) => self.parse_jump_instruction(&first),
            Some(SpannedToken {
                token: Token::Newline,
                ..
            })
            | None => self.parse_bare_comp(&first),
            Some(token) => Err(self.error(
                ParseErrorKind::UnexpectedToken(token_to_string(&token.token)),
                token.span,
            )),
        }
    }

    fn parse_assignment(
        &mut self,
        dest_tokens: &[SpannedToken<'a>],
    ) -> Result<Instruction, ParseError> {
        let dest = parse_dest(dest_tokens)
            .map_err(|kind| self.error(kind, self.component_span(dest_tokens)))?;
        self.expect(Token::Eq)?;

        let comp_tokens = self.read_remaining_component_until_control()?;
        let comp = parse_comp(&comp_tokens)
            .map_err(|kind| self.error(kind, self.component_span(&comp_tokens)))?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: Some(dest),
            comp,
            jump: None,
        })
    }

    fn parse_jump_instruction(
        &mut self,
        comp_tokens: &[SpannedToken<'a>],
    ) -> Result<Instruction, ParseError> {
        let comp = parse_comp(comp_tokens)
            .map_err(|kind| self.error(kind, self.component_span(comp_tokens)))?;
        self.expect(Token::SemiColon)?;

        let jump_tokens = self.read_remaining_component_until_control()?;
        let jump = parse_jump(&jump_tokens)
            .map_err(|kind| self.error(kind, self.component_span(&jump_tokens)))?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: None,
            comp,
            jump: Some(jump),
        })
    }

    fn parse_bare_comp(
        &mut self,
        comp_tokens: &[SpannedToken<'a>],
    ) -> Result<Instruction, ParseError> {
        let comp = parse_comp(comp_tokens)
            .map_err(|kind| self.error(kind, self.component_span(comp_tokens)))?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: None,
            comp,
            jump: None,
        })
    }

    fn read_component_until_control(
        &mut self,
        first_token: SpannedToken<'a>,
    ) -> Result<Vec<SpannedToken<'a>>, ParseError> {
        let mut value = vec![first_token];

        self.read_remaining_component_into(&mut value)?;
        Ok(value)
    }

    /// Reads a C-instruction component after a control token such as `=` or `;`
    /// has already been consumed.
    fn read_remaining_component_until_control(
        &mut self,
    ) -> Result<Vec<SpannedToken<'a>>, ParseError> {
        let mut value = Vec::new();
        self.read_remaining_component_into(&mut value)?;
        Ok(value)
    }

    /// Appends component tokens until a token that separates C-instruction
    /// fields is reached. Semantic validity is still checked later by
    /// `Dest`, `Comp`, and `Jump`.
    fn read_remaining_component_into(
        &mut self,
        value: &mut Vec<SpannedToken<'a>>,
    ) -> Result<(), ParseError> {
        loop {
            match self.peek()? {
                Some(SpannedToken {
                    token: Token::Eq | Token::SemiColon | Token::Newline,
                    ..
                })
                | None => {
                    break;
                }
                Some(_) => value.push(
                    self.advance()?
                        .expect("peek confirmed a current token exists"),
                ),
            }
        }

        Ok(())
    }

    /// Consumes the current token if it is the expected terminal variant.
    fn expect(&mut self, expected: Token<'_>) -> Result<(), ParseError> {
        match self.advance()? {
            Some(token) if same_token_variant(&token.token, &expected) => Ok(()),
            Some(token) => Err(self.unexpected(token)),
            None => Err(self.error(ParseErrorKind::UnexpectedEnd, self.eof_span())),
        }
    }

    fn expect_newline_or_end(&mut self) -> Result<(), ParseError> {
        match self.advance()? {
            Some(SpannedToken {
                token: Token::Newline,
                ..
            })
            | None => Ok(()),
            Some(token) => Err(self.unexpected(token)),
        }
    }

    fn skip_to_valid_position(&mut self) -> Result<(), ParseError> {
        self.check_lex_error()?;
        while matches!(
            self.current.as_ref(),
            Some(Ok(SpannedToken {
                token: Token::Newline,
                ..
            }))
        ) {
            self.advance()?;
        }
        Ok(())
    }

    /// Consumes and returns the current token, then advances one token ahead.
    fn advance(&mut self) -> Result<Option<SpannedToken<'a>>, ParseError> {
        let token = self.current.take();
        self.current = next_spanned(&mut self.tokenizer, self.source);
        match token {
            Some(Ok(token)) => Ok(Some(token)),
            None => Ok(None),
            Some(Err(error)) => {
                Err(self.error(ParseErrorKind::UnexpectedToken(error.text), error.span))
            }
        }
    }

    fn peek(&self) -> Result<Option<&SpannedToken<'a>>, ParseError> {
        match self.current.as_ref() {
            Some(Ok(token)) => Ok(Some(token)),
            None => Ok(None),
            Some(Err(error)) => Err(self.error(
                ParseErrorKind::UnexpectedToken(error.text.clone()),
                error.span,
            )),
        }
    }

    fn check_lex_error(&self) -> Result<(), ParseError> {
        self.peek().map(|_| ())
    }
    fn eof_span(&self) -> SourceSpan {
        let position = self.tokenizer.extras.position();
        if position.offset == self.source.len() {
            SourceSpan::at(position)
        } else {
            SourceSpan::eof(self.source)
        }
    }
    fn component_span(&self, tokens: &[SpannedToken<'_>]) -> SourceSpan {
        match (tokens.first(), tokens.last()) {
            (Some(first), Some(last)) => first.span.cover(last.span),
            _ => self.eof_span(),
        }
    }
    fn error(&self, kind: ParseErrorKind, span: SourceSpan) -> ParseError {
        ParseError { kind, span }
    }
    fn unexpected(&self, token: SpannedToken<'a>) -> ParseError {
        self.error(
            ParseErrorKind::UnexpectedToken(token_to_string(&token.token)),
            token.span,
        )
    }
}

impl TryFrom<&str> for Dest {
    type Error = ParseErrorKind;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "M" => Ok(Self::M),
            "D" => Ok(Self::D),
            "MD" => Ok(Self::MD),
            "A" => Ok(Self::A),
            "AM" => Ok(Self::AM),
            "AD" => Ok(Self::AD),
            "AMD" => Ok(Self::AMD),
            _ => Err(ParseErrorKind::InvalidDest(value.to_string())),
        }
    }
}

impl TryFrom<&str> for Comp {
    type Error = ParseErrorKind;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "0" => Ok(Self::Zero),
            "1" => Ok(Self::One),
            "-1" => Ok(Self::NegOne),
            "D" => Ok(Self::D),
            "A" => Ok(Self::A),
            "M" => Ok(Self::M),
            "!D" => Ok(Self::NotD),
            "!A" => Ok(Self::NotA),
            "!M" => Ok(Self::NotM),
            "-D" => Ok(Self::NegD),
            "-A" => Ok(Self::NegA),
            "-M" => Ok(Self::NegM),
            "D+1" => Ok(Self::DPlusOne),
            "A+1" => Ok(Self::APlusOne),
            "M+1" => Ok(Self::MPlusOne),
            "D-1" => Ok(Self::DMinusOne),
            "A-1" => Ok(Self::AMinusOne),
            "M-1" => Ok(Self::MMinusOne),
            "D+A" => Ok(Self::DPlusA),
            "D+M" => Ok(Self::DPlusM),
            "D-A" => Ok(Self::DMinusA),
            "D-M" => Ok(Self::DMinusM),
            "A-D" => Ok(Self::AMinusD),
            "M-D" => Ok(Self::MMinusD),
            "D&A" => Ok(Self::DAndA),
            "D&M" => Ok(Self::DAndM),
            "D|A" => Ok(Self::DOrA),
            "D|M" => Ok(Self::DOrM),
            _ => Err(ParseErrorKind::InvalidComp(value.to_string())),
        }
    }
}

impl TryFrom<&str> for Jump {
    type Error = ParseErrorKind;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "JGT" => Ok(Self::Jgt),
            "JEQ" => Ok(Self::Jeq),
            "JGE" => Ok(Self::Jge),
            "JLT" => Ok(Self::Jlt),
            "JNE" => Ok(Self::Jne),
            "JLE" => Ok(Self::Jle),
            "JMP" => Ok(Self::Jmp),
            _ => Err(ParseErrorKind::InvalidJump(value.to_string())),
        }
    }
}

fn parse_a_number(value: &str) -> Result<AValue, ParseErrorKind> {
    let number = value
        .parse::<u16>()
        .map_err(|_| ParseErrorKind::InvalidNumber(value.to_string()))?;

    if number > 32_767 {
        return Err(ParseErrorKind::InvalidNumber(value.to_string()));
    }

    Ok(AValue::Number(number))
}

fn parse_a_symbol(value: &str) -> Result<AValue, ParseErrorKind> {
    if let Some(symbol) = predefined_symbol(value) {
        return Ok(AValue::Predefined(symbol));
    }

    validate_symbol(value).map_err(invalid_symbol)?;
    Ok(AValue::Symbol(value.to_string()))
}

fn parse_dest(tokens: &[SpannedToken<'_>]) -> Result<Dest, ParseErrorKind> {
    match tokens {
        [
            SpannedToken {
                token: Token::Identifier("M"),
                ..
            },
        ] => Ok(Dest::M),
        [
            SpannedToken {
                token: Token::Identifier("D"),
                ..
            },
        ] => Ok(Dest::D),
        [
            SpannedToken {
                token: Token::Identifier("MD"),
                ..
            },
        ] => Ok(Dest::MD),
        [
            SpannedToken {
                token: Token::Identifier("A"),
                ..
            },
        ] => Ok(Dest::A),
        [
            SpannedToken {
                token: Token::Identifier("AM"),
                ..
            },
        ] => Ok(Dest::AM),
        [
            SpannedToken {
                token: Token::Identifier("AD"),
                ..
            },
        ] => Ok(Dest::AD),
        [
            SpannedToken {
                token: Token::Identifier("AMD"),
                ..
            },
        ] => Ok(Dest::AMD),
        _ => Err(ParseErrorKind::InvalidDest(component_text(tokens))),
    }
}

fn parse_comp(tokens: &[SpannedToken<'_>]) -> Result<Comp, ParseErrorKind> {
    let bare: Vec<_> = tokens.iter().map(|token| token.token).collect();
    let comp = match bare.as_slice() {
        [Token::Number("0")] => Comp::Zero,
        [Token::Number("1")] => Comp::One,
        [Token::Minus, Token::Number("1")] => Comp::NegOne,
        [Token::Identifier("D")] => Comp::D,
        [Token::Identifier("A")] => Comp::A,
        [Token::Identifier("M")] => Comp::M,
        [Token::Not, Token::Identifier("D")] => Comp::NotD,
        [Token::Not, Token::Identifier("A")] => Comp::NotA,
        [Token::Not, Token::Identifier("M")] => Comp::NotM,
        [Token::Minus, Token::Identifier("D")] => Comp::NegD,
        [Token::Minus, Token::Identifier("A")] => Comp::NegA,
        [Token::Minus, Token::Identifier("M")] => Comp::NegM,
        [Token::Identifier("D"), Token::Plus, Token::Number("1")] => Comp::DPlusOne,
        [Token::Identifier("A"), Token::Plus, Token::Number("1")] => Comp::APlusOne,
        [Token::Identifier("M"), Token::Plus, Token::Number("1")] => Comp::MPlusOne,
        [Token::Identifier("D"), Token::Minus, Token::Number("1")] => Comp::DMinusOne,
        [Token::Identifier("A"), Token::Minus, Token::Number("1")] => Comp::AMinusOne,
        [Token::Identifier("M"), Token::Minus, Token::Number("1")] => Comp::MMinusOne,
        [Token::Identifier("D"), Token::Plus, Token::Identifier("A")] => Comp::DPlusA,
        [Token::Identifier("D"), Token::Plus, Token::Identifier("M")] => Comp::DPlusM,
        [Token::Identifier("D"), Token::Minus, Token::Identifier("A")] => Comp::DMinusA,
        [Token::Identifier("D"), Token::Minus, Token::Identifier("M")] => Comp::DMinusM,
        [Token::Identifier("A"), Token::Minus, Token::Identifier("D")] => Comp::AMinusD,
        [Token::Identifier("M"), Token::Minus, Token::Identifier("D")] => Comp::MMinusD,
        [Token::Identifier("D"), Token::Amp, Token::Identifier("A")] => Comp::DAndA,
        [Token::Identifier("D"), Token::Amp, Token::Identifier("M")] => Comp::DAndM,
        [Token::Identifier("D"), Token::Pipe, Token::Identifier("A")] => Comp::DOrA,
        [Token::Identifier("D"), Token::Pipe, Token::Identifier("M")] => Comp::DOrM,
        _ => return Err(ParseErrorKind::InvalidComp(component_text(tokens))),
    };

    Ok(comp)
}

fn parse_jump(tokens: &[SpannedToken<'_>]) -> Result<Jump, ParseErrorKind> {
    let bare: Vec<_> = tokens.iter().map(|token| token.token).collect();
    match bare.as_slice() {
        [Token::Identifier("JGT")] => Ok(Jump::Jgt),
        [Token::Identifier("JEQ")] => Ok(Jump::Jeq),
        [Token::Identifier("JGE")] => Ok(Jump::Jge),
        [Token::Identifier("JLT")] => Ok(Jump::Jlt),
        [Token::Identifier("JNE")] => Ok(Jump::Jne),
        [Token::Identifier("JLE")] => Ok(Jump::Jle),
        [Token::Identifier("JMP")] => Ok(Jump::Jmp),
        _ => Err(ParseErrorKind::InvalidJump(component_text(tokens))),
    }
}

fn token_to_string(token: &Token<'_>) -> String {
    match token {
        Token::Number(value) | Token::Identifier(value) => value.to_string(),
        Token::SemiColon => ";".to_string(),
        Token::OpenParen => "(".to_string(),
        Token::CloseParen => ")".to_string(),
        Token::Plus => "+".to_string(),
        Token::Minus => "-".to_string(),
        Token::Amp => "&".to_string(),
        Token::Pipe => "|".to_string(),
        Token::Eq => "=".to_string(),
        Token::At => "@".to_string(),
        Token::Newline => "\n".to_string(),
        Token::Not => "!".to_string(),
    }
}

fn component_text(tokens: &[SpannedToken<'_>]) -> String {
    tokens
        .iter()
        .map(|token| token_to_string(&token.token))
        .collect()
}

fn same_token_variant(left: &Token<'_>, right: &Token<'_>) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn invalid_symbol(error: InstructionError) -> ParseErrorKind {
    match error {
        InstructionError::InvalidSymbol(value) => ParseErrorKind::InvalidSymbol(value),
        InstructionError::AddressOutOfRange(_) => {
            unreachable!("symbol validation cannot produce an address error")
        }
    }
}

fn next_spanned<'a>(
    lexer: &mut Lexer<'a, Token<'a>>,
    source: &'a str,
) -> Option<Result<SpannedToken<'a>, LexError>> {
    let result = match lexer.next() {
        Some(result) => result,
        None => {
            lexer.extras.finish(source);
            return None;
        }
    };
    let range = lexer.span();
    let span = lexer.extras.span_for(source, range);
    Some(match result {
        Ok(token) => Ok(SpannedToken { token, span }),
        Err(()) => Err(LexError {
            text: lexer.slice().to_string(),
            span,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::PredefinedSymbol;

    fn parse(input: &str) -> Result<Vec<Instruction>, ParseErrorKind> {
        Parser::new(input).parse_all().map_err(|error| error.kind)
    }

    #[test]
    fn reports_unicode_aware_source_spans() {
        let error = Parser::new("// first\r\n  💥").parse_all().unwrap_err();
        assert_eq!(
            error.kind,
            ParseErrorKind::UnexpectedToken("💥".to_string())
        );
        assert_eq!(error.span.start.offset, 12);
        assert_eq!((error.span.start.line, error.span.start.column), (2, 3));
        assert_eq!((error.span.end.line, error.span.end.column), (2, 4));
    }

    #[test]
    fn reports_zero_width_eof_spans() {
        let error = Parser::new("@").parse_all().unwrap_err();
        assert_eq!(error.kind, ParseErrorKind::UnexpectedEnd);
        assert_eq!(error.span, SourceSpan::eof("@"));
        assert_eq!(error.span.byte_range(), 1..1);
    }

    #[test]
    fn parses_a_instructions() {
        assert_eq!(parse("@2"), Ok(vec![Instruction::A(AValue::Number(2))]));
        assert_eq!(
            parse("@R0"),
            Ok(vec![Instruction::A(AValue::Predefined(
                PredefinedSymbol::R0
            ))])
        );
        assert_eq!(
            parse("@R1"),
            Ok(vec![Instruction::A(AValue::Predefined(
                PredefinedSymbol::R1
            ))])
        );
        assert_eq!(
            parse("@R15"),
            Ok(vec![Instruction::A(AValue::Predefined(
                PredefinedSymbol::R15
            ))])
        );
        assert_eq!(
            parse("@SCREEN"),
            Ok(vec![Instruction::A(AValue::Predefined(
                PredefinedSymbol::SCREEN
            ))])
        );
        assert_eq!(
            parse("@i"),
            Ok(vec![Instruction::A(AValue::Symbol("i".to_string()))])
        );
    }

    #[test]
    fn parses_labels() {
        assert_eq!(
            parse("(LOOP)"),
            Ok(vec![Instruction::Label("LOOP".to_string())])
        );
    }

    #[test]
    fn parses_c_instructions() {
        assert_eq!(
            parse("D=A"),
            Ok(vec![Instruction::C {
                dest: Some(Dest::D),
                comp: Comp::A,
                jump: None,
            }])
        );
        assert_eq!(
            parse("M=D+1"),
            Ok(vec![Instruction::C {
                dest: Some(Dest::M),
                comp: Comp::DPlusOne,
                jump: None,
            }])
        );
        assert_eq!(
            parse("0;JMP"),
            Ok(vec![Instruction::C {
                dest: None,
                comp: Comp::Zero,
                jump: Some(Jump::Jmp),
            }])
        );
        assert_eq!(
            parse("D;JGT"),
            Ok(vec![Instruction::C {
                dest: None,
                comp: Comp::D,
                jump: Some(Jump::Jgt),
            }])
        );
        assert_eq!(
            parse("AMD=D|A"),
            Ok(vec![Instruction::C {
                dest: Some(Dest::AMD),
                comp: Comp::DOrA,
                jump: None,
            }])
        );
    }

    #[test]
    fn ignores_comments_and_whitespace() {
        assert_eq!(
            parse("  // comment\n\n  @2  // value\n  D=A\n"),
            Ok(vec![
                Instruction::A(AValue::Number(2)),
                Instruction::C {
                    dest: Some(Dest::D),
                    comp: Comp::A,
                    jump: None,
                }
            ])
        );
    }

    #[test]
    fn accepts_inline_comments_after_c_instructions() {
        assert_eq!(
            parse("D=A// copy\n0;JMP // loop\nD // bare computation\n"),
            Ok(vec![
                Instruction::C {
                    dest: Some(Dest::D),
                    comp: Comp::A,
                    jump: None,
                },
                Instruction::C {
                    dest: None,
                    comp: Comp::Zero,
                    jump: Some(Jump::Jmp),
                },
                Instruction::C {
                    dest: None,
                    comp: Comp::D,
                    jump: None,
                },
            ])
        );
    }

    #[test]
    fn parses_every_c_instruction_component() {
        let destinations = [
            Dest::M,
            Dest::D,
            Dest::MD,
            Dest::A,
            Dest::AM,
            Dest::AD,
            Dest::AMD,
        ];
        for dest in destinations {
            assert_eq!(
                parse(&format!("{dest}=0")),
                Ok(vec![Instruction::C {
                    dest: Some(dest),
                    comp: Comp::Zero,
                    jump: None,
                }])
            );
        }

        let computations = [
            Comp::Zero,
            Comp::One,
            Comp::NegOne,
            Comp::D,
            Comp::A,
            Comp::M,
            Comp::NotD,
            Comp::NotA,
            Comp::NotM,
            Comp::NegD,
            Comp::NegA,
            Comp::NegM,
            Comp::DPlusOne,
            Comp::APlusOne,
            Comp::MPlusOne,
            Comp::DMinusOne,
            Comp::AMinusOne,
            Comp::MMinusOne,
            Comp::DPlusA,
            Comp::DPlusM,
            Comp::DMinusA,
            Comp::DMinusM,
            Comp::AMinusD,
            Comp::MMinusD,
            Comp::DAndA,
            Comp::DAndM,
            Comp::DOrA,
            Comp::DOrM,
        ];
        for comp in computations {
            assert_eq!(
                parse(&comp.to_string()),
                Ok(vec![Instruction::C {
                    dest: None,
                    comp,
                    jump: None,
                }])
            );
        }

        let jumps = [
            Jump::Jgt,
            Jump::Jeq,
            Jump::Jge,
            Jump::Jlt,
            Jump::Jne,
            Jump::Jle,
            Jump::Jmp,
        ];
        for jump in jumps {
            assert_eq!(
                parse(&format!("0;{jump}")),
                Ok(vec![Instruction::C {
                    dest: None,
                    comp: Comp::Zero,
                    jump: Some(jump),
                }])
            );
        }
    }

    #[test]
    fn rejects_invalid_a_instructions() {
        assert_eq!(parse("@"), Err(ParseErrorKind::UnexpectedEnd));
        assert_eq!(
            parse("@32768"),
            Err(ParseErrorKind::InvalidNumber("32768".to_string()))
        );
        assert_eq!(
            parse("@2bad"),
            Err(ParseErrorKind::InvalidSymbol("2bad".to_string()))
        );
        assert_eq!(
            parse("@2?"),
            Err(ParseErrorKind::UnexpectedToken("?".to_string()))
        );
    }

    #[test]
    fn rejects_invalid_labels() {
        assert_eq!(
            parse("()"),
            Err(ParseErrorKind::UnexpectedToken(")".to_string()))
        );
        assert_eq!(parse("(LOOP"), Err(ParseErrorKind::UnexpectedEnd));
    }

    #[test]
    fn rejects_invalid_c_instructions() {
        assert_eq!(
            parse("DM=A"),
            Err(ParseErrorKind::InvalidDest("DM".to_string()))
        );
        assert_eq!(
            parse("D++A"),
            Err(ParseErrorKind::InvalidComp("D++A".to_string()))
        );
        assert_eq!(
            parse("D;JNOPE"),
            Err(ParseErrorKind::InvalidJump("JNOPE".to_string()))
        );
        assert_eq!(
            parse("D=M;JGT"),
            Err(ParseErrorKind::UnexpectedToken(";".to_string()))
        );
        assert_eq!(
            parse("D;JMP=0"),
            Err(ParseErrorKind::UnexpectedToken("=".to_string()))
        );
        assert_eq!(
            parse("D=;JMP"),
            Err(ParseErrorKind::InvalidComp("".to_string()))
        );
        assert_eq!(
            parse("@2 D=A"),
            Err(ParseErrorKind::UnexpectedToken("D".to_string()))
        );
        assert_eq!(
            parse("D=A?"),
            Err(ParseErrorKind::UnexpectedToken("?".to_string()))
        );
        assert_eq!(
            parse("?"),
            Err(ParseErrorKind::UnexpectedToken("?".to_string()))
        );
    }
}
