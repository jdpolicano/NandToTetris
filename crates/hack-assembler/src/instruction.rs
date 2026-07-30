use std::fmt::{self, Display, Formatter};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    A(AValue),
    C {
        dest: Option<Dest>,
        comp: Comp,
        jump: Option<Jump>,
    },
    Label(String),
    Comment(String),
}

impl From<PredefinedSymbol> for Instruction {
    fn from(symbol: PredefinedSymbol) -> Self {
        AValue::from(symbol).into()
    }
}

impl From<AValue> for Instruction {
    fn from(value: AValue) -> Self {
        Self::A(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InstructionError {
    #[error("A-instruction address `{0}` is outside the range 0..=32767")]
    AddressOutOfRange(u16),
    #[error("invalid symbol `{0}`")]
    InvalidSymbol(String),
}

impl Instruction {
    pub fn a_number(value: u16) -> Result<Self, InstructionError> {
        if value <= 32_767 {
            Ok(Self::A(AValue::Number(value)))
        } else {
            Err(InstructionError::AddressOutOfRange(value))
        }
    }

    pub fn a_symbol(value: impl Into<String>) -> Result<Self, InstructionError> {
        let value = value.into();
        validate_symbol(&value)?;
        Ok(Self::A(match PredefinedSymbol::from_name(&value) {
            Some(symbol) => AValue::Predefined(symbol),
            None => AValue::Symbol(value),
        }))
    }

    pub fn a_predefined(symbol: PredefinedSymbol) -> Self {
        symbol.into()
    }

    pub fn label(value: impl Into<String>) -> Result<Self, InstructionError> {
        let value = value.into();
        validate_symbol(&value)?;
        Ok(Self::Label(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AValue {
    Number(u16),
    Predefined(PredefinedSymbol),
    Symbol(String),
}

impl From<PredefinedSymbol> for AValue {
    fn from(symbol: PredefinedSymbol) -> Self {
        Self::Predefined(symbol)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredefinedSymbol {
    SP,
    LCL,
    ARG,
    THIS,
    THAT,
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    SCREEN,
    KEYBOARD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dest {
    M,
    D,
    MD,
    A,
    AM,
    AD,
    AMD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comp {
    Zero,
    One,
    NegOne,
    D,
    A,
    M,
    NotD,
    NotA,
    NotM,
    NegD,
    NegA,
    NegM,
    DPlusOne,
    APlusOne,
    MPlusOne,
    DMinusOne,
    AMinusOne,
    MMinusOne,
    DPlusA,
    DPlusM,
    DMinusA,
    DMinusM,
    AMinusD,
    MMinusD,
    DAndA,
    DAndM,
    DOrA,
    DOrM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jump {
    Jgt,
    Jeq,
    Jge,
    Jlt,
    Jne,
    Jle,
    Jmp,
}

pub(crate) fn validate_symbol(value: &str) -> Result<(), InstructionError> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(InstructionError::InvalidSymbol(value.to_string()));
    };

    if !is_symbol_start(first) || !chars.all(is_symbol_char) {
        return Err(InstructionError::InvalidSymbol(value.to_string()));
    }

    Ok(())
}

impl PredefinedSymbol {
    pub const fn address(self) -> u16 {
        match self {
            Self::SP | Self::R0 => 0,
            Self::LCL | Self::R1 => 1,
            Self::ARG | Self::R2 => 2,
            Self::THIS | Self::R3 => 3,
            Self::THAT | Self::R4 => 4,
            Self::R5 => 5,
            Self::R6 => 6,
            Self::R7 => 7,
            Self::R8 => 8,
            Self::R9 => 9,
            Self::R10 => 10,
            Self::R11 => 11,
            Self::R12 => 12,
            Self::R13 => 13,
            Self::R14 => 14,
            Self::R15 => 15,
            Self::SCREEN => 16_384,
            Self::KEYBOARD => 24_576,
        }
    }

    pub(crate) fn from_name(value: &str) -> Option<Self> {
        match value {
            "SP" => Some(Self::SP),
            "LCL" => Some(Self::LCL),
            "ARG" => Some(Self::ARG),
            "THIS" => Some(Self::THIS),
            "THAT" => Some(Self::THAT),
            "R0" => Some(Self::R0),
            "R1" => Some(Self::R1),
            "R2" => Some(Self::R2),
            "R3" => Some(Self::R3),
            "R4" => Some(Self::R4),
            "R5" => Some(Self::R5),
            "R6" => Some(Self::R6),
            "R7" => Some(Self::R7),
            "R8" => Some(Self::R8),
            "R9" => Some(Self::R9),
            "R10" => Some(Self::R10),
            "R11" => Some(Self::R11),
            "R12" => Some(Self::R12),
            "R13" => Some(Self::R13),
            "R14" => Some(Self::R14),
            "R15" => Some(Self::R15),
            "SCREEN" => Some(Self::SCREEN),
            "KEYBOARD" => Some(Self::KEYBOARD),
            _ => None,
        }
    }
}

impl Display for PredefinedSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SP => "SP",
            Self::LCL => "LCL",
            Self::ARG => "ARG",
            Self::THIS => "THIS",
            Self::THAT => "THAT",
            Self::R0 => "R0",
            Self::R1 => "R1",
            Self::R2 => "R2",
            Self::R3 => "R3",
            Self::R4 => "R4",
            Self::R5 => "R5",
            Self::R6 => "R6",
            Self::R7 => "R7",
            Self::R8 => "R8",
            Self::R9 => "R9",
            Self::R10 => "R10",
            Self::R11 => "R11",
            Self::R12 => "R12",
            Self::R13 => "R13",
            Self::R14 => "R14",
            Self::R15 => "R15",
            Self::SCREEN => "SCREEN",
            Self::KEYBOARD => "KEYBOARD",
        })
    }
}

pub(crate) fn predefined_symbol(value: &str) -> Option<PredefinedSymbol> {
    PredefinedSymbol::from_name(value)
}

pub(crate) fn predefined_symbol_address(value: &str) -> Option<u16> {
    predefined_symbol(value).map(PredefinedSymbol::address)
}

fn is_symbol_start(value: char) -> bool {
    value.is_ascii_alphabetic() || matches!(value, '_' | '.' | '$' | ':')
}

fn is_symbol_char(value: char) -> bool {
    is_symbol_start(value) || value.is_ascii_digit()
}

impl Display for AValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => write!(f, "{value}"),
            Self::Predefined(value) => write!(f, "{value}"),
            Self::Symbol(value) => f.write_str(value),
        }
    }
}

macro_rules! display_variants {
    ($type:ty, $($variant:ident => $text:literal),+ $(,)?) => {
        impl Display for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str(match self {
                    $(Self::$variant => $text,)+
                })
            }
        }
    };
}

display_variants!(Dest, M => "M", D => "D", MD => "MD", A => "A", AM => "AM", AD => "AD", AMD => "AMD");
display_variants!(
    Comp,
    Zero => "0", One => "1", NegOne => "-1", D => "D", A => "A", M => "M",
    NotD => "!D", NotA => "!A", NotM => "!M", NegD => "-D", NegA => "-A", NegM => "-M",
    DPlusOne => "D+1", APlusOne => "A+1", MPlusOne => "M+1",
    DMinusOne => "D-1", AMinusOne => "A-1", MMinusOne => "M-1",
    DPlusA => "D+A", DPlusM => "D+M", DMinusA => "D-A", DMinusM => "D-M",
    AMinusD => "A-D", MMinusD => "M-D", DAndA => "D&A", DAndM => "D&M",
    DOrA => "D|A", DOrM => "D|M"
);
display_variants!(Jump, Jgt => "JGT", Jeq => "JEQ", Jge => "JGE", Jlt => "JLT", Jne => "JNE", Jle => "JLE", Jmp => "JMP");

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::A(value) => write!(f, "@{value}"),
            Self::C { dest, comp, jump } => {
                if let Some(dest) = dest {
                    write!(f, "{dest}=")?;
                }
                write!(f, "{comp}")?;
                if let Some(jump) = jump {
                    write!(f, ";{jump}")?;
                }
                Ok(())
            }
            Self::Label(symbol) => write!(f, "({symbol})"),
            Self::Comment(comment) => write!(f, "// {comment}"),
        }
    }
}
