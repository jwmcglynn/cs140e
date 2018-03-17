use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
pub struct File {
    start: Cluster,
    vfat: Shared<VFat>,
    name: String,
    metadata: Metadata,
    size: u32,

    pointer: u64,
}

impl File {
    // Create a new file, called by Dir.
    pub fn new(start: Cluster, vfat: Shared<VFat>, name: String,
               metadata: Metadata, size: u32)
        -> File
    {
        File { start, vfat, name, metadata, size, pointer: 0 }
    }

    /// Returns the file name.
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl traits::File for File {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()> {
        // No-ops, this is a write-only-to-memory filesystem.
        Ok(())
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.size as u64
    }
}

impl io::Read for File {
    /// Read from the file into a buffer.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.vfat.borrow_mut().read_cluster(self.start, self.pointer as usize,
                                            buf)
    }
}

impl io::Write for File {
    /// Write into the file.
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("write")
    }

    /// Flush the file changes to disk, not implemented for in-memory
    /// cached filesystem.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => {
                if offset > self.size as u64 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       "Out of bounds"))
                } else {
                    self.pointer = offset;
                    Ok(self.pointer)
                }
            },
            SeekFrom::End(offset) => {
                if offset > 0 || offset + (self.size as i64) < 0 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       "Out of bounds"))
                } else {
                    self.pointer = (self.size as i64 + offset) as u64;
                    Ok(self.pointer)
                }
            },
            SeekFrom::Current(offset) => {
                if offset >= 0 && offset as u64 + self.pointer
                                    <= self.size as u64 {
                    self.pointer += offset as u64;
                    Ok(self.pointer)
                } else if offset < 0 && (-offset as u64) <= self.pointer {
                    self.pointer -= -offset as u64;
                    Ok(self.pointer)
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       "Out of bounds"))
                }
            }
        }
    }
}
