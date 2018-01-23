// FIXME: Make me pass! Diff budget: 25 lines.

enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}

fn main() {
    assert_eq!(Seconds(120), Minutes(2));
    assert_eq!(Seconds(420), Minutes(7));
    assert_eq!(MilliSeconds(420000), Minutes(7));
    assert_eq!(MilliSeconds(43000), Seconds(43));
}
