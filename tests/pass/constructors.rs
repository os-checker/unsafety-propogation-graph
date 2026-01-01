#![allow(dead_code)]

struct S {
    s: String,
}

fn a() -> S {
    S { s: String::new() }
}

impl S {
    fn new() -> S {
        a()
    }
}
