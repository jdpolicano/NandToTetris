use crate::instruction::{
    AValue, Comp, Dest, Instruction, InstructionError, Jump, predefined_symbol, validate_symbol,
};
use crate::token::Token;
use logos::{Lexer, Logos};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
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

pub struct Parser<'a> {
    tokenizer: Lexer<'a, Token<'a>>,
    current: Option<Result<Token<'a>, ()>>,
}

impl<'a> Parser<'a> {
    /// Creates a parser over the provided Hack assembly source.
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Token::lexer(input);
        let current = tokenizer.next();

        Self { tokenizer, current }
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
            Some(Token::At) => self.parse_a_instruction(),
            Some(Token::OpenParen) => self.parse_label(),
            Some(token) => self.parse_c_instruction(token),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_a_instruction(&mut self) -> Result<Instruction, ParseError> {
        let value = match self.advance()? {
            Some(Token::Number(value)) => parse_a_number(value)?,
            Some(Token::Identifier(value)) => parse_a_symbol(value)?,
            Some(token) => return Err(ParseError::UnexpectedToken(token_to_string(&token))),
            None => return Err(ParseError::UnexpectedEnd),
        };

        self.expect_newline_or_end()?;
        Ok(Instruction::A(value))
    }

    fn parse_label(&mut self) -> Result<Instruction, ParseError> {
        let label = match self.advance()? {
            Some(Token::Identifier(value)) => value,
            Some(token) => return Err(ParseError::UnexpectedToken(token_to_string(&token))),
            None => return Err(ParseError::UnexpectedEnd),
        };

        self.expect(Token::CloseParen)?;
        self.expect_newline_or_end()?;
        validate_symbol(label).map_err(invalid_symbol)?;
        Ok(Instruction::Label(label.to_string()))
    }

    fn parse_c_instruction(&mut self, first_token: Token<'a>) -> Result<Instruction, ParseError> {
        let first = self.read_component_until_control(first_token)?;

        match self.peek()? {
            Some(Token::Eq) => self.parse_assignment(&first),
            Some(Token::SemiColon) => self.parse_jump_instruction(&first),
            Some(Token::Newline) | None => self.parse_bare_comp(&first),
            Some(token) => Err(ParseError::UnexpectedToken(token_to_string(token))),
        }
    }

    fn parse_assignment(&mut self, dest_tokens: &[Token<'a>]) -> Result<Instruction, ParseError> {
        let dest = parse_dest(dest_tokens)?;
        self.expect(Token::Eq)?;

        let comp_tokens = self.read_remaining_component_until_control()?;
        let comp = parse_comp(&comp_tokens)?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: Some(dest),
            comp,
            jump: None,
        })
    }

    fn parse_jump_instruction(
        &mut self,
        comp_tokens: &[Token<'a>],
    ) -> Result<Instruction, ParseError> {
        let comp = parse_comp(comp_tokens)?;
        self.expect(Token::SemiColon)?;

        let jump_tokens = self.read_remaining_component_until_control()?;
        let jump = parse_jump(&jump_tokens)?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: None,
            comp,
            jump: Some(jump),
        })
    }

    fn parse_bare_comp(&mut self, comp_tokens: &[Token<'a>]) -> Result<Instruction, ParseError> {
        let comp = parse_comp(comp_tokens)?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: None,
            comp,
            jump: None,
        })
    }

    fn read_component_until_control(
        &mut self,
        first_token: Token<'a>,
    ) -> Result<Vec<Token<'a>>, ParseError> {
        let mut value = vec![first_token];

        self.read_remaining_component_into(&mut value)?;
        Ok(value)
    }

    /// Reads a C-instruction component after a control token such as `=` or `;`
    /// has already been consumed.
    fn read_remaining_component_until_control(&mut self) -> Result<Vec<Token<'a>>, ParseError> {
        let mut value = Vec::new();
        self.read_remaining_component_into(&mut value)?;
        Ok(value)
    }

    /// Appends component tokens until a token that separates C-instruction
    /// fields is reached. Semantic validity is still checked later by
    /// `Dest`, `Comp`, and `Jump`.
    fn read_remaining_component_into(
        &mut self,
        value: &mut Vec<Token<'a>>,
    ) -> Result<(), ParseError> {
        loop {
            match self.peek()? {
                Some(Token::Eq | Token::SemiColon | Token::Newline) | None => {
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
            Some(token) if same_token_variant(&token, &expected) => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken(token_to_string(&token))),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn expect_newline_or_end(&mut self) -> Result<(), ParseError> {
        match self.advance()? {
            Some(Token::Newline) | None => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken(token_to_string(&token))),
        }
    }

    fn skip_to_valid_position(&mut self) -> Result<(), ParseError> {
        if self.current.as_ref().is_some_and(Result::is_err) {
            return Err(ParseError::UnexpectedToken(
                self.tokenizer.slice().to_string(),
            ));
        }
        while matches!(self.current.as_ref(), Some(Ok(Token::Newline))) {
            self.advance()?;
        }
        Ok(())
    }

    /// Consumes and returns the current token, then advances one token ahead.
    fn advance(&mut self) -> Result<Option<Token<'a>>, ParseError> {
        let token = self.current.take();
        if token.as_ref().is_some_and(Result::is_err) {
            return Err(ParseError::UnexpectedToken(
                self.tokenizer.slice().to_string(),
            ));
        }
        self.current = self.tokenizer.next();
        match token {
            Some(Ok(token)) => Ok(Some(token)),
            None => Ok(None),
            Some(Err(())) => unreachable!("lexer errors return before advancing"),
        }
    }

    fn peek(&self) -> Result<Option<&Token<'a>>, ParseError> {
        match self.current.as_ref() {
            Some(Ok(token)) => Ok(Some(token)),
            None => Ok(None),
            Some(Err(())) => Err(ParseError::UnexpectedToken(
                self.tokenizer.slice().to_string(),
            )),
        }
    }
}

impl TryFrom<&str> for Dest {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "M" => Ok(Self::M),
            "D" => Ok(Self::D),
            "MD" => Ok(Self::MD),
            "A" => Ok(Self::A),
            "AM" => Ok(Self::AM),
            "AD" => Ok(Self::AD),
            "AMD" => Ok(Self::AMD),
            _ => Err(ParseError::InvalidDest(value.to_string())),
        }
    }
}

impl TryFrom<&str> for Comp {
    type Error = ParseError;

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
            _ => Err(ParseError::InvalidComp(value.to_string())),
        }
    }
}

impl TryFrom<&str> for Jump {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "JGT" => Ok(Self::Jgt),
            "JEQ" => Ok(Self::Jeq),
            "JGE" => Ok(Self::Jge),
            "JLT" => Ok(Self::Jlt),
            "JNE" => Ok(Self::Jne),
            "JLE" => Ok(Self::Jle),
            "JMP" => Ok(Self::Jmp),
            _ => Err(ParseError::InvalidJump(value.to_string())),
        }
    }
}

fn parse_a_number(value: &str) -> Result<AValue, ParseError> {
    let number = value
        .parse::<u16>()
        .map_err(|_| ParseError::InvalidNumber(value.to_string()))?;

    if number > 32_767 {
        return Err(ParseError::InvalidNumber(value.to_string()));
    }

    Ok(AValue::Number(number))
}

fn parse_a_symbol(value: &str) -> Result<AValue, ParseError> {
    if let Some(symbol) = predefined_symbol(value) {
        return Ok(AValue::Predefined(symbol));
    }

    validate_symbol(value).map_err(invalid_symbol)?;
    Ok(AValue::Symbol(value.to_string()))
}

fn parse_dest(tokens: &[Token<'_>]) -> Result<Dest, ParseError> {
    match tokens {
        [Token::Identifier("M")] => Ok(Dest::M),
        [Token::Identifier("D")] => Ok(Dest::D),
        [Token::Identifier("MD")] => Ok(Dest::MD),
        [Token::Identifier("A")] => Ok(Dest::A),
        [Token::Identifier("AM")] => Ok(Dest::AM),
        [Token::Identifier("AD")] => Ok(Dest::AD),
        [Token::Identifier("AMD")] => Ok(Dest::AMD),
        _ => Err(ParseError::InvalidDest(component_text(tokens))),
    }
}

fn parse_comp(tokens: &[Token<'_>]) -> Result<Comp, ParseError> {
    let comp = match tokens {
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
        _ => return Err(ParseError::InvalidComp(component_text(tokens))),
    };

    Ok(comp)
}

fn parse_jump(tokens: &[Token<'_>]) -> Result<Jump, ParseError> {
    match tokens {
        [Token::Identifier("JGT")] => Ok(Jump::Jgt),
        [Token::Identifier("JEQ")] => Ok(Jump::Jeq),
        [Token::Identifier("JGE")] => Ok(Jump::Jge),
        [Token::Identifier("JLT")] => Ok(Jump::Jlt),
        [Token::Identifier("JNE")] => Ok(Jump::Jne),
        [Token::Identifier("JLE")] => Ok(Jump::Jle),
        [Token::Identifier("JMP")] => Ok(Jump::Jmp),
        _ => Err(ParseError::InvalidJump(component_text(tokens))),
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

fn component_text(tokens: &[Token<'_>]) -> String {
    tokens.iter().map(token_to_string).collect()
}

fn same_token_variant(left: &Token<'_>, right: &Token<'_>) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn invalid_symbol(error: InstructionError) -> ParseError {
    match error {
        InstructionError::InvalidSymbol(value) => ParseError::InvalidSymbol(value),
        InstructionError::AddressOutOfRange(_) => {
            unreachable!("symbol validation cannot produce an address error")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::PredefinedSymbol;

    fn parse(input: &str) -> Result<Vec<Instruction>, ParseError> {
        Parser::new(input).parse_all()
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
        assert_eq!(parse("@"), Err(ParseError::UnexpectedEnd));
        assert_eq!(
            parse("@32768"),
            Err(ParseError::InvalidNumber("32768".to_string()))
        );
        assert_eq!(
            parse("@2bad"),
            Err(ParseError::InvalidSymbol("2bad".to_string()))
        );
        assert_eq!(
            parse("@2?"),
            Err(ParseError::UnexpectedToken("?".to_string()))
        );
    }

    #[test]
    fn rejects_invalid_labels() {
        assert_eq!(
            parse("()"),
            Err(ParseError::UnexpectedToken(")".to_string()))
        );
        assert_eq!(parse("(LOOP"), Err(ParseError::UnexpectedEnd));
    }

    #[test]
    fn rejects_invalid_c_instructions() {
        assert_eq!(
            parse("DM=A"),
            Err(ParseError::InvalidDest("DM".to_string()))
        );
        assert_eq!(
            parse("D++A"),
            Err(ParseError::InvalidComp("D++A".to_string()))
        );
        assert_eq!(
            parse("D;JNOPE"),
            Err(ParseError::InvalidJump("JNOPE".to_string()))
        );
        assert_eq!(
            parse("D=M;JGT"),
            Err(ParseError::UnexpectedToken(";".to_string()))
        );
        assert_eq!(
            parse("D;JMP=0"),
            Err(ParseError::UnexpectedToken("=".to_string()))
        );
        assert_eq!(
            parse("D=;JMP"),
            Err(ParseError::InvalidComp("".to_string()))
        );
        assert_eq!(
            parse("@2 D=A"),
            Err(ParseError::UnexpectedToken("D".to_string()))
        );
        assert_eq!(
            parse("D=A?"),
            Err(ParseError::UnexpectedToken("?".to_string()))
        );
        assert_eq!(
            parse("?"),
            Err(ParseError::UnexpectedToken("?".to_string()))
        );
    }
}
