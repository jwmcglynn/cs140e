// FIXME: Make me compile! Diff budget: 3 lines.

// Do not modify the inner type &'a T.
struct RefWrapper<T>(&'a T);

// Do not modify the inner type &'b RefWrapper<'a, T>.
struct RefWrapperWrapper<T>(&'b RefWrapper<'a, T>);

impl RefWrapperWrapper {
    fn inner(&self) -> &'a T {
        (self.0).0
    }
}

pub fn main() { }
