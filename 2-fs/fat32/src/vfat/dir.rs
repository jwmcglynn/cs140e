use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;

use traits;
use util::{VecExt, Unused};
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    start: Cluster,
    vfat: Shared<VFat>,

    name: String,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    filename: [u8; 8],
    extension: [u8; 3],
    attributes: Attributes,
    reserved: Unused<u8>,
    creation_time_subsecond: Unused<u8>,
    created: Timestamp,
    accessed: Date,
    cluster_high: u16,
    modified: Timestamp,
    cluster_low: u16,
    file_size: u32
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    sequence_number: u8,
    name_1: [u16; 5],
    attributes: u8,
    unused_1: Unused<u8>,
    checksum: u8,
    name_2: [u16; 6],
    unused_2: Unused<u16>,
    name_3: [u16; 2],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    entry_info: u8,
    unknown: Unused<[u8; 10]>,
    attributes: u8,
    unknown_2: Unused<[u8; 20]>,
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

pub struct DirIterator {
    data: Vec<VFatDirEntry>,
    offset: usize,
    vfat: Shared<VFat>,
}

impl VFatRegularDirEntry {
    pub fn filename(&self) -> String {
        let name = VFatRegularDirEntry::fat_string(&self.filename);

        if !self.is_dir() {
            let extension = VFatRegularDirEntry::fat_string(&self.extension);

            if !extension.is_empty() {
                let mut full_name = name.into_owned();
                full_name.push('.');
                full_name.push_str(&extension);
                return full_name;
            }
        }

        name.into_owned()
    }

    pub fn fat_string<'a>(buf: &'a [u8]) -> Cow<'a, str> {
        let mut end = 0;
        for i in 0..buf.len() {
            if buf[i] == 0x00 || buf[i] == 0x00 {
                break
            }

            end += 1;
        }

        String::from_utf8_lossy(&buf[..end])
    }

    pub fn is_dir(&self) -> bool {
        self.attributes.directory()
    }

    pub fn cluster(&self) -> Cluster {
        return Cluster::from((self.cluster_high as u32) << 16
                                | self.cluster_low as u32)
    }
}

impl VFatLfnDirEntry {
    pub fn sequence_number(&self) -> usize {
        let result = self.sequence_number & 0b11111;
        assert!(result != 0);
        result as usize
    }

    pub fn last_entry(&self) -> bool {
        self.sequence_number & 0b01000000 != 0
    }

    pub fn append_name(&self, buf: &mut Vec<u16>) {
        let subsets = [&self.name_1[..], &self.name_2[..], &self.name_3[..]];
        for subset in subsets.iter() {
            for c in subset.iter() {
                if *c == 0x0000 || *c == 0x00FF {
                    return;
                }

                buf.push(*c);
            }
        }
    }
}

impl VFatUnknownDirEntry {
    const ENTRY_END: u8 = 0x00;
    const ENTRY_UNUSED: u8 = 0xE5;

    const LFN_FLAG: u8 = 0x0F;

    pub fn is_end(&self) -> bool {
        self.entry_info == VFatUnknownDirEntry::ENTRY_END
    }

    pub fn is_unused(&self) -> bool {
        self.entry_info == VFatUnknownDirEntry::ENTRY_UNUSED
    }

    pub fn is_lfn(&self) -> bool {
        self.attributes == VFatUnknownDirEntry::LFN_FLAG
    }
}

impl Dir {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        use traits::Dir;

        let name_str = name.as_ref().to_str().ok_or(
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8"))?;

        self.entries()?.find(|item| {
            match item {
                &Entry::File(ref file) => file.name().eq_ignore_ascii_case(
                    name_str),
                &Entry::Dir(ref dir) => dir.name().eq_ignore_ascii_case(
                    name_str),
            }
        }).ok_or(io::Error::new(io::ErrorKind::NotFound, "Not found"))
    }

    /// Returns the directory name, or an empty string if this is the root
    /// directory.
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl DirIterator {
    fn lfn_to_string(lfn: &mut Vec<&VFatLfnDirEntry>) -> String {
        lfn.sort_by_key(|a| a.sequence_number());

        let mut name_data: Vec<u16> = Vec::new();
        for entry in lfn.iter() {
            entry.append_name(&mut name_data);
        }

        String::from_utf16_lossy(name_data.as_slice())
    }

    pub fn create_entry(&self, lfn: &mut Vec<&VFatLfnDirEntry>,
                        entry: VFatRegularDirEntry)
        -> Entry
    {
        let name = if lfn.is_empty() {
            entry.filename()
        } else {
            DirIterator::lfn_to_string(lfn)
        };

        if entry.is_dir() {
            Entry::Dir(Dir {
                start: entry.cluster(),
                vfat: self.vfat.clone(),
                name
            })
        } else {
            let metadata = Metadata::new(
                entry.attributes, entry.created,
                Timestamp { date: entry.accessed, time: Time::default() },
                entry.modified);

            Entry::File(File::new(entry.cluster(), self.vfat.clone(), name,
                                  metadata, entry.file_size))
        }
    }
}

impl Iterator for DirIterator {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut lfn: Vec<&VFatLfnDirEntry> = Vec::new();
        let mut has_last_entry = false;

        for offset in self.offset..self.data.len() {
            let entry = &self.data[offset];

            let entry_unknown = unsafe { entry.unknown };
            if entry_unknown.is_end() {
                break;
            }

            if entry_unknown.is_unused() {
                continue;
            }

            if entry_unknown.is_lfn() {
                let long_filename = unsafe { &entry.long_filename };
                assert!(!has_last_entry);

                has_last_entry = long_filename.last_entry();
                lfn.push(long_filename);
            } else {
                return Some(self.create_entry(&mut lfn,
                                              unsafe { entry.regular }));
            }
        }

        None
    }
}

impl traits::Dir for Dir {
    type Entry = Entry;
    type Iter = DirIterator;

    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut data = Vec::new();
        let bytes_read = self.vfat.borrow_mut().read_chain(self.start,
                                                           &mut data)?;
        assert_eq!(bytes_read, data.len());

        Ok(DirIterator { data: unsafe { data.cast() }, offset: 0,
                         vfat: self.vfat.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn validate_sizes() {
        assert_eq!(mem::size_of::<VFatDirEntry>(), 32);
        assert_eq!(mem::size_of::<VFatRegularDirEntry>(), 32);
        assert_eq!(mem::size_of::<VFatLfnDirEntry>(), 32);
        assert_eq!(mem::size_of::<VFatUnknownDirEntry>(), 32);
    }
}
