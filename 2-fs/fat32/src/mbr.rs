use std::{fmt, io, mem};

use traits::BlockDevice;
use util::Unused;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CHS {
    head: u8,
    sector_cylinder_upper: u8,
    cylinder_lower: u8,
}

impl CHS {
    pub fn sector(self) -> u8 {
        self.sector_cylinder_upper & 0b00111111
    }

    pub fn cylinder(self) -> u16 {
        let cylinder_upper = (self.sector_cylinder_upper & 0b11000000) as u16;
        cylinder_upper | self.cylinder_lower as u16
    }
}

impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .field("head", &self.head)
            .field("sector", &self.sector())
            .field("cylinder", &self.cylinder())
            .finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootIndicator(u8);

impl BootIndicator {
    const NO: u8 = 0x00;
    const ACTIVE: u8 = 0x80;

    pub fn is_valid(&self) -> bool {
        match self.0 {
            BootIndicator::NO => true,
            BootIndicator::ACTIVE => true,
            _ => false,
        }
    }
}

impl fmt::Debug for BootIndicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BootIndicator({} ({})", match self.0 {
            BootIndicator::NO => "NO",
            BootIndicator::ACTIVE => "ACTIVE",
            _ => "Unknown",
        }, self.0)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum PartitionType {
    Fat32 = 0x0B,
    Fat32Alt = 0x0C,
}

#[repr(C, packed)]
#[derive(Clone, Debug)]
pub struct PartitionEntry {
    boot_indicator: BootIndicator,
    starting_chs: CHS,
    partition_type: u8,
    ending_chs: CHS,
    relative_sector: u32,
    total_sectors: u32,
}

/// The master boot record (MBR).
#[repr(C, packed)]
#[derive(Debug)]
pub struct MasterBootRecord {
    bootstrap: Unused<[u8; 436]>,
    disk_id: [u8; 10],
    partition_table: [PartitionEntry; 4],
    signature: u16,
}

const MBR_SIZE: usize = 512;
const FAT32_SIGNATURE: u16 = 0xAA55;

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partition `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occurred while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut sector = [0u8; MBR_SIZE];
        let bytes = device.read_sector(0, &mut sector)?;

        if bytes != MBR_SIZE {
            return Err(Error::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "MBR too short")))
        }

        let mbr: MasterBootRecord = unsafe { mem::transmute(sector) };
        if mbr.signature != FAT32_SIGNATURE {
            return Err(Error::BadSignature);
        }

        for i in 0..4 {
            let ref partition = mbr.partition_table[i];
            if !partition.boot_indicator.is_valid() {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }

        Ok(mbr)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn test_block_size_small() {
        let mut data = [0u8; 511];
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("EOF") {
            Error::Io(io) => assert_eq!(io.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(false, "Invalid error"),
        }
    }

    #[test]
    fn test_simple_block() {
        let mut data = [0u8; 512];
        // Partition entries.
        data[446] = 0x00;
        data[462] = 0x00;
        data[478] = 0x00;
        data[494] = 0x00;
        // Signature.
        data[510] = 0x55;
        data[511] = 0xAA;
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..])).expect("Valid block");
    }

    #[test]
    fn test_invalid_signature() {
        let mut data = [0u8; 512];
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("Signature") {
            Error::BadSignature => (),
            _ => assert!(false, "Invalid error"),
        }

        data[510] = 0x55;
        data[511] = 0xAB;
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("Signature") {
            Error::BadSignature => (),
            _ => assert!(false, "Invalid error"),
        }

        data[511] = 0xAA;
        data[446] = 0x01;
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("Signature") {
            Error::UnknownBootIndicator(0) => (),
            _ => assert!(false, "Invalid error"),
        }

        data[511] = 0xAA;
        data[446] = 0x00;
        data[462] = 0x01;
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("Signature") {
            Error::UnknownBootIndicator(1) => (),
            _ => assert!(false, "Invalid error"),
        }

        data[462] = 0x00;
        data[478] = 0x01;
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("Signature") {
            Error::UnknownBootIndicator(2) => (),
            _ => assert!(false, "Invalid error"),
        }

        data[478] = 0x00;
        data[494] = 0x01;
        let result = MasterBootRecord::from(
            Cursor::new(&mut data[..]));
        match result.expect_err("Signature") {
            Error::UnknownBootIndicator(3) => (),
            _ => assert!(false, "Invalid error"),
        }
    }
}

