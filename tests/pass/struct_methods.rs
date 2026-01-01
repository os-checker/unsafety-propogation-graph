#![allow(dead_code)]

struct S {
    s: String,
}

impl S {
    fn s_ref(&self) {}

    fn s_mut_ref(&mut self) {}

    fn field_ref(&self) {
        _ = &self.s;
    }

    fn field_mut_ref(&mut self) {
        _ = &mut self.s;
    }

    fn write_field(&mut self) {
        self.s = String::new();
    }
}
