use crate::cpu::{Cpu, CpuError, bus::Bus, opcode::AddressingMode};

impl<B: Bus> Cpu<B> {
    pub(crate) fn operand_address(&self, mode: AddressingMode) -> Result<u16, CpuError> {
        let address = match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => u16::from(self.bus.read(self.program_counter)),
            AddressingMode::ZeroPageX => {
                let base = self.bus.read(self.program_counter);
                u16::from(base.wrapping_add(self.register_x))
            }
            AddressingMode::ZeroPageY => {
                let base = self.bus.read(self.program_counter);
                u16::from(base.wrapping_add(self.register_y))
            }
            AddressingMode::Absolute => self.bus.read_u16(self.program_counter),
            AddressingMode::AbsoluteX => self
                .bus
                .read_u16(self.program_counter)
                .wrapping_add(u16::from(self.register_x)),
            AddressingMode::AbsoluteY => self
                .bus
                .read_u16(self.program_counter)
                .wrapping_add(u16::from(self.register_y)),
            AddressingMode::IndirectX => {
                let base = self.bus.read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x);
                let lo = self.bus.read(u16::from(ptr));
                let hi = self.bus.read(u16::from(ptr.wrapping_add(1)));
                u16::from_le_bytes([lo, hi])
            }
            AddressingMode::IndirectY => {
                let ptr = self.bus.read(self.program_counter);
                let lo = self.bus.read(u16::from(ptr));
                let hi = self.bus.read(u16::from(ptr.wrapping_add(1)));
                u16::from_le_bytes([lo, hi]).wrapping_add(u16::from(self.register_y))
            }
            AddressingMode::Implied => return Err(CpuError::UnsupportedAddressingMode),
        };

        Ok(address)
    }
}
