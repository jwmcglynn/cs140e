// FIXME: Make me pass! Diff budget: 25 lines.

#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}

use Duration::*;

impl PartialEq for Duration {
    fn eq(&self, other: &Duration) -> bool {
        let normalize = |duration: &Duration| {
            match self {
                &MilliSeconds(milliseconds) => milliseconds,
                &Seconds(seconds) => (seconds as u64) * 1000,
                &Minutes(minutes) => (minutes as u64) * 60 * 1000,
            }
        };

        normalize(self) == normalize(other)
    }
}

fn main() {
    assert_eq!(Seconds(120), Minutes(2));
    assert_eq!(Seconds(420), Minutes(7));
    assert_eq!(MilliSeconds(420000), Minutes(7));
    assert_eq!(MilliSeconds(43000), Seconds(43));
}
