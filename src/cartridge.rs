const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;

const PRG_ROM_PAGE_SIZE: usize = 16 * 1024;
const CHR_ROM_PAGE_SIZE: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rom {
    pub prg_rom: Box<[u8]>,
    pub chr_rom: Box<[u8]>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn parse(raw: &[u8]) -> Result<Self, RomError> {
        if raw.len() < HEADER_SIZE {
            return Err(RomError::HeaderTooShort { len: raw.len() });
        }

        if raw[..4] != NES_TAG {
            return Err(RomError::InvalidFormat);
        }

        let mapper = (raw[7] & 0xF0) | (raw[6] >> 4);
        let ines_version = (raw[7] >> 2) & 0b11;

        if ines_version != 0 {
            return Err(RomError::UnsupportedInesVersion(ines_version));
        }

        let four_screen = raw[6] & 0b10000 != 0;
        let vertical_mirroring = raw[6] & 0b0001 != 0;

        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = usize::from(raw[4]) * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = usize::from(raw[5]) * CHR_ROM_PAGE_SIZE;

        let has_trainer = raw[6] & 0b0100 != 0;

        let prg_rom_start = HEADER_SIZE + if has_trainer { TRAINER_SIZE } else { 0 };
        let chr_rom_start = prg_rom_start
            .checked_add(prg_rom_size)
            .ok_or(RomError::InvalidSize)?;

        let end = chr_rom_start
            .checked_add(chr_rom_size)
            .ok_or(RomError::InvalidSize)?;

        if end > raw.len() {
            return Err(RomError::Truncated {
                expected: end,
                actual: raw.len(),
            });
        }

        Ok(Self {
            prg_rom: raw[prg_rom_start..chr_rom_start]
                .to_vec()
                .into_boxed_slice(),
            chr_rom: raw[chr_rom_start..end].to_vec().into_boxed_slice(),
            mapper,
            screen_mirroring,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomError {
    HeaderTooShort { len: usize },
    InvalidFormat,
    UnsupportedInesVersion(u8),
    InvalidSize,
    Truncated { expected: usize, actual: usize },
}

impl core::fmt::Display for RomError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::HeaderTooShort { len } => {
                write!(f, "ROM header is too short: {len} bytes")
            }

            Self::InvalidFormat => f.write_str("file is not in iNES format"),

            Self::UnsupportedInesVersion(version) => {
                write!(f, "unsupported iNES version: {version}")
            }

            Self::InvalidSize => f.write_str("ROM size overflow"),

            Self::Truncated { expected, actual } => {
                write!(
                    f,
                    "ROM is truncated: expected at least \
                     {expected} bytes, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for RomError {}
