impl S {
    fn mutate_a(&mut self) {
        self.a = String::new();
    }

    fn mutate(&mut self) {
        self.a = String::new();
        self.b.push(' ');
    }
}

impl E {
    fn mutate1(&mut self) {
        if let E::A(a) = self {
            a.push(' ');
        }
    }

    fn mutate2(&mut self) {
        match self {
            E::A(a) => *a = String::new(),
            E::B(b) => b.push(' '),
        };
    }

    fn mutate_plain(&mut self) {
        *self = match self {
            E::A(_) => E::A(String::new()),
            E::B(_) => E::B(String::new()),
        };
    }
}

/// Struct S doc.
struct S {
    /// Field a doc.
    a: String,
    /// Field b doc.
    b: String,
}

/// Enum doc.
enum E {
    /// Varaint A doc.
    A(String),
    /// Varaint B doc.
    B(String),
}
