//  _______________ $10000  _______________
// | PRG-ROM       |       |               |
// | Upper Bank    |       |               |
// |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
// | PRG-ROM       |       |               |
// | Lower Bank    |       |               |
// |_______________| $8000 |_______________|
// | SRAM          |       | SRAM          |
// |_______________| $6000 |_______________|
// | Expansion ROM |       | Expansion ROM |
// |_______________| $4020 |_______________|
// | I/O Registers |       |               |
// |_ _ _ _ _ _ _ _| $4000 |               |
// | Mirrors       |       | I/O Registers |
// | $2000-$2007   |       |               |
// |_ _ _ _ _ _ _ _| $2008 |               |
// | I/O Registers |       |               |
// |_______________| $2000 |_______________|
// | Mirrors       |       |               |
// | $0000-$07FF   |       |               |
// |_ _ _ _ _ _ _ _| $0800 |               |
// | RAM           |       | RAM           |
// |_ _ _ _ _ _ _ _| $0200 |               |
// | Stack         |       |               |
// |_ _ _ _ _ _ _ _| $0100 |               |
// | Zero Page     |       |               |
// |_______________| $0000 |_______________|

use crate::cartridge::Rom;

pub const ADDRESS_SPACE_SIZE: usize = 0x1_00000;
const CPU_RAM_SIZE: usize = 0x0800;

const RAM_START: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;

const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

const PRG_ROM_START: u16 = 0x8000;

pub trait Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    #[inline]
    fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr.wrapping_add(1));
        u16::from_le_bytes([lo, hi])
    }

    #[inline]
    fn write_u16(&mut self, addr: u16, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }

    fn load(&mut self, start: u16, bytes: &[u8]) -> Result<(), BusError> {
        let start = usize::from(start);
        let end = start.checked_add(bytes.len()).ok_or(BusError::OutOfRange)?;
        if end > ADDRESS_SPACE_SIZE {
            return Err(BusError::OutOfRange);
        }

        for (offset, &byte) in bytes.iter().enumerate() {
            self.write((start + offset) as u16, byte);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusError {
    OutOfRange,
    UnsupportedMapper { mapper: u8 },
}

impl core::fmt::Display for BusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfRange => f.write_str("memory range exceeds the 16-bit address space"),
            Self::UnsupportedMapper { mapper } => {
                write!(f, "unsupported NES mapper: {mapper}")
            }
        }
    }
}

impl std::error::Error for BusError {}

#[derive(Clone)]
pub struct FlatMemory {
    data: Box<[u8; ADDRESS_SPACE_SIZE]>,
}

impl FlatMemory {
    #[must_use]
    pub fn new() -> Self {
        let data = vec![0; ADDRESS_SPACE_SIZE]
            .into_boxed_slice()
            .try_into()
            .expect("ADDRESS_SPACE_SIZE is exactly the vector length");
        Self { data }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl Bus for FlatMemory {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        self.data[usize::from(addr)]
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) {
        self.data[usize::from(addr)] = data;
    }

    fn load(&mut self, start: u16, bytes: &[u8]) -> Result<(), BusError> {
        let start = usize::from(start);
        let end = start.checked_add(bytes.len()).ok_or(BusError::OutOfRange)?;
        let dst = self.data.get_mut(start..end).ok_or(BusError::OutOfRange)?;
        dst.copy_from_slice(bytes);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NesBus {
    cpu_ram: [u8; CPU_RAM_SIZE],
    rom: Rom,
}

impl NesBus {
    #[must_use]
    pub fn new(rom: Rom) -> Result<Self, BusError> {
        if rom.mapper != 0 {
            return Err(BusError::UnsupportedMapper { mapper: rom.mapper });
        }
        Ok(Self {
            cpu_ram: [0; CPU_RAM_SIZE],
            rom,
        })
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        debug_assert!(addr >= PRG_ROM_START);

        let mut offset = usize::from(addr - PRG_ROM_START);

        if self.rom.prg_rom.len() == 0x4000 {
            offset %= 0x4000;
        }

        self.rom.prg_rom[offset]
    }
}

impl Bus for NesBus {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_MIRRORS_END => {
                let mirrored_addr = addr & 0x07FF;
                self.cpu_ram[usize::from(mirrored_addr)]
            }

            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let _mirrored_addr = PPU_REGISTERS_START | (addr & 0x0007);

                todo!("PPU is not supported yet")
            }

            PRG_ROM_START..=0xFFFF => self.read_prg_rom(addr),

            _ => 0,
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_MIRRORS_END => {
                let mirrored_addr = addr & 0x07FF;
                self.cpu_ram[usize::from(mirrored_addr)] = data;
            }

            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let _mirrored_addr = PPU_REGISTERS_START | (addr & 0x0007);

                todo!("PPU is not supported yet")
            }

            PRG_ROM_START..=0xFFFF => {
                panic!("attempt to write to catridge PRG-ROM")
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::Mirroring;

    fn test_rom(prg_rom: Vec<u8>) -> Rom {
        Rom {
            prg_rom: prg_rom.into_boxed_slice(),
            chr_rom: vec![0; 0x2000].into_boxed_slice(),
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
        }
    }

    #[test]
    fn cpu_ram_is_mirrored() {
        let rom = test_rom(vec![0; 0x8000]);
        let mut bus = NesBus::new(rom).unwrap();

        bus.write(0x0005, 0x42);

        assert_eq!(bus.read(0x0005), 0x42);
        assert_eq!(bus.read(0x0805), 0x42);
        assert_eq!(bus.read(0x1005), 0x42);
        assert_eq!(bus.read(0x1805), 0x42);
    }

    #[test]
    fn writing_to_mirror_updates_base_ram() {
        let rom = test_rom(vec![0; 0x8000]);
        let mut bus = NesBus::new(rom).unwrap();

        bus.write(0x1805, 0x69);

        assert_eq!(bus.read(0x0005), 0x69);
    }

    #[test]
    fn mirrors_16kb_prg_rom() {
        let mut prg_rom = vec![0; 0x4000];
        prg_rom[0x0123] = 0x42;

        let rom = test_rom(prg_rom);
        let bus = NesBus::new(rom).unwrap();

        assert_eq!(bus.read(0x8123), 0x42);
        assert_eq!(bus.read(0xC123), 0x42);
    }

    #[test]
    fn does_not_mirror_32kb_prg_rom() {
        let mut prg_rom = vec![0; 0x8000];
        prg_rom[0x0000] = 0x11;
        prg_rom[0x4000] = 0x22;

        let rom = test_rom(prg_rom);
        let bus = NesBus::new(rom).unwrap();

        assert_eq!(bus.read(0x8000), 0x11);
        assert_eq!(bus.read(0xC000), 0x22);
    }
}
