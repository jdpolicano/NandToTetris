use crate::instruction::{
    AValue, Comp, Dest, Instruction, InstructionError, Jump, predefined_symbol, validate_symbol,
};
use crate::token::{Token, Tokenizer};
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
    tokenizer: Tokenizer<'a>,
    current: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a parser over the provided Hack assembly source.
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Tokenizer::new(input);
        let current = tokenizer.next();

        Self { tokenizer, current }
    }

    /// Parses the full input into instructions, skipping blank lines.
    pub fn parse_all(&mut self) -> Result<Vec<Instruction>, ParseError> {
        let mut instructions = Vec::new();

        self.skip_blank_lines();
        while self.current.is_some() {
            instructions.push(self.parse_instruction()?);
            self.skip_blank_lines();
        }

        Ok(instructions)
    }

    /// Consumes the first token of an instruction and dispatches to the
    /// grammar-specific parser for the rest of that instruction.
    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        match self.advance() {
            Some(Token::At) => self.parse_a_instruction(),
            Some(Token::OpenParen) => self.parse_label(),
            Some(token) => self.parse_c_instruction(token),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_a_instruction(&mut self) -> Result<Instruction, ParseError> {
        let value = match self.advance() {
            Some(Token::Number(value)) => parse_a_number(value)?,
            Some(Token::Identifier(value)) => parse_a_symbol(value)?,
            Some(token) => return Err(ParseError::UnexpectedToken(token_to_string(&token))),
            None => return Err(ParseError::UnexpectedEnd),
        };

        self.expect_newline_or_end()?;
        Ok(Instruction::A(value))
    }

    fn parse_label(&mut self) -> Result<Instruction, ParseError> {
        let label = match self.advance() {
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

        match self.peek() {
            Some(Token::Eq) => self.parse_assignment(first),
            Some(Token::SemiColon) => self.parse_jump_instruction(first),
            Some(Token::Newline) | None => self.parse_bare_comp(first),
            Some(token) => Err(ParseError::UnexpectedToken(token_to_string(token))),
        }
    }

    fn parse_assignment(&mut self, dest_text: String) -> Result<Instruction, ParseError> {
        let dest = Dest::try_from(dest_text.as_str())?;
        self.expect(Token::Eq)?;

        let comp = parse_comp(self.read_remaining_component_until_control()?)?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: Some(dest),
            comp,
            jump: None,
        })
    }

    fn parse_jump_instruction(&mut self, comp_text: String) -> Result<Instruction, ParseError> {
        let comp = parse_comp(comp_text)?;
        self.expect(Token::SemiColon)?;

        let jump = parse_jump(self.read_remaining_component_until_control()?)?;
        self.expect_newline_or_end()?;

        Ok(Instruction::C {
            dest: None,
            comp,
            jump: Some(jump),
        })
    }

    fn parse_bare_comp(&mut self, comp_text: String) -> Result<Instruction, ParseError> {
        let comp = parse_comp(comp_text)?;
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
    ) -> Result<String, ParseError> {
        let mut value = String::new();
        push_component_token(&mut value, &first_token)?;

        self.read_remaining_component_into(&mut value)?;
        Ok(value)
    }

    /// Reads a C-instruction component after a control token such as `=` or `;`
    /// has already been consumed.
    fn read_remaining_component_until_control(&mut self) -> Result<String, ParseError> {
        let mut value = String::new();
        self.read_remaining_component_into(&mut value)?;
        Ok(value)
    }

    /// Appends component tokens until a token that separates C-instruction
    /// fields is reached. Semantic validity is still checked later by
    /// `Dest`, `Comp`, and `Jump`.
    fn read_remaining_component_into(&mut self, value: &mut String) -> Result<(), ParseError> {
        while !matches!(
            self.peek(),
            Some(Token::Eq | Token::SemiColon | Token::Newline) | None
        ) {
            let token = self
                .advance()
                .expect("peek confirmed a current token exists");
            push_component_token(value, &token)?;
        }

        Ok(())
    }

    /// Consumes the current token if it is the expected terminal variant.
    fn expect(&mut self, expected: Token<'_>) -> Result<(), ParseError> {
        match self.advance() {
            Some(token) if same_token_variant(&token, &expected) => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken(token_to_string(&token))),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn expect_newline_or_end(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Newline) | None => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken(token_to_string(&token))),
        }
    }

    fn skip_blank_lines(&mut self) {
        while matches!(self.current.as_ref(), Some(Token::Newline)) {
            self.advance();
        }
    }

    /// Consumes and returns the current token, then advances one token ahead.
    fn advance(&mut self) -> Option<Token<'a>> {
        let token = self.current.take();
        self.current = self.tokenizer.next();
        token
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.current.as_ref()
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

fn parse_comp(value: String) -> Result<Comp, ParseError> {
    if value.is_empty() {
        return Err(ParseError::InvalidComp(value));
    }

    Comp::try_from(value.as_str())
}

fn parse_jump(value: String) -> Result<Jump, ParseError> {
    if value.is_empty() {
        return Err(ParseError::InvalidJump(value));
    }

    Jump::try_from(value.as_str())
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
        Token::Newline => "\\n".to_string(),
    }
}

fn push_component_token(value: &mut String, token: &Token<'_>) -> Result<(), ParseError> {
    match token {
        Token::Number(_)
        | Token::Identifier(_)
        | Token::Plus
        | Token::Minus
        | Token::Amp
        | Token::Pipe => {
            value.push_str(&token_to_string(token));
            Ok(())
        }
        token => Err(ParseError::UnexpectedToken(token_to_string(token))),
    }
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
    }
}
