// FIXME: Prevent this file from compiling! Diff budget: 1 line.
#[derive(Clone, Copy)]
struct MyType(usize);

// Note: do not modify this function.
fn main() {
    let x = MyType(10);
    let y = x;
    let z = x;
}
