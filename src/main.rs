use paste::paste;

macro_rules! register_pair {
    ($high_reg:ident, $low_reg:ident) => {
        paste::item! {
        fn [<get_ $high_reg $low_reg>](&self) -> u16 {
            (self.$high_reg as u16) << 8 | self.$low_reg as u16
        }

        fn [<set_ $high_reg $low_reg>](&mut self, value: u16) {
            self.$high_reg = ((value & 0xFF00) >> 8) as u8;
            self.$low_reg = (value & 0xFF) as u8;
        }
        }
    };
}

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

impl Registers {
    register_pair!(a, f);
    register_pair!(b, c);
    register_pair!(d, e);
    register_pair!(h, l);
}

struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl FlagsRegister {
    const ZERO_FLAG_BYTE_POSITION: u8 = 7;
    const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
    const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
    const CARRY_FLAG_BYTE_POSITION: u8 = 4;
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero { 1 } else { 0 }) << FlagsRegister::ZERO_FLAG_BYTE_POSITION
            | (if flag.subtract { 1 } else { 0 }) << FlagsRegister::SUBTRACT_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << FlagsRegister::HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << FlagsRegister::CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> FlagsRegister::ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> FlagsRegister::SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> FlagsRegister::HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> FlagsRegister::CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registers() {
        let mut registers: Registers = {
            Registers {
                a: 0xFF as u8,
                b: 0x11 as u8,
                c: 0x22 as u8,
                d: 0x33 as u8,
                e: 0x44 as u8,
                f: 0x55 as u8,
                h: 0x66 as u8,
                l: 0x77 as u8,
            }
        };

        assert_eq!(0xFF55, registers.get_af());
        assert_eq!(0x1122, registers.get_bc());
        assert_eq!(0x3344, registers.get_de());
        assert_eq!(0x6677, registers.get_hl());

        registers.set_bc(0xDEAD);
        assert_eq!(0xDEAD, registers.get_bc());
        assert_eq!(0xDE, registers.b);
        assert_eq!(0xAD, registers.c);
    }
}
