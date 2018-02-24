use std::{fmt, io};

use traits::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CHS {
    // FIXME: Fill me in.
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    // FIXME: Fill me in.
}

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    // FIXME: Fill me in.
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        unimplemented!("MasterBootRecord::from()")
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!("MasterBootRecord::fmt()")
    }
}
