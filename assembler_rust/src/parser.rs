use crate::token::{Token, TokenType, Tokenizer};
use std::cell::RefCell;

const SYMBOL_SP_CHARS: [char; 4] = ['_', '.', '$', ':'];
const MAX_NUMERIC_CONSTANT: u16 = 2u16.pow(15) - 1;

#[derive(Debug, PartialEq, Eq)]
pub enum Address {
    NumericConstant(u16),
    Symbol(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    AInstruction(Address),
    CInstruction {
        dest: Option<String>,
        comp: String,
        jump: Option<String>,
    },
    Label(String),
}

impl Instruction {
    pub fn is_address(&self) -> bool {
        matches!(self, Instruction::AInstruction(_))
    }

    pub fn is_c_instruction(&self) -> bool {
        matches!(self, Instruction::CInstruction { .. })
    }

    pub fn is_label(&self) -> bool {
        matches!(self, Instruction::Label(_))
    }

    pub fn is_assignment(&self) -> bool {
        matches!(self, Instruction::CInstruction { dest: Some(_), .. })
    }

    pub fn is_jump(&self) -> bool {
        matches!(self, Instruction::CInstruction { jump: Some(_), .. })
    }
}

pub struct Parser<'a> {
    tokenizer: RefCell<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            tokenizer: RefCell::new(Tokenizer::new(src)),
        }
    }

    pub fn reset(&mut self) {
        self.tokenizer.borrow_mut().restore();
    }

    pub fn next_instruction(&self) -> Result<Option<Instruction>, String> {
        self.skip_to_next_instruction();

        if self.is_done() {
            return Ok(None);
        }

        let next_token = self.take_token()?;
        match next_token.get_type() {
            TokenType::Address => self.parse_a_instruction(),
            TokenType::OpenParens => self.parse_label(),
            TokenType::Text => self.parse_c_instruction(next_token),
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn parse_a_instruction(&self) -> Result<Option<Instruction>, String> {
        let address = self.expect_read(TokenType::Text)?;

        if let Ok(num) = address.parse::<u16>() {
            if num > MAX_NUMERIC_CONSTANT {
                return Err("numeric constant exceeds hack maximum numeric constant".to_string());
            }
            return Ok(Some(Instruction::AInstruction(Address::NumericConstant(
                num,
            ))));
        }

        let _ = self.validate_symbol(&address)?;
        let _ = self.peek_token().map_or(Ok(()), |t| {
            t.expect_one_of(&[
                TokenType::WhiteSpace,
                TokenType::Newline,
                TokenType::Comment,
            ])
            .map(|_| ())
        })?;
        Ok(Some(Instruction::AInstruction(Address::Symbol(address))))
    }

    fn parse_label(&self) -> Result<Option<Instruction>, String> {
        // expect a string token.
        let label = self.expect_read(TokenType::Text)?;
        // validate the label's syntax
        let _ = self.validate_symbol(&label)?;
        // expect a closing parenthesis
        let _ = self
            .take_token()
            .and_then(|t| t.expect_type(TokenType::CloseParens))?;
        Ok(Some(Instruction::Label(label)))
    }

    fn parse_c_instruction(&self, text_token: Token) -> Result<Option<Instruction>, String> {
        let dest_or_comp = self.read_token(text_token);
        let next_token = self.take_token()?;
        match next_token.get_type() {
            TokenType::Eq => {
                let dest = Some(dest_or_comp);
                let comp = self.expect_read(TokenType::Text)?;
                let jump = None;
                Ok(Some(Instruction::CInstruction { dest, comp, jump }))
            }

            TokenType::Comp => {
                let dest = None;
                let comp = dest_or_comp;
                let jump = Some(self.expect_read(TokenType::Text)?);
                Ok(Some(Instruction::CInstruction { dest, comp, jump }))
            }

            _ => Err(format!(
                "ctype expected '=' || ';', got {:?}",
                self.tokenizer.borrow().read_token(next_token)
            )),
        }
    }

    fn validate_symbol(&self, label: &str) -> Result<(), String> {
        if label.is_empty() {
            return Err("empty label".to_string());
        }

        if !label
            .chars()
            .all(|c| c.is_alphanumeric() || SYMBOL_SP_CHARS.contains(&c))
        {
            return Err("invalid label".to_string());
        }

        // must not begin with a digit
        if label.chars().next().unwrap().is_digit(10) {
            return Err("label cannot begin with a digit".to_string());
        }

        Ok(())
    }

    fn skip_to_next_instruction(&self) {
        self.skip_while(|toke| match toke.get_type() {
            TokenType::WhiteSpace | TokenType::Newline | TokenType::Comment => true,
            _ => false,
        })
    }

    fn skip_while<T>(&self, f: T)
    where
        T: Fn(&Token) -> bool,
    {
        let mut tokenizer = self.tokenizer.borrow_mut();
        while let Some(toke) = tokenizer.peek_token() {
            if f(&toke) {
                tokenizer.next_token();
            } else {
                return;
            }
        }
    }

    fn take_token(&self) -> Result<Token, String> {
        self.tokenizer
            .borrow_mut()
            .next_token()
            .ok_or_else(|| "end of stream".to_string())
    }

    fn read_token(&self, t: Token) -> String {
        self.tokenizer.borrow().read_token(t).to_string()
    }

    fn expect_read(&self, t: TokenType) -> Result<String, String> {
        self.take_token()
            .and_then(|toke| toke.expect_type(t))
            .map(|t| self.read_token(t))
    }

    fn peek_token(&self) -> Option<Token> {
        self.tokenizer.borrow().peek_token()
    }

    fn is_done(&self) -> bool {
        self.tokenizer.borrow().is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::{Address, Instruction, Parser};

    #[test]
    fn test_parser_address() {
        let src = "@1234";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::AInstruction(Address::NumericConstant(
                1234
            ))))
        );
    }

    #[test]
    fn test_parser_address_skipping_space() {
        let src = " @1234 ";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::AInstruction(Address::NumericConstant(
                1234
            ))))
        );
    }

    #[test]
    fn test_parser_address_symbol() {
        let src = "@LOOP";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::AInstruction(Address::Symbol(
                "LOOP".to_string()
            ))))
        );
    }

    #[test]
    fn test_parser_address_symbol_trailing_space() {
        let src = "@LOOP  ";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::AInstruction(Address::Symbol(
                "LOOP".to_string()
            ))))
        );
    }

    #[test]
    fn test_parser_address_symbol_all_sp_chars() {
        let src = "@L.O_O$P:";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::AInstruction(Address::Symbol(
                "L.O_O$P:".to_string()
            ))))
        );
    }

    #[test]
    fn test_parser_address_symbol_invalid_sp_chars() {
        let invalid_chars = vec![
            '(', '@', '#', '%', '^', '&', '*', '-', '+', '=', '{', '}', '[', ']', '|', '\\', ';',
            '"', '\'', '<', '>', ',', '?', '/',
        ];

        for c in invalid_chars {
            let src = format!("@loop{}", c);
            let parser = Parser::new(&src);
            let instruction1 = parser.next_instruction();
            assert!(
                instruction1.is_err(),
                "expected error for char '{}' but received {:?}",
                c,
                instruction1
            );
        }
    }

    #[test]
    fn test_parser_label() {
        let src = "(LOOP)";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::Label(src[1..src.len() - 1].to_string())))
        );
    }

    #[test]
    fn test_parser_label_trailing_space() {
        let src = "(LOOP)  ";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::Label("LOOP".to_string())))
        );
    }

    #[test]
    fn test_parser_label_all_sp_chars() {
        let src = "(L.O_O$P:)";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert_eq!(
            instruction1,
            Ok(Some(Instruction::Label(src[1..src.len() - 1].to_string())))
        );
    }

    #[test]
    fn test_parser_label_invalid_sp_chars() {
        let invalid_chars = vec![
            '(', '@', '#', '%', '^', '&', '*', '-', '+', '=', '{', '}', '[', ']', '|', '\\', ';',
            '"', '\'', '<', '>', ',', '?', '/',
        ];
        for c in invalid_chars {
            let src = format!("(LOOP{})", c);
            let parser = Parser::new(&src);
            let instruction = parser.next_instruction();
            assert!(
                instruction.is_err(),
                "expected error matching {} but received {:?}",
                src,
                instruction
            );
        }
    }

    #[test]
    fn test_parser_label_syntax_err_spaceing() {
        let src = "( LOOP)";
        let parser = Parser::new(src);
        let instruction1 = parser.next_instruction();
        assert!(
            instruction1.is_err(),
            "expected error matching {} but received {:?}",
            src,
            instruction1
        );

        let src2 = "(LOOP )";
        let parser2 = Parser::new(src2);
        let instruction2 = parser2.next_instruction();
        assert!(
            instruction2.is_err(),
            "expected error matching {} but received {:?}",
            src2,
            instruction2
        );

        let src3 = "(LOOP";
        let parser3 = Parser::new(src3);
        let instruction3 = parser3.next_instruction();
        assert!(
            instruction3.is_err(),
            "expected error matching {} but received {:?}",
            src3,
            instruction3
        );

        let src4 = "(LO OP)";
        let parser4 = Parser::new(src4);
        let instruction4 = parser4.next_instruction();
        assert!(
            instruction4.is_err(),
            "expected error matching {} but received {:?}",
            src4,
            instruction4
        );
    }
}
