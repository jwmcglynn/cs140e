use traits;
use vfat::{File, Dir, Metadata};

#[derive(Debug)]
enum EntryData {
    File(File),
    Dir(Dir)
}

#[derive(Debug)]
pub struct Entry {
    item: EntryData,
    name: String,
    metadata: Metadata,
}

impl Entry {
    pub fn new_file(name: String, metadata: Metadata, file: File) -> Entry {
        Entry { item: EntryData::File(file), name, metadata }
    }

    pub fn new_dir(name: String, metadata: Metadata, dir: Dir) -> Entry {
        Entry { item: EntryData::Dir(dir), name, metadata }
    }
}

impl traits::Entry for Entry {
    type File = File;
    type Dir = Dir;
    type Metadata = Metadata;

    /// The name of the file or directory corresponding to this entry.
    fn name(&self) -> &str {
        self.name.as_str()
    }

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata {
        &self.metadata
    }

    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&Self::File> {
        match &self.item {
            &EntryData::File(ref file) => Some(file),
            &EntryData::Dir(_) => None,
        }
    }

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Self::Dir> {
        match &self.item {
            &EntryData::File(_) => None,
            &EntryData::Dir(ref dir) => Some(dir),
        }
    }

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<Self::File> {
        match self.item {
            EntryData::File(file) => Some(file),
            EntryData::Dir(_) => None,
        }
    }

    /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Self::Dir> {
        match self.item {
            EntryData::File(_) => None,
            EntryData::Dir(dir) => Some(dir),
        }
    }
}