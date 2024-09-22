#[derive(Debug, PartialEq)]
pub struct Alu {
    zx: bool,
    nx: bool,
    zy: bool,
    ny: bool,
    f: bool,
    no: bool,
}

impl Alu {
    pub fn new(zx: bool, nx: bool, zy: bool, ny: bool, f: bool, no: bool) -> Self {
        Self {
            zx,
            nx,
            zy,
            ny,
            f,
            no,
        }
    }

    pub fn load_bits(&mut self, bits: u16) -> &mut Self {
        self.zx = bits & 0b100000 != 0;
        self.nx = bits & 0b010000 != 0;
        self.zy = bits & 0b001000 != 0;
        self.ny = bits & 0b000100 != 0;
        self.f = bits & 0b000010 != 0;
        self.no = bits & 0b000001 != 0;
        self
    }

    pub fn execute(&self, x: i16, y: i16) -> i16 {
        // Apply zx and nx to x input
        let x = if self.zx { 0 } else { x };
        let x = if self.nx { !x } else { x };

        // Apply zy and ny to y input
        let y = if self.zy { 0 } else { y };
        let y = if self.ny { !y } else { y };

        // Compute either addition or bitwise AND based on f
        let result = if self.f {
            x.wrapping_add(y) // Perform addition
        } else {
            x & y // Perform bitwise AND
        };

        // Apply no to negate the output if needed
        if self.no {
            !result
        } else {
            result
        }
    }
}

impl Default for Alu {
    fn default() -> Self {
        Self::new(false, false, false, false, false, false)
    }
}

#[cfg(test)]
pub mod unit {
    use super::*;

    #[test]
    fn test_alu_new() {
        let alu = Alu::new(true, false, true, false, true, false);
        assert_eq!(alu.zx, true);
        assert_eq!(alu.nx, false);
        assert_eq!(alu.zy, true);
        assert_eq!(alu.ny, false);
        assert_eq!(alu.f, true);
        assert_eq!(alu.no, false);
    }

    #[test]
    fn test_alu_from_bits() {
        let mut alu = Alu::new(false, false, false, false, false, false);
        alu.load_bits(0b101010);
        assert_eq!(alu.zx, true);
        assert_eq!(alu.nx, false);
        assert_eq!(alu.zy, true);
        assert_eq!(alu.ny, false);
        assert_eq!(alu.f, true);
        assert_eq!(alu.no, false);
    }

    #[test]
    fn test_alu_execute_zero() {
        let alu = Alu::new(true, false, true, false, false, false);
        assert_eq!(alu.execute(100, 100), 0)
    }

    #[test]
    fn test_alu_execute_add() {
        let alu = Alu::new(false, false, false, false, true, false);
        assert_eq!(alu.execute(100, 100), 200)
    }

    #[test]
    fn test_alu_execute_and() {
        let alu = Alu::new(false, false, false, false, false, false);
        assert_eq!(alu.execute(0b1010, 0b1100), 0b1000)
    }

    #[test]
    fn test_alu_execute_not() {
        let alu = Alu::new(false, false, false, false, false, true);
        assert_eq!(alu.execute(0b1010, 0b1100), !0b1000)
    }

    #[test]
    fn test_alu_execute_negate() {
        let alu = Alu::new(false, true, false, true, false, false);
        assert_eq!(alu.execute(100, 100), (!100) & (!100))
    }

    #[test]
    fn test_alu_execute_all() {
        let alu = Alu::new(true, true, true, true, true, true);
        assert_eq!(alu.execute(100, 100), 1)
    }

    #[test]
    fn test_alu_execute_overflow() {
        let alu = Alu::new(false, false, false, false, true, false);
        assert_eq!(alu.execute(i16::MAX, 1), i16::MIN)
    }
}
