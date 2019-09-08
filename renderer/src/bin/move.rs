struct A;

struct B {
    a: A,
    b_i: usize,
}

struct C {
    b: B,
    c_i: usize,
}

impl std::ops::Deref for C {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.b
    }
}

impl std::ops::DerefMut for C {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.b
    }
}

fn main() {
    let mut a = A;

    for b_i in 0..2 {
        {
            let b = do_b(B { a, b_i });
            a = b.a;
        }
    }
}

fn do_b(mut b: B) -> B {
    for c_i in 0..2 {
        {
            let c = do_c(C { b, c_i });
            b = c.b;
        }
    }

    b
}

fn do_c(c: C) -> C {
    dbg!(c.b_i, c.c_i);
    c
}
