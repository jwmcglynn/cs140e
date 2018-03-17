use std::fmt;

use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

impl Date {
    pub fn year(&self) -> usize {
        (self.0 >> 9) as usize + 1980
    }

    pub fn month(&self) -> u8 {
        ((self.0 >> 5) as u8) & 0b1111
    }

    pub fn day(&self) -> u8 {
        (self.0 & 0b11111) as u8
    }
}

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

impl Time {
    pub fn hour(&self) -> u8 {
        (self.0 >> 11) as u8
    }

    pub fn minute(&self) -> u8 {
        ((self.0 >> 5) as u8) & 0b111111
    }

    pub fn second(&self) -> u8 {
        (self.0 as u8) & 0b11111 * 2
    }
}

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    attributes: Attributes,
    created: Timestamp,
    accessed: Timestamp,
    modified: Timestamp,
}

impl Attributes {
    const READ_ONLY: u8 = 0x01;
    const HIDDEN: u8 = 0x02;
    const SYSTEM: u8 = 0x04;
    const VOLUME_ID: u8 = 0x08;
    const DIRECTORY: u8 = 0x10;
    const ARCHIVE: u8 = 0x20;

    pub fn read_only(&self) -> bool {
        (self.0 & Attributes::READ_ONLY) != 0
    }

    pub fn hidden(&self) -> bool {
        (self.0 & Attributes::HIDDEN) != 0
    }

    pub fn system(&self) -> bool {
        (self.0 & Attributes::SYSTEM) != 0
    }

    pub fn volume_id(&self) -> bool {
        (self.0 & Attributes::VOLUME_ID) != 0
    }

    pub fn directory(&self) -> bool {
        (self.0 & Attributes::DIRECTORY) != 0
    }

    pub fn archive(&self) -> bool {
        (self.0 & Attributes::ARCHIVE) != 0
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut had_attribute = false;
        let attributes = [self.read_only(), self.hidden(), self.system(),
                          self.volume_id(), self.directory(), self.archive()];
        let attribute_names = ["READ_ONLY", "HIDDEN", "SYSTEM", "VOLUME_ID",
                               "DIRECTORY", "ARCHIVE"];

        assert_eq!(attributes.len(), attribute_names.len());

        for i in 0..attributes.len() {
            if attributes[i] {
                if had_attribute {
                    write!(f, "{}", "|")?;
                }
                write!(f, "{}", attribute_names[i].to_string())?;
                had_attribute = true;
            }
        }

        Ok(())
    }
}

impl traits::Timestamp for Timestamp {
    /// The calendar year.
    ///
    /// The year is not offset. 2009 is 2009.
    fn year(&self) -> usize {
        self.date.year()
    }

    /// The calendar month, starting at 1 for January. Always in range [1, 12].
    ///
    /// January is 1, Feburary is 2, ..., December is 12.
    fn month(&self) -> u8 {
        self.date.month()
    }

    /// The calendar day, starting at 1. Always in range [1, 31].
    fn day(&self) -> u8 {
        self.date.day()
    }

    /// The 24-hour hour. Always in range [0, 24).
    fn hour(&self) -> u8 {
        self.time.hour()
    }

    /// The minute. Always in range [0, 60).
    fn minute(&self) -> u8 {
        self.time.minute()
    }

    /// The second. Always in range [0, 60).
    fn second(&self) -> u8 {
        self.time.second()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use traits::Timestamp;
        write!(f, "{}-{}-{} {}:{:02}:{:02}", self.year(), self.month(),
               self.day(), self.hour(), self.minute(), self.second())
    }
}

impl Metadata {
    pub fn new(attributes: Attributes, created: Timestamp, accessed: Timestamp,
               modified: Timestamp) -> Metadata {
        Metadata { attributes, created, accessed, modified }
    }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    /// Whether the associated entry is read only.
    fn read_only(&self) -> bool {
        self.attributes.read_only()
    }

    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool {
        self.attributes.hidden()
    }

    /// The timestamp when the entry was created.
    fn created(&self) -> Self::Timestamp {
        self.created
    }

    /// The timestamp for the entry's last access.
    fn accessed(&self) -> Self::Timestamp {
        self.accessed
    }

    /// The timestamp for the entry's last modification.
    fn modified(&self) -> Self::Timestamp {
        self.modified
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "attributes={} created={} accessed={} modified={}",
               self.attributes, self.created, self.accessed, self.modified)
    }
}
