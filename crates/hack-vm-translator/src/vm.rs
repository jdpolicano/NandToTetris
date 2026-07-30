use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

impl Display for Arithmetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Neg => "neg",
            Self::Eq => "eq",
            Self::Gt => "gt",
            Self::Lt => "lt",
            Self::And => "and",
            Self::Or => "or",
            Self::Not => "not",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Local,
    Argument,
    This,
    That,
    Pointer,
    Temp,
    Constant,
    Static,
}

impl Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Local => "local",
            Self::Argument => "argument",
            Self::This => "this",
            Self::That => "that",
            Self::Pointer => "pointer",
            Self::Temp => "temp",
            Self::Constant => "constant",
            Self::Static => "static",
        };

        f.write_str(value)
    }
}
