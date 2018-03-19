use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std::str;
use std::io::Write;
use std::path::{Path, PathBuf};
use fat32::traits::{Dir, Entry, FileSystem, Timestamp, Metadata};

use FILE_SYSTEM;

#[cfg(not(test))]
use ALLOCATOR;
#[cfg(not(test))]
use allocator::AllocStats;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

    fn execute(&self, working_dir: &mut PathBuf) {
        match self.path() {
            "echo" => handle_echo(&self.args[1..]),
            "memstat" => handle_memstat(&self.args[1..]),
            "pwd" => handle_pwd(&self.args[1..], working_dir),
            "cd" => handle_cd(&self.args[1..], working_dir),
            "ls" => handle_ls(&self.args[1..], working_dir),
            "cat" => handle_cat(&self.args[1..], working_dir),
            path => kprintln!("Unknown command: {}", path)
        }
    }
}

fn handle_echo(args: &[&str]) {
    let len = args.len();

    if len > 0 {
        for s in args[..len - 1].iter() {
            kprint!("{} ", s);
        }

        kprintln!("{}", args[len - 1]);
    }
}

#[cfg(not(test))]
fn handle_memstat(args: &[&str]) {
    if args.len() > 0 {
        kprintln!("Too many args. Usage:");
        kprintln!("memstat");
        kprintln!();
        return;
    }

    ALLOCATOR.print_stats();
}

#[cfg(test)]
fn handle_memstat(_args: &[&str]) {
    // No-op for test.
}

fn handle_pwd(args: &[&str], working_dir: &PathBuf) {
    if args.len() > 0 {
        kprintln!("Too many args. Usage:");
        kprintln!("pwd");
        kprintln!();
        return;
    }

    kprintln!("{}", working_dir.as_path().display());
}

fn handle_cd(args: &[&str], working_dir: &mut PathBuf) {
    if args.len() != 1 {
        kprintln!("Usage:");
        kprintln!("cd <directory>");
        kprintln!();
        return;
    }

    if args[0] == "." {
        // No-op.
    } else if args[0] == ".." {
        working_dir.pop();
    } else {
        let path = Path::new(args[0]);

        let mut new_dir = working_dir.clone();
        new_dir.push(path);

        let entry = FILE_SYSTEM.open(new_dir.as_path());
        if entry.is_err() {
            kprintln!("Path not found.");
            return;
        }

        if entry.unwrap().as_dir().is_some() {
            working_dir.push(path);
        } else {
            kprintln!("Not a directory.");
        }
    }
}


fn print_entry<E: Entry>(entry: &E) {
    fn write_bool(b: bool, c: char) {
        if b { kprint!("{}", c); } else { kprint!("-"); }
    }

    fn write_timestamp<T: Timestamp>(ts: T) {
        kprint!("{:02}/{:02}/{} {:02}:{:02}:{:02} ",
               ts.month(), ts.day(), ts.year(), ts.hour(), ts.minute(), ts.second());
    }

    write_bool(entry.is_dir(), 'd');
    write_bool(entry.is_file(), 'f');
    write_bool(entry.metadata().read_only(), 'r');
    write_bool(entry.metadata().hidden(), 'h');
    kprint!("\t");

    write_timestamp(entry.metadata().created());
    write_timestamp(entry.metadata().modified());
    write_timestamp(entry.metadata().accessed());
    kprint!("\t");

    kprintln!("{}", entry.name());
}

fn handle_ls(mut args: &[&str], working_dir: &PathBuf) {
    let show_hidden = args.len() > 0 && args[0] == "-a";
    if show_hidden {
        args = &args[1..];
    }

    if args.len() > 1 {
        kprintln!("Usage:");
        kprintln!("ls [-a] [directory]");
        kprintln!();
        return;
    }

    let mut dir = working_dir.clone();
    if !args.is_empty() {
        if args[0] == "." {
            // No-op.
        } else if args[0] == ".." {
            dir.pop();
        } else {
            dir.push(args[0]);
        }
    }

    let entry_result = FILE_SYSTEM.open(dir.as_path());
    if entry_result.is_err() {
        kprintln!("Path not found.");
        return;
    }

    let entry = entry_result.unwrap();

    if let Some(dir_entry) = entry.into_dir() {
        let mut entries = dir_entry.entries().expect("List dir");
        for item in entries {
            if show_hidden || !item.metadata().hidden() {
                print_entry(&item);
            }
        }
    } else {
        kprintln!("Not a directory.");
    }
}

fn handle_cat(args: &[&str], working_dir: &PathBuf) {
    if args.len() != 1 {
        kprintln!("Usage:");
        kprintln!("cat <file>");
        kprintln!();
        return;
    }

    let mut dir = working_dir.clone();
    dir.push(args[0]);

    let entry_result = FILE_SYSTEM.open(dir.as_path());
    if entry_result.is_err() {
        kprintln!("Path not found.");
        return;
    }

    let entry = entry_result.unwrap();
    if let Some(ref mut file) = entry.into_file() {
        loop {
            use std::io::Read;

            let mut buffer = [0u8; 512];
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => kprint!("{}", String::from_utf8_lossy(&buffer)),
                Err(e) => kprint!("Failed to read file: {:?}", e)
            }
        }

        kprintln!("");
    } else {
        kprintln!("Not a file.");
    }
}

const BELL: u8 = 7;
const BACKSPACE: u8 = 8;
const DELETE: u8 = 127;

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    let mut working_dir = PathBuf::from("/");

    loop {
        let mut buf_storage = [0u8; 512];
        let mut buf = StackVec::new(&mut buf_storage);

        kprint!("{}", prefix);

        loop {
            let byte = CONSOLE.lock().read_byte();

            if byte == b'\r' || byte == b'\n' {
                let mut command_storage: [&str; 64] = [""; 64];
                let result = Command::parse(
                    str::from_utf8(buf.into_slice()).unwrap(),
                    &mut command_storage);

                kprint!("\n");

                match result {
                    Err(Error::TooManyArgs) => {
                        kprintln!("error: too many arguments");
                    },
                    Err(Error::Empty) => {
                        // No command, ignore.
                    }
                    Ok(command) => {
                        command.execute(&mut working_dir);
                    },
                }

                break
            } else {
                let mut console = CONSOLE.lock();

                if byte == BACKSPACE || byte == DELETE {
                    // Handle backspace and delete to erase a single character.
                    if buf.pop() == None {
                        console.write_byte(BELL);
                    } else {
                        console.write(&[BACKSPACE, b' ', BACKSPACE]).expect("write");
                    }
                } else if byte < 32 || byte == 255 {
                    // Discard non-printable characters and send an alert.
                    console.write_byte(BELL);
                } else {
                    if buf.push(byte).is_err() {
                        console.write_byte(BELL);
                    } else {
                        console.write_byte(byte);
                    }
                }
            }
        }
    }
}
