#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Illegal = 0,
    Brk,
    Lda,
    Sta,
    Tax,
    Inx,
}

impl Instruction {
    #[must_use]
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Self::Illegal => "???",
            Self::Brk => "BRK",
            Self::Lda => "LDA",
            Self::Sta => "STA",
            Self::Tax => "TAX",
            Self::Inx => "INX",
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
#[repr(C)]
pub struct Opcode {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub len: u8,
    pub cycles: u8,
    flags: u8,
}

const PAGE_CROSS_PENALTY: u8 = 1 << 0;

impl Opcode {
    const ILLEGAL: Self = Self::new(Instruction::Illegal, AddressingMode::Implied, 1, 0, false);

    const fn new(
        instruction: Instruction,
        mode: AddressingMode,
        len: u8,
        cycles: u8,
        page_cross_penalty: bool,
    ) -> Self {
        Self {
            instruction,
            mode,
            len,
            cycles,
            flags: if page_cross_penalty {
                PAGE_CROSS_PENALTY
            } else {
                0
            },
        }
    }

    #[must_use]
    pub const fn page_cross_penalty(self) -> bool {
        self.flags & PAGE_CROSS_PENALTY != 0
    }
}

const fn build_table() -> [Opcode; 256] {
    let mut table = [Opcode::ILLEGAL; 256];

    // BRK (tutorial stop sentinel)
    table[0x00] = Opcode::new(Instruction::Brk, AddressingMode::Implied, 1, 7, false);

    // Register instructions
    table[0xAA] = Opcode::new(Instruction::Tax, AddressingMode::Implied, 1, 2, false);
    table[0xE8] = Opcode::new(Instruction::Inx, AddressingMode::Implied, 1, 2, false);

    // LDA
    table[0xA9] = Opcode::new(Instruction::Lda, AddressingMode::Immediate, 2, 2, false);
    table[0xA5] = Opcode::new(Instruction::Lda, AddressingMode::ZeroPage, 2, 3, false);
    table[0xB5] = Opcode::new(Instruction::Lda, AddressingMode::ZeroPageX, 2, 4, false);
    table[0xAD] = Opcode::new(Instruction::Lda, AddressingMode::Absolute, 3, 4, false);
    table[0xBD] = Opcode::new(Instruction::Lda, AddressingMode::AbsoluteX, 3, 4, true);
    table[0xB9] = Opcode::new(Instruction::Lda, AddressingMode::AbsoluteY, 3, 4, true);
    table[0xA1] = Opcode::new(Instruction::Lda, AddressingMode::IndirectX, 2, 6, false);
    table[0xB1] = Opcode::new(Instruction::Lda, AddressingMode::IndirectY, 2, 5, true);

    // STA
    table[0x85] = Opcode::new(Instruction::Sta, AddressingMode::ZeroPage, 2, 3, false);
    table[0x95] = Opcode::new(Instruction::Sta, AddressingMode::ZeroPageX, 2, 4, false);
    table[0x8D] = Opcode::new(Instruction::Sta, AddressingMode::Absolute, 3, 4, false);
    table[0x9D] = Opcode::new(Instruction::Sta, AddressingMode::AbsoluteX, 3, 5, false);
    table[0x99] = Opcode::new(Instruction::Sta, AddressingMode::AbsoluteY, 3, 5, false);
    table[0x81] = Opcode::new(Instruction::Sta, AddressingMode::IndirectX, 2, 6, false);
    table[0x91] = Opcode::new(Instruction::Sta, AddressingMode::IndirectY, 2, 6, false);

    table
}

pub static OPCODES: [Opcode; 256] = build_table();

#[inline]
#[must_use]
pub fn decode(code: u8) -> Opcode {
    OPCODES[usize::from(code)]
}
