pub mod sd;

use std::io;
use std::path::Path;

use fat32::vfat::{self, Shared, VFat};
pub use fat32::traits;

use mutex::Mutex;
use self::sd::Sd;

pub struct FileSystem(Mutex<Option<Shared<VFat>>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub fn initialize(&self) {
        unimplemented!("FileSystem::initialize()")
    }
}

// FIXME: Implement `fat32::traits::FileSystem` for a useful type.
