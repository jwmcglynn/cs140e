use std::{io, fmt, mem};

use traits::BlockDevice;
use vfat::Error;
use util::Unused;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    // Base bios parameter block.
    jmp_block: Unused<[u8; 3]>,
    oem_identifier: Unused<[u8; 8]>,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    max_dir_entries: Unused<u16>,
    logical_sectors_16: u16,
    media_descriptor_type: u8,
    sectors_per_fat_16: Unused<u16>,
    sectors_per_track: Unused<u16>,
    heads_or_sides: Unused<u16>,
    hidden_sectors: Unused<u32>,
    logical_sectors_32: u32,

    // Extended bios parameter block.
    sectors_per_fat_32: u32,
    flags: u16,
    fat_version_number: [u8; 2],
    root_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    reserved: Unused<[u8; 12]>,
    drive_number: u8,
    reserved_2: Unused<u8>,
    signature: u8,
    volumeid_serial: u32,
    volume_label: [u8; 11],
    system_identifier: [u8; 8],
    boot_code: Unused<[u8; 420]>,
    bootable_partition_signature: u16,
}

const EBPB_SIZE: usize = 512;
const VALID_BOOTABLE_SIGNATURE: u16 = 0xAA55;

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(
        mut device: T,
        sector: u64
    ) -> Result<BiosParameterBlock, Error> {
        let mut sector_data = [0u8; EBPB_SIZE];
        let bytes = device.read_sector(sector, &mut sector_data)?;

        if bytes != EBPB_SIZE {
            return Err(Error::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "EBPB too short")))
        }

        let ebpb: BiosParameterBlock = unsafe { mem::transmute(sector_data) };
        if ebpb.bootable_partition_signature != VALID_BOOTABLE_SIGNATURE {
            return Err(Error::BadSignature);
        }

        Ok(ebpb)
    }

    /// The number of logical sectors for the partition.
    pub fn logical_sectors(&self) -> u32 {
        if self.logical_sectors_16 != 0 {
            self.logical_sectors_16 as u32
        } else {
            self.logical_sectors_32
        }
    }

    /// Number of bytes per logical sector.
    pub fn bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Sectors per FAT.
    pub fn sectors_per_fat(&self) -> u32 {
        self.sectors_per_fat_32
    }

    /// Sectors per cluster.
    pub fn sectors_per_cluster(&self) -> u8 {
        self.sectors_per_cluster
    }

    /// The sector offset, from the start of the partition, to the first fat
    /// sector.
    pub fn fat_start_sector(&self) -> u64 {
        self.reserved_sectors as u64
    }

    /// The sector offset, from the start of the partition, to the first data
    /// sector.
    pub fn data_start_sector(&self) -> u64 {
        self.fat_start_sector()
            + self.sectors_per_fat_32 as u64 * self.fat_count as u64
    }

    /// Root dir cluster.
    pub fn root_cluster(&self) -> u32 {
        self.root_cluster
    }

}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("reserved_sectors", &self.reserved_sectors)
            .field("fat_count", &self.fat_count)
            .field("logical_sectors", &self.logical_sectors())
            .field("media_descriptor_type", &self.media_descriptor_type)
            .field("sectors_per_fat", &self.sectors_per_fat_32)
            .field("flags", &self.flags)
            .field("fat_version_number", &self.fat_version_number)
            .field("root_cluster", &self.root_cluster)
            .field("fsinfo_sector", &self.fsinfo_sector)
            .field("backup_boot_sector", &self.backup_boot_sector)
            .field("drive_number", &self.drive_number)
            .field("signature", &self.signature)
            .field("volumeid_serial", &self.volumeid_serial)
            .field("volume_label", &self.volume_label)
            .field("system_identifier", &self.system_identifier)
            .field("bootable_partition_signature", &self.bootable_partition_signature)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn test_block_size_small() {
        let mut data = [0u8; 511];
        let result = BiosParameterBlock::from(
            Cursor::new(&mut data[..]), 0);
        match result.expect_err("EOF") {
            Error::Io(io) => assert_eq!(io.kind(), io::ErrorKind::UnexpectedEof),
            _ => assert!(false, "Invalid error"),
        }
    }

    #[test]
    fn test_simple_block() {
        let mut data = [0u8; 512];
        data[510] = 0x55;
        data[511] = 0xAA;
        BiosParameterBlock::from(
            Cursor::new(&mut data[..]), 0).expect("Valid block");
    }

    #[test]
    fn test_invalid_signature() {
        let mut data = [0u8; 512];
        let result = BiosParameterBlock::from(
            Cursor::new(&mut data[..]), 0);
        match result.expect_err("Signature") {
            Error::BadSignature => (),
            _ => assert!(false, "Invalid error"),
        }

        data[510] = 0x55;
        data[511] = 0xAB;
        let result = BiosParameterBlock::from(
            Cursor::new(&mut data[..]), 0);
        match result.expect_err("Signature") {
            Error::BadSignature => (),
            _ => assert!(false, "Invalid error"),
        }
    }
}
