use std::io::{self, SeekFrom};
use std::cmp::min;

use traits;
use vfat::{VFat, Shared, Cluster};

#[derive(Debug)]
pub struct File {
    start: Cluster,
    vfat: Shared<VFat>,
    size: u32,

    pointer: u64,

    cluster_current: Cluster,
    cluster_current_start: usize,
}

impl File {
    // Create a new file.
    pub fn new(start: Cluster, vfat: Shared<VFat>, size: u32)
        -> File
    {
        File { start, vfat, size, pointer: 0, cluster_current: start,
               cluster_current_start: 0 }
    }

    fn set_pointer(&mut self, pointer: u64) -> io::Result<u64> {
        self.pointer = pointer;

        let (cluster, cluster_start) = self.vfat.borrow_mut().find_sector(
            self.start, self.pointer as usize)?;
        self.cluster_current = cluster;
        self.cluster_current_start = cluster_start;

        Ok(self.pointer)
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
        let mut bytes_read: usize = 0;
        let max_read = min(self.size as usize - self.pointer as usize,
                           buf.len());

        while self.pointer < self.size as u64 {
            let bytes = self.vfat.borrow_mut().read_cluster(
                self.cluster_current,
                self.pointer as usize - self.cluster_current_start,
                &mut buf[bytes_read..max_read])?;
            if bytes == 0 {
                break;
            }

            bytes_read += bytes;
            self.pointer += bytes as u64;

            let (cluster, offset) = self.vfat.borrow_mut().find_sector(
                self.cluster_current,
                self.pointer as usize - self.cluster_current_start)?;
            self.cluster_current = cluster;
            self.cluster_current_start += offset;
        }

        Ok(bytes_read)
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
                    self.set_pointer(offset)
                }
            },
            SeekFrom::End(offset) => {
                if offset > 0 || offset + (self.size as i64) < 0 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       "Out of bounds"))
                } else {
                    let pointer = (self.size as i64 + offset) as u64;
                    self.set_pointer(pointer)
                }
            },
            SeekFrom::Current(offset) => {
                if offset >= 0 && offset as u64 + self.pointer
                                    <= self.size as u64 {
                    let pointer = self.pointer + offset as u64;
                    self.set_pointer(pointer)
                } else if offset < 0 && (-offset as u64) <= self.pointer {
                    let pointer = self.pointer - (-offset as u64);
                    self.set_pointer(pointer)
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       "Out of bounds"))
                }
            }
        }
    }
}
