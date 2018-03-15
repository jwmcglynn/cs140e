use std::{io, fmt};
use std::io::Write;
use std::collections::HashMap;

use traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool
}

#[derive(Debug)]
pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `n` _before_
    /// `partition.start` is made to physical sector `n`. Cached sectors before
    /// `partition.start` are the size of a physical sector. An access to a
    /// sector `n` at or after `partition.start` is made to the _logical_ sector
    /// `n - partition.start`. Cached sectors at or after `partition.start` are
    /// the size of a logical sector, `partition.sector_size`.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedDevice
        where T: BlockDevice + 'static
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition
        }
    }

    /// Maps a user's request for a sector `virt` to the physical sector and
    /// number of physical sectors required to access `virt`.
    fn virtual_to_physical(&self, virt: u64) -> (u64, u64) {
        if self.device.sector_size() == self.partition.sector_size {
            (virt, 1)
        } else if virt < self.partition.start {
            (virt, 1)
        } else {
            let factor = self.partition.sector_size / self.device.sector_size();
            let logical_offset = virt - self.partition.start;
            let physical_offset = logical_offset * factor;
            let physical_sector = self.partition.start + physical_offset;
            (physical_sector, factor)
        }
    }

    /// Loads the sector to the cache, if it is not already loaded.
    fn ensure_sector(&mut self, sector: u64) -> io::Result<()> {
        if !self.cache.contains_key(&sector) {
            let physical = self.virtual_to_physical(sector);
            let mut data = Vec::new();

            for i in 0..physical.1 {
                self.device.read_all_sector(physical.0 + i, &mut data)?;
            }

            self.cache.insert(sector, CacheEntry { data, dirty: false });
        }

        Ok(())
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        self.ensure_sector(sector)?;
        let entry = self.cache.get_mut(&sector).unwrap();

        entry.dirty = true;
        Ok(entry.data.as_mut_slice())
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        self.ensure_sector(sector)?;
        let entry = self.cache.get(&sector).unwrap();

        Ok(entry.data.as_slice())
    }
}

impl BlockDevice for CachedDevice {
    /// The size of sectors in the partition, reads may be smaller if they are
    /// outside the partition.
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }

    /// Read sector number `n` into `buf`.
    fn read_sector(&mut self, n: u64, mut buf: &mut [u8]) -> io::Result<usize> {
        let data = self.get(n)?;
        buf.write(data)
    }

    /// Overwrites sector `n` with the contents of `buf`.
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < self.sector_size() as usize {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buf too short"))
        } else {
            let mut data = self.get_mut(n)?;
            data.write(buf)
        }
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .field("partition", &self.partition)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn test_cache_read() {
        static mut TEST_DATA: [u8; 8192] = [0u8; 8192];

        // Set identifiable bytes to test the cache.
        unsafe {
            TEST_DATA[0] = 1;
            TEST_DATA[7680] = 255;
        }
        let mut cache = unsafe { CachedDevice::new(
            Cursor::new(&mut TEST_DATA[..]),
            Partition { start: 0, sector_size: 512 }) };

        let mut sector = [0u8; 512];
        assert_eq!(cache.read_sector(0, &mut sector).expect("Valid read"), 512);
        assert_eq!(sector[0], 1);
        assert_eq!(&sector[1..], &[0u8; 511][..]);

        assert_eq!(cache.read_sector(15, &mut sector).expect("Valid read"), 512);
        assert_eq!(sector[0], 255);
        assert_eq!(&sector[1..], &[0u8; 511][..]);

        // Now make a change to the underlying data and verify it isn't read,
        // the data should be cached.
        unsafe {
            TEST_DATA[1] = 127;
        }

        assert_eq!(cache.read_sector(0, &mut sector).expect("Valid read"), 512);
        assert_eq!(sector[0], 1);
        assert_eq!(&sector[1..], &[0u8; 511][..]);
    }

    #[test]
    fn test_cache_write() {
        static mut TEST_DATA: [u8; 8192] = [0u8; 8192];

        {
            let mut cache = unsafe { CachedDevice::new(
                Cursor::new(&mut TEST_DATA[..]),
                Partition { start: 0, sector_size: 512 }) };

            let mut sector = [0u8; 512];
            sector[0] = 255;
            assert_eq!(cache.write_sector(0, &sector).expect("Valid write"), 512);

            // Reading the data back from the cache should get the new value.
            let mut read_sector = [0u8; 512];
            assert_eq!(cache.read_sector(0, &mut read_sector).expect("Valid read"), 512);
            assert_eq!(read_sector[0], 255);
            assert_eq!(&read_sector[1..], &[0u8; 511][..]);
        }

        // The write should go to the cache, not the real device.
        assert_eq!(unsafe { TEST_DATA[0] }, 0);
    }

    #[test]
    fn test_partition() {
        static mut TEST_DATA: [u8; 8192] = [0u8; 8192];

        // Set identifiable bytes to test the cache.
        unsafe {
            TEST_DATA[2048] = 1;
            TEST_DATA[512] = 255;
        }
        let mut cache = unsafe { CachedDevice::new(
            Cursor::new(&mut TEST_DATA[..]),
            Partition { start: 2, sector_size: 1024 }) };

        // Before the partition.
        let mut physical_sector = [0u8; 512];
        assert_eq!(cache.read_sector(1, &mut physical_sector).expect("Valid read"), 512);
        assert_eq!(physical_sector[0], 255);
        assert_eq!(&physical_sector[1..], &[0u8; 511][..]);

        // After the partition.
        let mut sector = [0u8; 1024];
        assert_eq!(cache.read_sector(3, &mut sector).expect("Valid read"), 1024);
        assert_eq!(sector[0], 1);
        assert_eq!(&sector[1..], &[0u8; 1023][..]);
    }

    #[test]
    #[should_panic]
    fn test_partiton_error() {
        static mut TEST_DATA: [u8; 512] = [0u8; 512];

        let mut cache = unsafe { CachedDevice::new(
            Cursor::new(&mut TEST_DATA[..]),
            Partition { start: 2, sector_size: 256 }) };
    }

    #[test]
    fn test_bounds() {
        static mut TEST_DATA: [u8; 512] = [0u8; 512];

        let mut cache = unsafe { CachedDevice::new(
            Cursor::new(&mut TEST_DATA[..]),
            Partition { start: 0, sector_size: 512 }) };

        let mut sector = [0u8; 512];
        cache.read_sector(1, &mut sector).expect_err("Out of bounds");
    }
}
