#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Illegal = 0,
    Brk,
    Nop,
    Adc,
    Sbc,
    And,
    Eor,
    Ora,
    Asl,
    Lsr,
    Rol,
    Ror,
    Inc,
    Inx,
    Iny,
    Dec,
    Dex,
    Dey,
    Cmp,
    Cpy,
    Cpx,
    Jmp,
    Jsr,
    Rts,
    Rti,
    Bne,
    Bvs,
    Bvc,
    Bmi,
    Beq,
    Bcs,
    Bcc,
    Bpl,
    Bit,
    Lda,
    Ldx,
    Ldy,
    Sta,
    Stx,
    Sty,
    Cld,
    Cli,
    Clv,
    Clc,
    Sec,
    Sei,
    Sed,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
    Pha,
    Pla,
    Php,
    Plp,
}

impl Instruction {
    #[must_use]
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Self::Illegal => "???",
            Self::Brk => "BRK",
            Self::Nop => "NOP",
            Self::Adc => "ADC",
            Self::Sbc => "SBC",
            Self::And => "AND",
            Self::Eor => "EOR",
            Self::Ora => "ORA",
            Self::Asl => "ASL",
            Self::Lsr => "LSR",
            Self::Rol => "ROL",
            Self::Ror => "ROR",
            Self::Inc => "INC",
            Self::Inx => "INX",
            Self::Iny => "INY",
            Self::Dec => "DEC",
            Self::Dex => "DEX",
            Self::Dey => "DEY",
            Self::Cmp => "CMP",
            Self::Cpy => "CPY",
            Self::Cpx => "CPX",
            Self::Jmp => "JMP",
            Self::Jsr => "JSR",
            Self::Rts => "RTS",
            Self::Rti => "RTI",
            Self::Bne => "BNE",
            Self::Bvs => "BVS",
            Self::Bvc => "BVC",
            Self::Bmi => "BMI",
            Self::Beq => "BEQ",
            Self::Bcs => "BCS",
            Self::Bcc => "BCC",
            Self::Bpl => "BPL",
            Self::Bit => "BIT",
            Self::Lda => "LDA",
            Self::Ldx => "LDX",
            Self::Ldy => "LDY",
            Self::Sta => "STA",
            Self::Stx => "STX",
            Self::Sty => "STY",
            Self::Cld => "CLD",
            Self::Cli => "CLI",
            Self::Clv => "CLV",
            Self::Clc => "CLC",
            Self::Sec => "SEC",
            Self::Sei => "SEI",
            Self::Sed => "SED",
            Self::Tax => "TAX",
            Self::Tay => "TAY",
            Self::Tsx => "TSX",
            Self::Txa => "TXA",
            Self::Txs => "TXS",
            Self::Tya => "TYA",
            Self::Pha => "PHA",
            Self::Pla => "PLA",
            Self::Php => "PHP",
            Self::Plp => "PLP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AddressingMode {
    Implied = 0,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub len: u8,
    pub cycles: u8,
}

impl Opcode {
    const ILLEGAL: Self = Self::new(Instruction::Illegal, AddressingMode::Implied, 1, 0);

    const fn new(instruction: Instruction, mode: AddressingMode, len: u8, cycles: u8) -> Self {
        Self {
            instruction,
            mode,
            len,
            cycles,
        }
    }
}

const fn build_table() -> [Opcode; 256] {
    let mut table = [Opcode::ILLEGAL; 256];

    table[0x00] = Opcode::new(Instruction::Brk, AddressingMode::Implied, 1, 7);
    table[0xEA] = Opcode::new(Instruction::Nop, AddressingMode::Implied, 1, 2);

    // ADC
    table[0x69] = Opcode::new(Instruction::Adc, AddressingMode::Immediate, 2, 2);
    table[0x65] = Opcode::new(Instruction::Adc, AddressingMode::ZeroPage, 2, 3);
    table[0x75] = Opcode::new(Instruction::Adc, AddressingMode::ZeroPageX, 2, 4);
    table[0x6D] = Opcode::new(Instruction::Adc, AddressingMode::Absolute, 3, 4);
    table[0x7D] = Opcode::new(Instruction::Adc, AddressingMode::AbsoluteX, 3, 4);
    table[0x79] = Opcode::new(Instruction::Adc, AddressingMode::AbsoluteY, 3, 4);
    table[0x61] = Opcode::new(Instruction::Adc, AddressingMode::IndirectX, 2, 6);
    table[0x71] = Opcode::new(Instruction::Adc, AddressingMode::IndirectY, 2, 5);

    // SBC
    table[0xE9] = Opcode::new(Instruction::Sbc, AddressingMode::Immediate, 2, 2);
    table[0xE5] = Opcode::new(Instruction::Sbc, AddressingMode::ZeroPage, 2, 3);
    table[0xF5] = Opcode::new(Instruction::Sbc, AddressingMode::ZeroPageX, 2, 4);
    table[0xED] = Opcode::new(Instruction::Sbc, AddressingMode::Absolute, 3, 4);
    table[0xFD] = Opcode::new(Instruction::Sbc, AddressingMode::AbsoluteX, 3, 4);
    table[0xF9] = Opcode::new(Instruction::Sbc, AddressingMode::AbsoluteY, 3, 4);
    table[0xE1] = Opcode::new(Instruction::Sbc, AddressingMode::IndirectX, 2, 6);
    table[0xF1] = Opcode::new(Instruction::Sbc, AddressingMode::IndirectY, 2, 5);

    // AND
    table[0x29] = Opcode::new(Instruction::And, AddressingMode::Immediate, 2, 2);
    table[0x25] = Opcode::new(Instruction::And, AddressingMode::ZeroPage, 2, 3);
    table[0x35] = Opcode::new(Instruction::And, AddressingMode::ZeroPageX, 2, 4);
    table[0x2D] = Opcode::new(Instruction::And, AddressingMode::Absolute, 3, 4);
    table[0x3D] = Opcode::new(Instruction::And, AddressingMode::AbsoluteX, 3, 4);
    table[0x39] = Opcode::new(Instruction::And, AddressingMode::AbsoluteY, 3, 4);
    table[0x21] = Opcode::new(Instruction::And, AddressingMode::IndirectX, 2, 6);
    table[0x31] = Opcode::new(Instruction::And, AddressingMode::IndirectY, 2, 5);

    // EOR
    table[0x49] = Opcode::new(Instruction::Eor, AddressingMode::Immediate, 2, 2);
    table[0x45] = Opcode::new(Instruction::Eor, AddressingMode::ZeroPage, 2, 3);
    table[0x55] = Opcode::new(Instruction::Eor, AddressingMode::ZeroPageX, 2, 4);
    table[0x4D] = Opcode::new(Instruction::Eor, AddressingMode::Absolute, 3, 4);
    table[0x5D] = Opcode::new(Instruction::Eor, AddressingMode::AbsoluteX, 3, 4);
    table[0x59] = Opcode::new(Instruction::Eor, AddressingMode::AbsoluteY, 3, 4);
    table[0x41] = Opcode::new(Instruction::Eor, AddressingMode::IndirectX, 2, 6);
    table[0x51] = Opcode::new(Instruction::Eor, AddressingMode::IndirectY, 2, 5);

    // ORA
    table[0x09] = Opcode::new(Instruction::Ora, AddressingMode::Immediate, 2, 2);
    table[0x05] = Opcode::new(Instruction::Ora, AddressingMode::ZeroPage, 2, 3);
    table[0x15] = Opcode::new(Instruction::Ora, AddressingMode::ZeroPageX, 2, 4);
    table[0x0D] = Opcode::new(Instruction::Ora, AddressingMode::Absolute, 3, 4);
    table[0x1D] = Opcode::new(Instruction::Ora, AddressingMode::AbsoluteX, 3, 4);
    table[0x19] = Opcode::new(Instruction::Ora, AddressingMode::AbsoluteY, 3, 4);
    table[0x01] = Opcode::new(Instruction::Ora, AddressingMode::IndirectX, 2, 6);
    table[0x11] = Opcode::new(Instruction::Ora, AddressingMode::IndirectY, 2, 5);

    // Shifts / rotates
    table[0x0A] = Opcode::new(Instruction::Asl, AddressingMode::Implied, 1, 2);
    table[0x06] = Opcode::new(Instruction::Asl, AddressingMode::ZeroPage, 2, 5);
    table[0x16] = Opcode::new(Instruction::Asl, AddressingMode::ZeroPageX, 2, 6);
    table[0x0E] = Opcode::new(Instruction::Asl, AddressingMode::Absolute, 3, 6);
    table[0x1E] = Opcode::new(Instruction::Asl, AddressingMode::AbsoluteX, 3, 7);

    table[0x4A] = Opcode::new(Instruction::Lsr, AddressingMode::Implied, 1, 2);
    table[0x46] = Opcode::new(Instruction::Lsr, AddressingMode::ZeroPage, 2, 5);
    table[0x56] = Opcode::new(Instruction::Lsr, AddressingMode::ZeroPageX, 2, 6);
    table[0x4E] = Opcode::new(Instruction::Lsr, AddressingMode::Absolute, 3, 6);
    table[0x5E] = Opcode::new(Instruction::Lsr, AddressingMode::AbsoluteX, 3, 7);

    table[0x2A] = Opcode::new(Instruction::Rol, AddressingMode::Implied, 1, 2);
    table[0x26] = Opcode::new(Instruction::Rol, AddressingMode::ZeroPage, 2, 5);
    table[0x36] = Opcode::new(Instruction::Rol, AddressingMode::ZeroPageX, 2, 6);
    table[0x2E] = Opcode::new(Instruction::Rol, AddressingMode::Absolute, 3, 6);
    table[0x3E] = Opcode::new(Instruction::Rol, AddressingMode::AbsoluteX, 3, 7);

    table[0x6A] = Opcode::new(Instruction::Ror, AddressingMode::Implied, 1, 2);
    table[0x66] = Opcode::new(Instruction::Ror, AddressingMode::ZeroPage, 2, 5);
    table[0x76] = Opcode::new(Instruction::Ror, AddressingMode::ZeroPageX, 2, 6);
    table[0x6E] = Opcode::new(Instruction::Ror, AddressingMode::Absolute, 3, 6);
    table[0x7E] = Opcode::new(Instruction::Ror, AddressingMode::AbsoluteX, 3, 7);

    // Increment / decrement
    table[0xE6] = Opcode::new(Instruction::Inc, AddressingMode::ZeroPage, 2, 5);
    table[0xF6] = Opcode::new(Instruction::Inc, AddressingMode::ZeroPageX, 2, 6);
    table[0xEE] = Opcode::new(Instruction::Inc, AddressingMode::Absolute, 3, 6);
    table[0xFE] = Opcode::new(Instruction::Inc, AddressingMode::AbsoluteX, 3, 7);
    table[0xE8] = Opcode::new(Instruction::Inx, AddressingMode::Implied, 1, 2);
    table[0xC8] = Opcode::new(Instruction::Iny, AddressingMode::Implied, 1, 2);

    table[0xC6] = Opcode::new(Instruction::Dec, AddressingMode::ZeroPage, 2, 5);
    table[0xD6] = Opcode::new(Instruction::Dec, AddressingMode::ZeroPageX, 2, 6);
    table[0xCE] = Opcode::new(Instruction::Dec, AddressingMode::Absolute, 3, 6);
    table[0xDE] = Opcode::new(Instruction::Dec, AddressingMode::AbsoluteX, 3, 7);
    table[0xCA] = Opcode::new(Instruction::Dex, AddressingMode::Implied, 1, 2);
    table[0x88] = Opcode::new(Instruction::Dey, AddressingMode::Implied, 1, 2);

    // Compare
    table[0xC9] = Opcode::new(Instruction::Cmp, AddressingMode::Immediate, 2, 2);
    table[0xC5] = Opcode::new(Instruction::Cmp, AddressingMode::ZeroPage, 2, 3);
    table[0xD5] = Opcode::new(Instruction::Cmp, AddressingMode::ZeroPageX, 2, 4);
    table[0xCD] = Opcode::new(Instruction::Cmp, AddressingMode::Absolute, 3, 4);
    table[0xDD] = Opcode::new(Instruction::Cmp, AddressingMode::AbsoluteX, 3, 4);
    table[0xD9] = Opcode::new(Instruction::Cmp, AddressingMode::AbsoluteY, 3, 4);
    table[0xC1] = Opcode::new(Instruction::Cmp, AddressingMode::IndirectX, 2, 6);
    table[0xD1] = Opcode::new(Instruction::Cmp, AddressingMode::IndirectY, 2, 5);

    table[0xC0] = Opcode::new(Instruction::Cpy, AddressingMode::Immediate, 2, 2);
    table[0xC4] = Opcode::new(Instruction::Cpy, AddressingMode::ZeroPage, 2, 3);
    table[0xCC] = Opcode::new(Instruction::Cpy, AddressingMode::Absolute, 3, 4);

    table[0xE0] = Opcode::new(Instruction::Cpx, AddressingMode::Immediate, 2, 2);
    table[0xE4] = Opcode::new(Instruction::Cpx, AddressingMode::ZeroPage, 2, 3);
    table[0xEC] = Opcode::new(Instruction::Cpx, AddressingMode::Absolute, 3, 4);

    // Flow control
    table[0x4C] = Opcode::new(Instruction::Jmp, AddressingMode::Absolute, 3, 3);
    table[0x6C] = Opcode::new(Instruction::Jmp, AddressingMode::Implied, 3, 5);
    table[0x20] = Opcode::new(Instruction::Jsr, AddressingMode::Absolute, 3, 6);
    table[0x60] = Opcode::new(Instruction::Rts, AddressingMode::Implied, 1, 6);
    table[0x40] = Opcode::new(Instruction::Rti, AddressingMode::Implied, 1, 6);

    table[0xD0] = Opcode::new(Instruction::Bne, AddressingMode::Implied, 2, 2);
    table[0x70] = Opcode::new(Instruction::Bvs, AddressingMode::Implied, 2, 2);
    table[0x50] = Opcode::new(Instruction::Bvc, AddressingMode::Implied, 2, 2);
    table[0x30] = Opcode::new(Instruction::Bmi, AddressingMode::Implied, 2, 2);
    table[0xF0] = Opcode::new(Instruction::Beq, AddressingMode::Implied, 2, 2);
    table[0xB0] = Opcode::new(Instruction::Bcs, AddressingMode::Implied, 2, 2);
    table[0x90] = Opcode::new(Instruction::Bcc, AddressingMode::Implied, 2, 2);
    table[0x10] = Opcode::new(Instruction::Bpl, AddressingMode::Implied, 2, 2);

    table[0x24] = Opcode::new(Instruction::Bit, AddressingMode::ZeroPage, 2, 3);
    table[0x2C] = Opcode::new(Instruction::Bit, AddressingMode::Absolute, 3, 4);

    // Loads
    table[0xA9] = Opcode::new(Instruction::Lda, AddressingMode::Immediate, 2, 2);
    table[0xA5] = Opcode::new(Instruction::Lda, AddressingMode::ZeroPage, 2, 3);
    table[0xB5] = Opcode::new(Instruction::Lda, AddressingMode::ZeroPageX, 2, 4);
    table[0xAD] = Opcode::new(Instruction::Lda, AddressingMode::Absolute, 3, 4);
    table[0xBD] = Opcode::new(Instruction::Lda, AddressingMode::AbsoluteX, 3, 4);
    table[0xB9] = Opcode::new(Instruction::Lda, AddressingMode::AbsoluteY, 3, 4);
    table[0xA1] = Opcode::new(Instruction::Lda, AddressingMode::IndirectX, 2, 6);
    table[0xB1] = Opcode::new(Instruction::Lda, AddressingMode::IndirectY, 2, 5);

    table[0xA2] = Opcode::new(Instruction::Ldx, AddressingMode::Immediate, 2, 2);
    table[0xA6] = Opcode::new(Instruction::Ldx, AddressingMode::ZeroPage, 2, 3);
    table[0xB6] = Opcode::new(Instruction::Ldx, AddressingMode::ZeroPageY, 2, 4);
    table[0xAE] = Opcode::new(Instruction::Ldx, AddressingMode::Absolute, 3, 4);
    table[0xBE] = Opcode::new(Instruction::Ldx, AddressingMode::AbsoluteY, 3, 4);

    table[0xA0] = Opcode::new(Instruction::Ldy, AddressingMode::Immediate, 2, 2);
    table[0xA4] = Opcode::new(Instruction::Ldy, AddressingMode::ZeroPage, 2, 3);
    table[0xB4] = Opcode::new(Instruction::Ldy, AddressingMode::ZeroPageX, 2, 4);
    table[0xAC] = Opcode::new(Instruction::Ldy, AddressingMode::Absolute, 3, 4);
    table[0xBC] = Opcode::new(Instruction::Ldy, AddressingMode::AbsoluteX, 3, 4);

    // Stores
    table[0x85] = Opcode::new(Instruction::Sta, AddressingMode::ZeroPage, 2, 3);
    table[0x95] = Opcode::new(Instruction::Sta, AddressingMode::ZeroPageX, 2, 4);
    table[0x8D] = Opcode::new(Instruction::Sta, AddressingMode::Absolute, 3, 4);
    table[0x9D] = Opcode::new(Instruction::Sta, AddressingMode::AbsoluteX, 3, 5);
    table[0x99] = Opcode::new(Instruction::Sta, AddressingMode::AbsoluteY, 3, 5);
    table[0x81] = Opcode::new(Instruction::Sta, AddressingMode::IndirectX, 2, 6);
    table[0x91] = Opcode::new(Instruction::Sta, AddressingMode::IndirectY, 2, 6);

    table[0x86] = Opcode::new(Instruction::Stx, AddressingMode::ZeroPage, 2, 3);
    table[0x96] = Opcode::new(Instruction::Stx, AddressingMode::ZeroPageY, 2, 4);
    table[0x8E] = Opcode::new(Instruction::Stx, AddressingMode::Absolute, 3, 4);

    table[0x84] = Opcode::new(Instruction::Sty, AddressingMode::ZeroPage, 2, 3);
    table[0x94] = Opcode::new(Instruction::Sty, AddressingMode::ZeroPageX, 2, 4);
    table[0x8C] = Opcode::new(Instruction::Sty, AddressingMode::Absolute, 3, 4);

    // Flags
    table[0xD8] = Opcode::new(Instruction::Cld, AddressingMode::Implied, 1, 2);
    table[0x58] = Opcode::new(Instruction::Cli, AddressingMode::Implied, 1, 2);
    table[0xB8] = Opcode::new(Instruction::Clv, AddressingMode::Implied, 1, 2);
    table[0x18] = Opcode::new(Instruction::Clc, AddressingMode::Implied, 1, 2);
    table[0x38] = Opcode::new(Instruction::Sec, AddressingMode::Implied, 1, 2);
    table[0x78] = Opcode::new(Instruction::Sei, AddressingMode::Implied, 1, 2);
    table[0xF8] = Opcode::new(Instruction::Sed, AddressingMode::Implied, 1, 2);

    // Transfers
    table[0xAA] = Opcode::new(Instruction::Tax, AddressingMode::Implied, 1, 2);
    table[0xA8] = Opcode::new(Instruction::Tay, AddressingMode::Implied, 1, 2);
    table[0xBA] = Opcode::new(Instruction::Tsx, AddressingMode::Implied, 1, 2);
    table[0x8A] = Opcode::new(Instruction::Txa, AddressingMode::Implied, 1, 2);
    table[0x9A] = Opcode::new(Instruction::Txs, AddressingMode::Implied, 1, 2);
    table[0x98] = Opcode::new(Instruction::Tya, AddressingMode::Implied, 1, 2);

    // Stack
    table[0x48] = Opcode::new(Instruction::Pha, AddressingMode::Implied, 1, 3);
    table[0x68] = Opcode::new(Instruction::Pla, AddressingMode::Implied, 1, 4);
    table[0x08] = Opcode::new(Instruction::Php, AddressingMode::Implied, 1, 3);
    table[0x28] = Opcode::new(Instruction::Plp, AddressingMode::Implied, 1, 4);

    table
}

pub static OPCODES: [Opcode; 256] = build_table();

#[inline]
#[must_use]
pub fn decode(code: u8) -> Opcode {
    OPCODES[usize::from(code)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_all_official_opcodes() {
        let count = OPCODES
            .iter()
            .filter(|opcode| opcode.instruction != Instruction::Illegal)
            .count();

        assert_eq!(count, 151);
    }
}
