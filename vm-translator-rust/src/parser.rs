use crate::token::{Token, TokenType, Tokenizer};

const MAX_ADDRESS: u16 = 2_u16.pow(15) - 1;

#[derive(Debug, PartialEq)]
pub enum VmCommand<'a> {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Push { segment: &'a str, index: u16 },
    Pop { segment: &'a str, index: u16 },
}

pub struct VmParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
}

impl<'a> VmParser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokens = Tokenizer::new(source).take_tokens();
        Self {
            tokens,
            source,
            pos: 0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn next_command(&mut self) -> Result<VmCommand, String> {
        let _ = self.skip_unnecessary_tokens();
        let toke = self.next_token()?;
        self.match_command(toke)
    }

    pub fn match_command(&mut self, command_toke: Token) -> Result<VmCommand, String> {
        match &self.source[command_toke.start..command_toke.end] {
            "add" => Ok(VmCommand::Add),
            "sub" => Ok(VmCommand::Sub),
            "neg" => Ok(VmCommand::Neg),
            "eq" => Ok(VmCommand::Eq),
            "gt" => Ok(VmCommand::Gt),
            "lt" => Ok(VmCommand::Lt),
            "and" => Ok(VmCommand::And),
            "or" => Ok(VmCommand::Or),
            "not" => Ok(VmCommand::Not),
            "push" => return self.match_push(),
            "pop" => return self.match_pop(),
            other => Err(format!("Invalid command: {}", other)),
        }
    }

    fn match_push(&mut self) -> Result<VmCommand, String> {
        let segment = self.match_segment()?;
        let index = self.match_index()?;
        Ok(VmCommand::Push { segment, index })
    }

    fn match_pop(&mut self) -> Result<VmCommand, String> {
        let segment = self.match_segment()?;
        let index = self.match_index()?;
        Ok(VmCommand::Pop { segment, index })
    }

    fn match_segment(&mut self) -> Result<&'a str, String> {
        let _ = self.skip_while(|t| t.token_type == TokenType::WhiteSpace);
        let segment_toke = self.next_token()?;
        let segment = &self.source[segment_toke.start..segment_toke.end];
        match segment {
            "argument" | "local" | "static" | "constant" | "this" | "that" | "pointer" | "temp" => {
                Ok(segment)
            }
            other => Err(format!("Invalid segment: {:?}", other)),
        }
    }

    fn match_index(&mut self) -> Result<u16, String> {
        let _ = self.skip_while(|t| t.token_type == TokenType::WhiteSpace);
        let index_toke = self.next_token()?;
        let index = &self.source[index_toke.start..index_toke.end];
        match index.parse::<u16>() {
            Ok(i) => {
                if i > MAX_ADDRESS {
                    Err(format!("Index out of bounds: {}", i))
                } else {
                    Ok(i)
                }
            }
            Err(e) => Err(format!("Invalid index: {}", e)),
        }
    }

    fn skip_while<F>(&mut self, f: F)
    where
        F: Fn(&Token) -> bool,
    {
        while !self.is_done() && f(&self.tokens[self.pos]) {
            self.pos += 1;
        }
    }

    fn skip_unnecessary_tokens(&mut self) {
        self.skip_while(|t| {
            t.token_type == TokenType::WhiteSpace
                || t.token_type == TokenType::Comment
                || t.token_type == TokenType::Newline
        });
    }

    fn next_token(&mut self) -> Result<Token, String> {
        if self.is_done() {
            return Err("Unexpected end of file".to_string());
        }
        self.tokens
            .get(self.pos)
            .map_or(Err("Unexpected end of file".to_string()), |t| {
                self.pos += 1;
                Ok(t.clone())
            })
    }
}

#[cfg(test)]

mod unit {
    use super::*;

    #[test]
    fn test_parser() {
        let source = "push constant 7\npush constant 8\nadd\n";
        let mut parser = VmParser::new(source);
        let command = parser.next_command().unwrap();
        assert_eq!(
            command,
            VmCommand::Push {
                segment: "constant",
                index: 7
            }
        );
        let command = parser.next_command().unwrap();
        assert_eq!(
            command,
            VmCommand::Push {
                segment: "constant",
                index: 8
            }
        );
        let command = parser.next_command().unwrap();
        assert_eq!(command, VmCommand::Add);
    }

    #[test]
    fn test_parser_error() {
        let source = "push constant 7\npush constant 8\nadd\n";
        let mut parser = VmParser::new(source);
        let _ = parser.next_command().unwrap();
        let _ = parser.next_command().unwrap();
        let _ = parser.next_command().unwrap();
        let result = parser.next_command();
        assert_eq!(result, Err("Unexpected end of file".to_string()));
    }

    #[test]
    fn test_parser_error_invalid_command() {
        let source = "push constant 7\nxor constant 8\nadd\n";
        let mut parser = VmParser::new(source);
        let _ = parser.next_command().unwrap();
        let result = parser.next_command();
        assert_eq!(result, Err("Invalid command: xor".to_string()));
    }
}
