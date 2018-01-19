// FIXME: Make me pass! Diff budget: 30 lines.

struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    fn default() -> Builder {
        Builder { string: None, number: None }
    }

    fn string<S: Into<String>>(&mut self, value: S) -> &mut Builder {
        self.string = Some(value.into());
        self
    }

    fn number(&mut self, value: usize) -> &mut Builder {
        self.number = Some(value);
        self
    }

    fn to_string(&self) -> String {
        // This could also be written as two if let/match statements, but this is less lines :-)
        match (&self.string, self.number) {
            (&Some(ref string), Some(number)) => string.clone() + " " + &number.to_string(),
            (&Some(ref string), None) => string.clone(),
            (&None, Some(number)) => number.to_string(),
            (&None, None) => "".to_string(),
        }
    }
}

// Do not modify this function.
fn main() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default()
        .string("heap!".to_owned())
        .to_string();

    assert_eq!(c, "heap!");
}
