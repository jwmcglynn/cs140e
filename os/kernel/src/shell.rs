use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std::str;
use std::io::Write;

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

    fn execute(&self) {
        match self.path() {
            "echo" => handle_echo(&self.args[1..]),
            "memstat" => handle_memstat(&self.args[1..]),
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

const BELL: u8 = 7;
const BACKSPACE: u8 = 8;
const DELETE: u8 = 127;

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
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
                        command.execute();
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
