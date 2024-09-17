/// Bitwise operations for Hack assembly instructions
/// The following masks are used to extract the relevant bits from the instruction:
/// - ADDRESS_MASK: 0b1000000000000000 (0x8000)
/// - A_BIT: 0b0001000000000000 (0x4000)
/// - COMP_MASK: 0b0001111111000000 (0x3FC0)
/// - DEST_MASK: 0b0000000000111000 (0x0078)
/// - JUMP_MASK: 0b0000000000000111 (0x0007)
const ADDRESS_MASK: u16 = 0b1000000000000000;
const A_BIT: u16 = 0b0001000000000000;
const COMP_MASK: u16 = 0b0000111111000000;
const DEST_MASK: u16 = 0b0000000000111000;
const JUMP_MASK: u16 = 0b0000000000000111;

pub struct Instruction(u16);

impl Instruction {
    pub fn new(instruction: u16) -> Self {
        Self(instruction)
    }

    pub fn a_bit_is_set(&self) -> bool {
        self.0 & A_BIT != 0
    }

    pub fn is_computation(&self) -> bool {
        self.0 & ADDRESS_MASK != 0
    }

    pub fn is_address(&self) -> bool {
        self.0 & ADDRESS_MASK == 0
    }

    pub fn inner(&self) -> u16 {
        self.0
    }

    pub fn comp_bits(&self) -> u16 {
        (self.0 & COMP_MASK) >> 6
    }

    pub fn jump(&self) -> Jump {
        Jump::new(self.0)
    }

    pub fn dest_is_none(&self) -> bool {
        self.0 & DEST_MASK == 0
    }

    pub fn dest_addr(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    pub fn dest_data(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub fn dest_mem(&self) -> bool {
        self.0 & (1 << 3) != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jump {
    Jgt,
    Jeq,
    Jge,
    Jlt,
    Jne,
    Jle,
    Jmp,
    None,
}

impl Jump {
    pub fn new(instruction: u16) -> Self {
        match instruction & JUMP_MASK {
            0b001 => Jump::Jgt,
            0b010 => Jump::Jeq,
            0b011 => Jump::Jge,
            0b100 => Jump::Jlt,
            0b101 => Jump::Jne,
            0b110 => Jump::Jle,
            0b111 => Jump::Jmp,
            0b000 => Jump::None,
            _ => panic!("Invalid instruction: {:016b}", instruction),
        }
    }

    pub fn is_none(&self) -> bool {
        *self == Jump::None
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Jump::Jgt => "JGT",
            Jump::Jeq => "JEQ",
            Jump::Jge => "JGE",
            Jump::Jlt => "JLT",
            Jump::Jne => "JNE",
            Jump::Jle => "JLE",
            Jump::Jmp => "JMP",
            Jump::None => "NONE",
        }
    }

    pub fn cmp(&self, input: i16) -> bool {
        match self {
            Jump::Jgt => input > 0,
            Jump::Jeq => input == 0,
            Jump::Jge => input >= 0,
            Jump::Jlt => input < 0,
            Jump::Jne => input != 0,
            Jump::Jle => input <= 0,
            Jump::Jmp => true,
            Jump::None => false,
        }
    }
}
