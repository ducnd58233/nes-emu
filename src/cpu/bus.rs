pub const ADDRESS_SPACE_SIZE: usize = 0x1_00000;

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
}

impl core::fmt::Display for BusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfRange => f.write_str("memory range exceeds the 16-bit address space"),
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
