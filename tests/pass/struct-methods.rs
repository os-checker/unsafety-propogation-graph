#![allow(dead_code)]

/// First line.
/// Second line.
///
/// Forth line.
struct S {
    /// Field comments.
    s: String,
}

impl S {
    fn new(s: String) -> S {
        S { s }
    }

    fn s_ref(&self) {}

    fn s_mut_ref(&mut self) {}

    fn field_ref(&self) {
        _ = &self.s;
    }

    fn field_mut_ref(&mut self) {
        _ = &mut self.s;
    }

    /// Update the field.
    fn write_field(&mut self) {
        self.s = String::new();
    }
}
