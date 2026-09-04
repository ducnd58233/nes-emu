use crate::cpu::{
    bus::{Bus, BusError, FlatMemory},
    opcode::{AddressingMode, Instruction, decode},
    status::Status,
};

mod addressing;
pub mod bus;
pub mod opcode;
pub mod status;

const PROGRAM_START: u16 = 0x8000;
const RESET_VECTOR: u16 = 0xFFFC;
const MAX_PROGRAM_SIZE: usize = (RESET_VECTOR - PROGRAM_START) as usize;
const STACK_BASE: u16 = 0x0100;
const STACK_RESET: u8 = 0xFD;

#[derive(Debug, Clone)]
pub struct Cpu<B> {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: Status,
    program_counter: u16,
    stack_pointer: u8,
    bus: B,
}

impl<B: Bus> Cpu<B> {
    pub fn new(bus: B) -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: Status::default(),
            program_counter: 0,
            stack_pointer: STACK_RESET,
            bus,
        }
    }

    #[must_use]
    pub fn a(&self) -> u8 {
        self.register_a
    }

    #[must_use]
    pub fn x(&self) -> u8 {
        self.register_x
    }

    #[must_use]
    pub fn y(&self) -> u8 {
        self.register_y
    }

    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn bus_mut(&mut self) -> &mut B {
        &mut self.bus
    }

    #[must_use]
    pub fn into_bus(self) -> B {
        self.bus
    }

    pub fn load_and_run(&mut self, program: &[u8]) -> Result<(), CpuError> {
        self.load(program)?;
        self.reset();
        self.run()
    }

    pub fn load(&mut self, program: &[u8]) -> Result<(), CpuError> {
        if program.len() > MAX_PROGRAM_SIZE {
            return Err(CpuError::ProgramTooLarge {
                len: program.len(),
                max: MAX_PROGRAM_SIZE,
            });
        }

        self.bus.load(PROGRAM_START, program)?;
        self.bus.write_u16(RESET_VECTOR, PROGRAM_START);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = Status::default();
        self.stack_pointer = STACK_RESET;
        self.program_counter = self.bus.read_u16(RESET_VECTOR);
    }

    pub fn run(&mut self) -> Result<(), CpuError> {
        while self.step()? {}
        Ok(())
    }

    /// Fetch, decode and execute one instruction.
    pub fn step(&mut self) -> Result<bool, CpuError> {
        let opcode_address = self.program_counter;
        let code = self.bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);

        let opcode = decode(code);
        if opcode.instruction == Instruction::Illegal {
            return Err(CpuError::UnknownOpcode {
                opcode: code,
                address: opcode_address,
            });
        }

        let pc_handled = match opcode.instruction {
            Instruction::Brk => return Ok(false),
            Instruction::Nop => false,

            Instruction::Lda => {
                let value = self.read_operand(opcode.mode)?;
                self.set_a(value);
                false
            }
            Instruction::Ldx => {
                self.register_x = self.read_operand(opcode.mode)?;
                self.status.update_zero_and_negative(self.register_x);
                false
            }
            Instruction::Ldy => {
                self.register_y = self.read_operand(opcode.mode)?;
                self.status.update_zero_and_negative(self.register_y);
                false
            }
            Instruction::Sta => {
                let address = self.operand_address(opcode.mode)?;
                self.bus.write(address, self.register_a);
                false
            }
            Instruction::Stx => {
                let address = self.operand_address(opcode.mode)?;
                self.bus.write(address, self.register_x);
                false
            }
            Instruction::Sty => {
                let address = self.operand_address(opcode.mode)?;
                self.bus.write(address, self.register_y);
                false
            }

            Instruction::Adc => {
                let value = self.read_operand(opcode.mode)?;
                self.add_to_a(value);
                false
            }
            Instruction::Sbc => {
                let value = self.read_operand(opcode.mode)?;
                self.add_to_a(!value);
                false
            }
            Instruction::And => {
                let value = self.read_operand(opcode.mode)?;
                self.set_a(self.register_a & value);
                false
            }
            Instruction::Eor => {
                let value = self.read_operand(opcode.mode)?;
                self.set_a(self.register_a ^ value);
                false
            }
            Instruction::Ora => {
                let value = self.read_operand(opcode.mode)?;
                self.set_a(self.register_a | value);
                false
            }

            Instruction::Asl => {
                self.asl(opcode.mode)?;
                false
            }
            Instruction::Lsr => {
                self.lsr(opcode.mode)?;
                false
            }
            Instruction::Rol => {
                self.rol(opcode.mode)?;
                false
            }
            Instruction::Ror => {
                self.ror(opcode.mode)?;
                false
            }

            Instruction::Inc => {
                self.inc(opcode.mode)?;
                false
            }
            Instruction::Dec => {
                self.dec(opcode.mode)?;
                false
            }
            Instruction::Inx => {
                self.register_x = self.register_x.wrapping_add(1);
                self.status.update_zero_and_negative(self.register_x);
                false
            }
            Instruction::Iny => {
                self.register_y = self.register_y.wrapping_add(1);
                self.status.update_zero_and_negative(self.register_y);
                false
            }
            Instruction::Dex => {
                self.register_x = self.register_x.wrapping_sub(1);
                self.status.update_zero_and_negative(self.register_x);
                false
            }
            Instruction::Dey => {
                self.register_y = self.register_y.wrapping_sub(1);
                self.status.update_zero_and_negative(self.register_y);
                false
            }

            Instruction::Cmp => {
                self.compare(opcode.mode, self.register_a)?;
                false
            }
            Instruction::Cpx => {
                self.compare(opcode.mode, self.register_x)?;
                false
            }
            Instruction::Cpy => {
                self.compare(opcode.mode, self.register_y)?;
                false
            }
            Instruction::Bit => {
                self.bit(opcode.mode)?;
                false
            }

            Instruction::Jmp => {
                if code == 0x6C {
                    let pointer = self.bus.read_u16(self.program_counter);
                    self.program_counter = self.read_u16_jmp_bug(pointer);
                } else {
                    self.program_counter = self.bus.read_u16(self.program_counter);
                }
                true
            }
            Instruction::Jsr => {
                self.stack_push_u16(self.program_counter.wrapping_add(1));
                self.program_counter = self.bus.read_u16(self.program_counter);
                true
            }
            Instruction::Rts => {
                self.program_counter = self.stack_pop_u16().wrapping_add(1);
                true
            }
            Instruction::Rti => {
                self.restore_status_from_stack();
                self.program_counter = self.stack_pop_u16();
                true
            }

            Instruction::Bne => self.branch(!self.status.contains(Status::ZERO)),
            Instruction::Bvs => self.branch(self.status.contains(Status::OVERFLOW)),
            Instruction::Bvc => self.branch(!self.status.contains(Status::OVERFLOW)),
            Instruction::Bmi => self.branch(self.status.contains(Status::NEGATIVE)),
            Instruction::Beq => self.branch(self.status.contains(Status::ZERO)),
            Instruction::Bcs => self.branch(self.status.contains(Status::CARRY)),
            Instruction::Bcc => self.branch(!self.status.contains(Status::CARRY)),
            Instruction::Bpl => self.branch(!self.status.contains(Status::NEGATIVE)),

            Instruction::Cld => {
                self.status.remove(Status::DECIMAL);
                false
            }
            Instruction::Cli => {
                self.status.remove(Status::INTERRUPT_DISABLE);
                false
            }
            Instruction::Clv => {
                self.status.remove(Status::OVERFLOW);
                false
            }
            Instruction::Clc => {
                self.status.remove(Status::CARRY);
                false
            }
            Instruction::Sec => {
                self.status.insert(Status::CARRY);
                false
            }
            Instruction::Sei => {
                self.status.insert(Status::INTERRUPT_DISABLE);
                false
            }
            Instruction::Sed => {
                self.status.insert(Status::DECIMAL);
                false
            }

            Instruction::Tax => {
                self.register_x = self.register_a;
                self.status.update_zero_and_negative(self.register_x);
                false
            }
            Instruction::Tay => {
                self.register_y = self.register_a;
                self.status.update_zero_and_negative(self.register_y);
                false
            }
            Instruction::Tsx => {
                self.register_x = self.stack_pointer;
                self.status.update_zero_and_negative(self.register_x);
                false
            }
            Instruction::Txa => {
                self.set_a(self.register_x);
                false
            }
            Instruction::Txs => {
                self.stack_pointer = self.register_x;
                false
            }
            Instruction::Tya => {
                self.set_a(self.register_y);
                false
            }

            Instruction::Pha => {
                self.stack_push(self.register_a);
                false
            }
            Instruction::Pla => {
                let value = self.stack_pop();
                self.set_a(value);
                false
            }
            Instruction::Php => {
                let flags = self.status.bits() | Status::BREAK | Status::UNUSED;
                self.stack_push(flags);
                false
            }
            Instruction::Plp => {
                self.restore_status_from_stack();
                false
            }

            Instruction::Illegal => unreachable!(),
        };

        if !pc_handled {
            self.program_counter = self
                .program_counter
                .wrapping_add(u16::from(opcode.len.saturating_sub(1)));
        }

        Ok(true)
    }

    fn read_operand(&self, mode: AddressingMode) -> Result<u8, CpuError> {
        Ok(self.bus.read(self.operand_address(mode)?))
    }

    fn set_a(&mut self, value: u8) {
        self.register_a = value;
        self.status.update_zero_and_negative(value);
    }

    fn add_to_a(&mut self, value: u8) {
        let carry = if self.status.contains(Status::CARRY) {
            1_u16
        } else {
            0
        };
        let sum = u16::from(self.register_a) + u16::from(value) + carry;
        let result = sum as u8;

        self.status.set(Status::CARRY, sum > 0xFF);
        self.status.set(
            Status::OVERFLOW,
            (value ^ result) & (result ^ self.register_a) & 0x80 != 0,
        );
        self.set_a(result);
    }

    fn asl(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        if mode == AddressingMode::Implied {
            let value = self.shift_left(self.register_a);
            self.register_a = value;
        } else {
            let address = self.operand_address(mode)?;
            let current = self.bus.read(address);
            let value = self.shift_left(current);
            self.bus.write(address, value);
        }
        Ok(())
    }

    fn lsr(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        if mode == AddressingMode::Implied {
            let value = self.shift_right(self.register_a);
            self.register_a = value;
        } else {
            let address = self.operand_address(mode)?;
            let current = self.bus.read(address);
            let value = self.shift_right(current);
            self.bus.write(address, value);
        }
        Ok(())
    }

    fn rol(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        if mode == AddressingMode::Implied {
            let value = self.rotate_left(self.register_a);
            self.register_a = value;
        } else {
            let address = self.operand_address(mode)?;
            let current = self.bus.read(address);
            let value = self.rotate_left(current);
            self.bus.write(address, value);
        }
        Ok(())
    }

    fn ror(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        if mode == AddressingMode::Implied {
            let value = self.rotate_right(self.register_a);
            self.register_a = value;
        } else {
            let address = self.operand_address(mode)?;
            let current = self.bus.read(address);
            let value = self.rotate_right(current);
            self.bus.write(address, value);
        }
        Ok(())
    }

    fn shift_left(&mut self, value: u8) -> u8 {
        self.status.set(Status::CARRY, value & 0x80 != 0);
        let result = value << 1;
        self.status.update_zero_and_negative(result);
        result
    }

    fn shift_right(&mut self, value: u8) -> u8 {
        self.status.set(Status::CARRY, value & 0x01 != 0);
        let result = value >> 1;
        self.status.update_zero_and_negative(result);
        result
    }

    fn rotate_left(&mut self, value: u8) -> u8 {
        let old_carry = u8::from(self.status.contains(Status::CARRY));
        self.status.set(Status::CARRY, value & 0x80 != 0);
        let result = (value << 1) | old_carry;
        self.status.update_zero_and_negative(result);
        result
    }

    fn rotate_right(&mut self, value: u8) -> u8 {
        let old_carry = if self.status.contains(Status::CARRY) {
            0x80
        } else {
            0
        };
        self.status.set(Status::CARRY, value & 0x01 != 0);
        let result = (value >> 1) | old_carry;
        self.status.update_zero_and_negative(result);
        result
    }

    fn inc(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        let address = self.operand_address(mode)?;
        let value = self.bus.read(address).wrapping_add(1);
        self.bus.write(address, value);
        self.status.update_zero_and_negative(value);
        Ok(())
    }

    fn dec(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        let address = self.operand_address(mode)?;
        let value = self.bus.read(address).wrapping_sub(1);
        self.bus.write(address, value);
        self.status.update_zero_and_negative(value);
        Ok(())
    }

    fn compare(&mut self, mode: AddressingMode, register: u8) -> Result<(), CpuError> {
        let value = self.read_operand(mode)?;
        self.status.set(Status::CARRY, register >= value);
        self.status
            .update_zero_and_negative(register.wrapping_sub(value));
        Ok(())
    }

    fn bit(&mut self, mode: AddressingMode) -> Result<(), CpuError> {
        let value = self.read_operand(mode)?;
        self.status.set(Status::ZERO, self.register_a & value == 0);
        self.status.set(Status::NEGATIVE, value & 0x80 != 0);
        self.status.set(Status::OVERFLOW, value & 0x40 != 0);
        Ok(())
    }

    fn branch(&mut self, condition: bool) -> bool {
        let offset = self.bus.read(self.program_counter) as i8;
        let next = self.program_counter.wrapping_add(1);
        self.program_counter = if condition {
            next.wrapping_add((offset as i16) as u16)
        } else {
            next
        };
        true
    }

    fn stack_push(&mut self, value: u8) {
        self.bus.write(
            STACK_BASE.wrapping_add(u16::from(self.stack_pointer)),
            value,
        );
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.bus
            .read(STACK_BASE.wrapping_add(u16::from(self.stack_pointer)))
    }

    fn stack_push_u16(&mut self, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop();
        let hi = self.stack_pop();
        u16::from_le_bytes([lo, hi])
    }

    fn restore_status_from_stack(&mut self) {
        self.status = Status::from_bits(self.stack_pop());
        self.status.remove(Status::BREAK);
        self.status.insert(Status::UNUSED);
    }

    fn read_u16_jmp_bug(&self, address: u16) -> u16 {
        let lo = self.bus.read(address);
        let hi_address = if address & 0x00FF == 0x00FF {
            address & 0xFF00
        } else {
            address.wrapping_add(1)
        };
        let hi = self.bus.read(hi_address);
        u16::from_le_bytes([lo, hi])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuError {
    Bus(BusError),
    UnknownOpcode { opcode: u8, address: u16 },
    ProgramTooLarge { len: usize, max: usize },
    UnsupportedAddressingMode,
}

impl From<BusError> for CpuError {
    fn from(value: BusError) -> Self {
        Self::Bus(value)
    }
}

impl core::fmt::Display for CpuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bus(err) => write!(f, "bus error: {err}"),
            Self::UnknownOpcode { opcode, address } => {
                write!(f, "unknown opcode {opcode:#04X} at {address:#06X}")
            }
            Self::ProgramTooLarge { len, max } => {
                write!(
                    f,
                    "program is {len} bytes; maximum loader size is {max} bytes"
                )
            }
            Self::UnsupportedAddressingMode => f.write_str("unsupported addressing mode"),
        }
    }
}

impl std::error::Error for CpuError {}

impl Cpu<FlatMemory> {
    #[must_use]
    pub fn with_flat_memory() -> Self {
        Self::new(FlatMemory::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lda_immediate_loads_data_and_flags() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[0xA9, 0x05, 0x00]).unwrap();

        assert_eq!(cpu.a(), 0x05);
        assert!(!cpu.status().contains(Status::ZERO));
        assert!(!cpu.status().contains(Status::NEGATIVE));
    }

    #[test]
    fn lda_zero_sets_zero_flag() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[0xA9, 0x00, 0x00]).unwrap();

        assert!(cpu.status().contains(Status::ZERO));
    }

    #[test]
    fn tax_and_inx_work_together() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA9, 0xFF, // LDA #$FF
            0xAA, // TAX
            0xE8, // INX -> 0
            0xE8, // INX -> 1
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.x(), 0x01);
    }

    #[test]
    fn lda_from_zero_page_memory() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.bus_mut().write(0x10, 0x55);
        cpu.load_and_run(&[0xA5, 0x10, 0x00]).unwrap();

        assert_eq!(cpu.a(), 0x55);
    }

    #[test]
    fn adc_and_sbc_update_accumulator() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA9, 0x10, // LDA #$10
            0x69, 0x05, // ADC #$05
            0x38, // SEC: SBC subtracts without borrow
            0xE9, 0x03, // SBC #$03
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0x12);
    }

    #[test]
    fn logic_instructions_work() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA9,
            0b1010_1010,
            0x29,
            0b1111_0000, // AND => 1010_0000
            0x49,
            0b0011_0000, // EOR => 1001_0000
            0x09,
            0b0000_0011, // ORA => 1001_0011
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0b1001_0011);
        assert!(cpu.status().contains(Status::NEGATIVE));
    }

    #[test]
    fn shifts_and_rotates_work_on_accumulator() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA9, 0x81, // 1000_0001
            0x0A, // ASL => 0000_0010, C=1
            0x2A, // ROL => 0000_0101, C=0
            0x4A, // LSR => 0000_0010, C=1
            0x6A, // ROR => 1000_0001
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0x81);
    }

    #[test]
    fn inc_dec_and_compare_work() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.bus_mut().write(0x20, 0x0A);
        cpu.load_and_run(&[
            0xE6, 0x20, // INC $20 => 0B
            0xC6, 0x20, // DEC $20 => 0A
            0xA9, 0x0A, 0xC5, 0x20, // CMP $20
            0x00,
        ])
        .unwrap();

        assert!(cpu.status().contains(Status::ZERO));
        assert!(cpu.status().contains(Status::CARRY));
    }

    #[test]
    fn ldx_ldy_and_store_variants_work() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA2, 0x11, // LDX #$11
            0xA0, 0x22, // LDY #$22
            0x86, 0x30, // STX $30
            0x84, 0x31, // STY $31
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.bus.read(0x30), 0x11);
        assert_eq!(cpu.bus.read(0x31), 0x22);
    }

    #[test]
    fn branch_consumes_relative_operand() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA9, 0x00, // Z = 1
            0xF0, 0x02, // BEQ +2, skip LDA #$01
            0xA9, 0x01, 0xA9, 0x02, 0x00,
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0x02);
    }

    #[test]
    fn jsr_and_rts_restore_program_counter() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0x20, 0x06, 0x80, // JSR $8006
            0xA9, 0x03, // LDA #$03 after return
            0x00, 0xA9, 0x07, // $8006: LDA #$07
            0x60, // RTS
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0x03);
    }

    #[test]
    fn stack_push_and_pull_accumulator() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA9, 0x42, 0x48, // PHA
            0xA9, 0x00, 0x68, // PLA
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0x42);
    }

    #[test]
    fn bit_copies_negative_and_overflow_flags() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.bus_mut().write(0x40, 0b1100_0000);
        cpu.load_and_run(&[0xA9, 0x01, 0x24, 0x40, 0x00]).unwrap();

        assert!(cpu.status().contains(Status::ZERO));
        assert!(cpu.status().contains(Status::NEGATIVE));
        assert!(cpu.status().contains(Status::OVERFLOW));
    }

    #[test]
    fn unknown_opcode_returns_error() {
        let mut cpu = Cpu::with_flat_memory();
        let err = cpu.load_and_run(&[0x02]).unwrap_err();

        assert!(matches!(err, CpuError::UnknownOpcode { opcode: 0x02, .. }));
    }
    #[test]
    fn negative_relative_branch_works() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0xA2, 0x02, // LDX #$02
            0xCA, // DEX
            0xD0, 0xFD, // BNE -3 -> DEX
            0x00,
        ])
        .unwrap();

        assert_eq!(cpu.x(), 0x00);
    }

    #[test]
    fn php_and_plp_restore_flags() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.load_and_run(&[
            0x38, // SEC
            0x08, // PHP
            0x18, // CLC
            0x28, // PLP
            0x00,
        ])
        .unwrap();

        assert!(cpu.status().contains(Status::CARRY));
        assert!(!cpu.status().contains(Status::BREAK));
        assert!(cpu.status().contains(Status::UNUSED));
    }

    #[test]
    fn jmp_indirect_keeps_6502_page_boundary_bug() {
        let mut cpu = Cpu::with_flat_memory();
        cpu.bus_mut().write(0x30FF, 0x06);
        cpu.bus_mut().write(0x3000, 0x80);
        cpu.bus_mut().write(0x3100, 0x99);

        cpu.load_and_run(&[
            0x6C, 0xFF, 0x30, // JMP ($30FF) -> $8006 because of the 6502 bug
            0x00, 0x00, 0x00, 0xA9, 0x42, 0x00,
        ])
        .unwrap();

        assert_eq!(cpu.a(), 0x42);
    }
}
