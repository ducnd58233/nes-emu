use crate::cpu::{Cpu, CpuError, OperandAddress, bus::Bus, opcode::AddressingMode};

impl<B: Bus> Cpu<B> {
    pub(crate) fn operand_address(&self, mode: AddressingMode) -> Result<OperandAddress, CpuError> {
        let result = match mode {
            AddressingMode::Immediate => OperandAddress {
                address: self.program_counter,
                page_crossed: false,
            },
            AddressingMode::ZeroPage => OperandAddress {
                address: u16::from(self.bus.read(self.program_counter)),
                page_crossed: false,
            },
            AddressingMode::ZeroPageX => {
                let base = self.bus.read(self.program_counter);
                OperandAddress {
                    address: u16::from(base.wrapping_add(self.register_x)),
                    page_crossed: false,
                }
            }
            AddressingMode::ZeroPageY => {
                let base = self.bus.read(self.program_counter);
                OperandAddress {
                    address: u16::from(base.wrapping_add(self.register_y)),
                    page_crossed: false,
                }
            }
            AddressingMode::Absolute => OperandAddress {
                address: self.bus.read_u16(self.program_counter),
                page_crossed: false,
            },
            AddressingMode::AbsoluteX => {
                let base = self.bus.read_u16(self.program_counter);
                let address = base.wrapping_add(u16::from(self.register_x));
                OperandAddress {
                    address,
                    page_crossed: page_crossed(base, address),
                }
            }
            AddressingMode::AbsoluteY => {
                let base = self.bus.read_u16(self.program_counter);
                let address = base.wrapping_add(u16::from(self.register_y));
                OperandAddress {
                    address,
                    page_crossed: page_crossed(base, address),
                }
            }
            AddressingMode::IndirectX => {
                // Zero-page pointer arithmetic wraps inside 0x00..=0xFF.
                let base = self.bus.read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x);
                let lo = self.bus.read(u16::from(ptr));
                let hi = self.bus.read(u16::from(ptr.wrapping_add(1)));
                OperandAddress {
                    address: u16::from_le_bytes([lo, hi]),
                    page_crossed: false,
                }
            }
            AddressingMode::IndirectY => {
                let ptr = self.bus.read(self.program_counter);
                let lo = self.bus.read(u16::from(ptr));
                let hi = self.bus.read(u16::from(ptr.wrapping_add(1)));
                let base = u16::from_le_bytes([lo, hi]);
                let address = base.wrapping_add(u16::from(self.register_y));
                OperandAddress {
                    address,
                    page_crossed: page_crossed(base, address),
                }
            }
            AddressingMode::Implied => return Err(CpuError::UnsupportedAddressingMode),
        };

        Ok(result)
    }
}

#[inline]
const fn page_crossed(base: u16, address: u16) -> bool {
    (base & 0xFF00) != (address & 0xFF00)
}
