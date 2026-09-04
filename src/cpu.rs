use crate::cpu::{
    bus::{Bus, BusError, FlatMemory},
    opcode::{Instruction, Opcode, decode},
    status::Status,
};

mod addressing;
pub mod bus;
pub mod opcode;
pub mod status;

const PROGRAM_START: u16 = 0x8000;
const RESET_VECTOR: u16 = 0xFFFC;
const MAX_PROGRAM_SIZE: usize = (RESET_VECTOR - PROGRAM_START) as usize;

#[derive(Debug, Clone)]
pub struct Cpu<B> {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: Status,
    program_counter: u16,
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

    pub fn load_and_run(&mut self, program: &[u8]) -> Result<RunStats, CpuError> {
        self.load(program);
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

    pub fn run(&mut self) -> Result<RunStats, CpuError> {
        let mut stats = RunStats::default();

        loop {
            match self.step()? {
                StepOutcome::Executed { cycles } => {
                    stats.instructions = stats.instructions.saturating_add(1);
                    stats.cycles = stats.cycles.saturating_add(u64::from(cycles));
                }
                StepOutcome::Halted { cycles } => {
                    stats.instructions = stats.instructions.saturating_add(1);
                    stats.cycles = stats.cycles.saturating_add(u64::from(cycles));
                    return Ok(stats);
                }
            }
        }
    }

    /// Fetch, decode and execute 1 instructions
    pub fn step(&mut self) -> Result<StepOutcome, CpuError> {
        // Fetch
        let opcode_addr = self.program_counter;
        let code = self.bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);

        // decode
        let opcode = decode(code);
        if opcode.instruction == Instruction::Illegal {
            return Err(CpuError::UnknownOpcode {
                opcode: code,
                address: opcode_addr,
            });
        }

        // execute
        let operand_pc = self.program_counter;
        let execution = self.execution(opcode)?;

        if !execution.pc_changed {
            self.program_counter = operand_pc.wrapping_add(u16::from(opcode.len.saturating_sub(1)));
        }

        let extra_cycle = u8::from(opcode.page_cross_penalty() && execution.page_crossed);
        let cycles = opcode.cycles.saturating_add(extra_cycle);

        if execution.halted {
            Ok(StepOutcome::Halted { cycles })
        } else {
            Ok(StepOutcome::Executed { cycles })
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = Status::default();
        self.program_counter = self.bus.read_u16(RESET_VECTOR);
    }

    fn execution(&mut self, opcode: Opcode) -> Result<Execution, CpuError> {
        match opcode.instruction {
            Instruction::Lda => {
                let operand = self.operand_address(opcode.mode)?;
                self.register_a = self.bus.read(operand.address);
                self.status.update_zero_and_negative(self.register_a);
                Ok(Execution::normal(operand.page_crossed))
            }
            Instruction::Sta => {
                let operand = self.operand_address(opcode.mode)?;
                self.bus.write(operand.address, self.register_a);
                Ok(Execution::normal(operand.page_crossed))
            }
            Instruction::Tax => {
                self.register_x = self.register_a;
                self.status.update_zero_and_negative(self.register_x);
                Ok(Execution::normal(false))
            }
            Instruction::Inx => {
                self.register_x = self.register_x.wrapping_add(1);
                self.status.update_zero_and_negative(self.register_x);
                Ok(Execution::normal(false))
            }
            Instruction::Brk => Ok(Execution {
                halted: true,
                page_crossed: false,
                pc_changed: false,
            }),
            Instruction::Illegal => Err(CpuError::UnknownOpcode {
                opcode: self.bus.read(self.program_counter.wrapping_sub(1)),
                address: self.program_counter.wrapping_sub(1),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Executed { cycles: u8 },
    Halted { cycles: u8 },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RunStats {
    pub instructions: u64,
    pub cycles: u64,
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct OperandAddress {
    pub address: u16,
    pub page_crossed: bool,
}

#[derive(Debug, Clone, Copy)]
struct Execution {
    halted: bool,
    page_crossed: bool,
    pc_changed: bool,
}

impl Execution {
    const fn normal(page_crossed: bool) -> Self {
        Self {
            halted: false,
            page_crossed,
            pc_changed: false,
        }
    }
}

impl Cpu<FlatMemory> {
    #[must_use]
    pub fn with_flat_memory() -> Self {
        Self::new(FlatMemory::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = Cpu::with_flat_memory();

        cpu.load_and_run(&[0xA9, 0x05, 0x00]).unwrap();

        assert_eq!(cpu.a(), 0x05);
        assert!(!cpu.status().contains(Status::ZERO));
        assert!(!cpu.status().contains(Status::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flags() {
        let mut cpu = Cpu::with_flat_memory();

        cpu.load_and_run(&[0xA9, 0x00, 0x00]).unwrap();

        assert!(cpu.status().contains(Status::ZERO));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = Cpu::with_flat_memory();

        cpu.load_and_run(&[
            0xA9, 0x0A, // LDA #$0A
            0xAA, // TAX
            0x00, // BRK
        ])
        .unwrap();

        assert_eq!(cpu.x(), 0x0A);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = Cpu::with_flat_memory();

        cpu.load_and_run(&[
            0xA9, 0xFF, // LDA #$FF
            0xAA, // TAX -> X = $FF
            0xE8, // INX -> $00
            0xE8, // INX -> $01
            0x00, // BRK
        ])
        .unwrap();

        assert_eq!(cpu.x(), 0x01);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = Cpu::with_flat_memory();

        cpu.load_and_run(&[
            0xA9, 0xC0, // LDA #$C0
            0xAA, // TAX -> X = $C0
            0xE8, // INX -> X = $C1
            0x00, // BRK
        ])
        .unwrap();

        assert_eq!(cpu.x(), 0xC1);
    }
}
