use paste::paste;

trait FlagsRegisterPair {
    fn set_from_u16(&mut self, value: u16);
    fn get_u16(&self) -> u16;
}

impl FlagsRegisterPair for u8 {
    fn set_from_u16(&mut self, value: u16) {
        *self = value as u8;
    }

    fn get_u16(&self) -> u16 {
        *self as u16
    }
}

impl FlagsRegisterPair for FlagsRegister {
    fn set_from_u16(&mut self, value: u16) {
        *self = FlagsRegister::from(value as u8);
    }

    fn get_u16(&self) -> u16 {
        u8::from(*self) as u16
    }
}

macro_rules! register_pair {
    ($high_reg:ident, $low_reg:ident) => {
        paste::item! {
        fn [<get_ $high_reg $low_reg>](&self) -> u16 {
            (self.$high_reg as u16) << 8 | self.$low_reg.get_u16()
        }

        fn [<set_ $high_reg $low_reg>](&mut self, value: u16) {
            self.$high_reg = ((value & 0xFF00) >> 8) as u8;
            self.$low_reg.set_from_u16(value & 0xFF);
        }
        }
    };
}

#[derive(Debug, Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8,
}

impl Registers {
    register_pair!(a, f);
    register_pair!(b, c);
    register_pair!(d, e);
    register_pair!(h, l);
}

#[derive(Debug, Default, Copy, Clone)]
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool, // https://gist.github.com/meganesu/9e228b6b587decc783aa9be34ae27841
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

enum Instruction {
    ADD(ArithmeticTarget),
    ADDHL(ADDHLTarget),
    ADC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
    AND(ArithmeticTarget),
    OR(ArithmeticTarget),
    XOR(ArithmeticTarget),
    CP(ArithmeticTarget), // same as SUB but without storing the result
    INC(ArithmeticTarget),
    DEC(ArithmeticTarget),
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum ADDHLTarget {
    BC,
    DE,
    HL,
}

#[derive(Debug, Default)]
struct CPU {
    registers: Registers,
}

impl CPU {
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a = self.add(value);
            }
            Instruction::ADDHL(target) => {
                let value = match target {
                    ADDHLTarget::BC => self.registers.get_bc(),
                    ADDHLTarget::DE => self.registers.get_de(),
                    ADDHLTarget::HL => self.registers.get_hl(),
                };
                let sum = self.addhl(value);
                self.registers.set_hl(sum);
            }
            Instruction::ADC(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a = self.adc(value);
            }
            Instruction::SUB(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a = self.sub(value);
            }
            Instruction::SBC(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a = self.sbc(value);
            }
            Instruction::AND(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a &= value;
            }
            Instruction::OR(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a |= value;
            }
            Instruction::XOR(target) => {
                let value = self.get_value_from_target(target);
                self.registers.a ^= value;
            }
            Instruction::CP(target) => {
                let value = self.get_value_from_target(target);
                self.sub(value);
            }
            Instruction::INC(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a = self.inc(self.registers.a),
                    ArithmeticTarget::B => self.registers.b = self.inc(self.registers.b),
                    ArithmeticTarget::C => self.registers.c = self.inc(self.registers.c),
                    ArithmeticTarget::D => self.registers.d = self.inc(self.registers.d),
                    ArithmeticTarget::E => self.registers.e = self.inc(self.registers.e),
                    ArithmeticTarget::H => self.registers.h = self.inc(self.registers.h),
                    ArithmeticTarget::L => self.registers.l = self.inc(self.registers.l),
                };
            }
            Instruction::DEC(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a = self.dec(self.registers.a),
                    ArithmeticTarget::B => self.registers.b = self.dec(self.registers.b),
                    ArithmeticTarget::C => self.registers.c = self.dec(self.registers.c),
                    ArithmeticTarget::D => self.registers.d = self.dec(self.registers.d),
                    ArithmeticTarget::E => self.registers.e = self.dec(self.registers.e),
                    ArithmeticTarget::H => self.registers.h = self.dec(self.registers.h),
                    ArithmeticTarget::L => self.registers.l = self.dec(self.registers.l),
                };
            }
        }
    }

    fn get_value_from_target(&self, target: ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::A => self.registers.a,
            ArithmeticTarget::B => self.registers.b,
            ArithmeticTarget::C => self.registers.c,
            ArithmeticTarget::D => self.registers.d,
            ArithmeticTarget::E => self.registers.e,
            ArithmeticTarget::H => self.registers.h,
            ArithmeticTarget::L => self.registers.l,
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (result, carry) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = ((self.registers.a & 0xF) + (result & 0xF) & 0x10) == 0x10;
        self.registers.f.carry = carry;

        result
    }

    fn addhl(&mut self, value: u16) -> u16 {
        let (result, carry) = self.registers.get_hl().overflowing_add(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry =
            ((self.registers.get_hl() & 0xFFF) + (result & 0xFFF)) > 0xFFF;
        self.registers.f.carry = carry;

        result
    }

    fn adc(&mut self, value: u8) -> u8 {
        let previous_carry = self.registers.f.carry as u8;
        let (mut result, carry) = self.registers.a.overflowing_add(value);
        result += carry as u8 + previous_carry;
        self.registers.f.carry = carry;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = ((self.registers.a & 0xF) + (result & 0xF) & 0x10) == 0x10;

        result
    }

    fn sub(&mut self, value: u8) -> u8 {
        let (result, borrow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (result & 0xF);
        self.registers.f.carry = borrow;

        result
    }

    fn sbc(&mut self, value: u8) -> u8 {
        let (mut result, borrow) = self.registers.a.overflowing_sub(value);
        result -= borrow as u8;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (result & 0xF);
        self.registers.f.carry = borrow;

        result
    }

    fn inc(&mut self, target: u8) -> u8 {
        let (result, carry) = target.overflowing_add(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (target & 0xF) == 0xF; // if the lower nibble is 0xF, then adding 1
                                                             // will carry to the higher nibble
        self.registers.f.carry = carry;

        result
    }

    fn dec(&mut self, target: u8) -> u8 {
        let (result, carry) = target.overflowing_sub(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (target & 0xF) == 0x0; // if the lower nibble is 0x0, then subtracting 1
                                                             // will borrow from the higher nibble
        self.registers.f.carry = carry;

        result
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    macro_rules! test_flags {
        ($cpu:expr, $zero:expr, $subtract:expr, $half_carry:expr, $carry:expr) => {{
            assert_eq!($cpu.registers.f.zero, $zero);
            assert_eq!($cpu.registers.f.subtract, $subtract);
            assert_eq!($cpu.registers.f.half_carry, $half_carry);
            assert_eq!($cpu.registers.f.carry, $carry);
        }};
    }

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
                f: FlagsRegister::from(0x55 as u8),
                h: 0x66 as u8,
                l: 0x77 as u8,
            }
        };

        assert_eq!(0x50, u8::from(registers.f)); // from FlagsRegister to u8 zeros out the lower bits

        assert_eq!(0xFF50, registers.get_af());
        assert_eq!(0x1122, registers.get_bc());
        assert_eq!(0x3344, registers.get_de());
        assert_eq!(0x6677, registers.get_hl());

        registers.set_bc(0xDEAD);
        assert_eq!(0xDEAD, registers.get_bc());
        assert_eq!(0xDE, registers.b);
        assert_eq!(0xAD, registers.c);
    }

    #[test]
    fn test_add() {
        let mut cpu = CPU::default();

        cpu.registers.a = 0x00;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::ADD(ArithmeticTarget::C));
        assert_eq!(0x01, cpu.registers.a);
        assert_eq!(0, u8::from(cpu.registers.f));

        cpu.registers.a = 0xFF;
        cpu.registers.d = 0x01;
        cpu.execute(Instruction::ADD(ArithmeticTarget::D));
        assert_eq!(0x00, cpu.registers.a);
        assert!(cpu.registers.f.carry);
    }

    #[test]
    fn test_addhl() {
        let mut cpu = CPU::default();
        cpu.registers.set_hl(0x0000);
        cpu.execute(Instruction::ADDHL(ADDHLTarget::HL));
        assert_eq!(0x0000, cpu.registers.get_hl());
        test_flags!(cpu, true, false, false, false);

        cpu.registers.set_hl(0x0001);
        cpu.execute(Instruction::ADDHL(ADDHLTarget::HL));
        assert_eq!(0x0002, cpu.registers.get_hl());
        test_flags!(cpu, false, false, false, false);

        cpu.registers.set_hl(0xFFFF);
        cpu.execute(Instruction::ADDHL(ADDHLTarget::HL));
        assert_eq!(0xFFFE, cpu.registers.get_hl());
        test_flags!(cpu, false, false, true, true);

        cpu.registers.set_hl(0x00FF);
        cpu.execute(Instruction::ADDHL(ADDHLTarget::HL));
        assert_eq!(0x01FE, cpu.registers.get_hl());
        test_flags!(cpu, false, false, false, false);

        cpu.registers.set_hl(0xFFF);
        cpu.execute(Instruction::ADDHL(ADDHLTarget::HL));
        assert_eq!(0x1FFE, cpu.registers.get_hl());
        test_flags!(cpu, false, false, true, false);
    }

    #[test]
    fn test_adc() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x00;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::ADC(ArithmeticTarget::C));
        assert_eq!(0x01, cpu.registers.a);
        test_flags!(cpu, false, false, false, false);

        cpu.registers.a = 0xFF;
        cpu.registers.d = 0x01;
        cpu.execute(Instruction::ADC(ArithmeticTarget::D));
        assert_eq!(0x01, cpu.registers.a);
        test_flags!(cpu, false, false, true, true);

        cpu.registers.a = 0xFF;
        cpu.registers.d = 0x01;
        cpu.registers.f.carry = true;
        cpu.execute(Instruction::ADC(ArithmeticTarget::D));
        assert_eq!(0x02, cpu.registers.a);
        test_flags!(cpu, false, false, true, true);

        cpu.registers.a = 0x8F;
        cpu.registers.b = 0x01;
        cpu.registers.f.carry = true;
        cpu.execute(Instruction::ADC(ArithmeticTarget::B));
        assert_eq!(0x91, cpu.registers.a);
        test_flags!(cpu, false, false, true, false);
    }

    #[test]
    fn test_sub() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x01;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::SUB(ArithmeticTarget::C));
        assert_eq!(0x00, cpu.registers.a);
        test_flags!(cpu, true, true, false, false);

        cpu.registers.a = 0x00;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::SUB(ArithmeticTarget::C));
        assert_eq!(0xFF, cpu.registers.a);
        test_flags!(cpu, false, true, true, true);

        cpu.registers.a = 0x20;
        cpu.registers.c = 0x11;
        cpu.execute(Instruction::SUB(ArithmeticTarget::C));
        assert_eq!(0x0F, cpu.registers.a);
        test_flags!(cpu, false, true, true, false);

        cpu.registers.a = 0x20;
        cpu.registers.c = 0x31;
        cpu.execute(Instruction::SUB(ArithmeticTarget::C));
        assert_eq!(0xEF, cpu.registers.a);
        test_flags!(cpu, false, true, true, true);
    }

    #[test]
    fn test_sbc() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x01;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::SBC(ArithmeticTarget::C));
        assert_eq!(0x00, cpu.registers.a);
        test_flags!(cpu, true, true, false, false);

        cpu.registers.a = 0x20;
        cpu.registers.c = 0x31;
        cpu.execute(Instruction::SBC(ArithmeticTarget::C));
        assert_eq!(0xEE, cpu.registers.a);
        test_flags!(cpu, false, true, true, true);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0b1010;
        cpu.registers.c = 0b1100;
        cpu.execute(Instruction::AND(ArithmeticTarget::C));
        assert_eq!(0b1000, cpu.registers.a);
        test_flags!(cpu, false, false, false, false);
    }

    #[test]
    fn test_or() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0b1010;
        cpu.registers.c = 0b1100;
        cpu.execute(Instruction::OR(ArithmeticTarget::C));
        assert_eq!(0b1110, cpu.registers.a);
        test_flags!(cpu, false, false, false, false);
    }

    #[test]
    fn test_xor() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0b1010;
        cpu.registers.c = 0b1100;
        cpu.execute(Instruction::XOR(ArithmeticTarget::C));
        assert_eq!(0b0110, cpu.registers.a);
        test_flags!(cpu, false, false, false, false);
    }

    #[test]
    fn test_cp() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x01;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::CP(ArithmeticTarget::C));
        assert_eq!(0x01, cpu.registers.a);
        test_flags!(cpu, true, true, false, false);

        cpu.registers.a = 0x00;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::CP(ArithmeticTarget::C));
        assert_eq!(0x00, cpu.registers.a);
        test_flags!(cpu, false, true, true, true);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x01;
        cpu.execute(Instruction::INC(ArithmeticTarget::A));
        assert_eq!(0x02, cpu.registers.a);
        test_flags!(cpu, false, false, false, false);

        cpu.registers.a = 0xFF;
        cpu.execute(Instruction::INC(ArithmeticTarget::A));
        assert_eq!(0x00, cpu.registers.a);
        test_flags!(cpu, true, false, true, true);

        cpu.registers.c = 0xFF;
        cpu.execute(Instruction::INC(ArithmeticTarget::C));
        assert_eq!(0x00, cpu.registers.c);
        test_flags!(cpu, true, false, true, true);

        cpu.registers.c = 0x0F;
        cpu.execute(Instruction::INC(ArithmeticTarget::C));
        assert_eq!(0x10, cpu.registers.c);
        test_flags!(cpu, false, false, true, false);
    }

    #[test]
    fn test_dec() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x01;
        cpu.execute(Instruction::DEC(ArithmeticTarget::A));
        assert_eq!(0x00, cpu.registers.a);
        test_flags!(cpu, true, true, false, false);

        cpu.registers.a = 0x00;
        cpu.execute(Instruction::DEC(ArithmeticTarget::A));
        assert_eq!(0xFF, cpu.registers.a);
        test_flags!(cpu, false, true, true, true);

        cpu.registers.c = 0x10;
        cpu.execute(Instruction::DEC(ArithmeticTarget::C));
        assert_eq!(0x0F, cpu.registers.c);
        test_flags!(cpu, false, true, true, false);
    }
}
