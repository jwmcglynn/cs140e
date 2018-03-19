use std::{i32, io};
use fat32::traits::BlockDevice;
use pi::timer::spin_sleep_us;

extern "C" {
    /// A global representing the last SD controller error that occurred.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occurred, or -2 if an error sending commands to
    /// the SD controller occurred.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occurred or -2 if an
    /// error sending commands to the SD controller occurred. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

#[no_mangle]
pub fn wait_micros(us: u32) {
    // If we wait for the us value, the SD driver times out.
    // Multiply by 100 to work around that issue and avoid the timeouts.
    spin_sleep_us(us as u64 * 100);
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    CommandError,
    Unknown
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    pub fn new() -> Result<Sd, Error> {
        let result = unsafe { sd_init() };
        if result == 0 {
            Ok(Sd { })
        } else {
            Err(Sd::map_error(result as i64))
        }
    }

    fn map_error(code: i64) -> Error {
        if code == -1 {
            Error::Timeout
        } else if code == -2 {
            Error::CommandError
        } else if code <= -3 {
            Error::Unknown
        } else {
            panic!("Unexpected SD error");
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 512 {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "buf too small"))
        } else if n > i32::MAX as u64 {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "n out of range"))
        } else {
            let bytes = unsafe { sd_readsector(n as i32, buf.as_mut_ptr()) };

            if bytes == 0 {
                let error = Sd::map_error(unsafe { sd_err });
                match error {
                    Error::Timeout => Err(io::Error::new(
                        io::ErrorKind::TimedOut, "Read timeout")),
                    _ => Err(io::Error::new(io::ErrorKind::Other,
                                            "Driver error")),
                }
            } else {
                Ok(bytes as usize)
            }
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
