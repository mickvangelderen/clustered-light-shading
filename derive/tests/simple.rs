use derive::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext, EnumPrev)]
pub enum ABC {
    A,
    B,
    C,
    D,
}

#[test]
fn next_works() {
    assert_eq!(Some(ABC::B), ABC::A.next());
    assert_eq!(Some(ABC::C), ABC::B.next());
    assert_eq!(Some(ABC::D), ABC::C.next());
    assert_eq!(None, ABC::D.next());
}

#[test]
fn wrapping_next_works() {
    assert_eq!(ABC::B, ABC::A.wrapping_next());
    assert_eq!(ABC::C, ABC::B.wrapping_next());
    assert_eq!(ABC::D, ABC::C.wrapping_next());
    assert_eq!(ABC::A, ABC::D.wrapping_next());
}

#[test]
fn wrapping_next_assign_works() {
    let mut x = ABC::A;
    x.wrapping_next_assign();
    assert_eq!(ABC::B, x);
    x.wrapping_next_assign();
    assert_eq!(ABC::C, x);
    x.wrapping_next_assign();
    assert_eq!(ABC::D, x);
    x.wrapping_next_assign();
    assert_eq!(ABC::A, x);
}

// #[test]
// fn prev_works() {
//     assert_eq!(None, ABC::A.prev());
//     assert_eq!(Some(ABC::A), ABC::B.prev());
//     assert_eq!(Some(ABC::B), ABC::C.prev());
//     assert_eq!(Some(ABC::C), ABC::D.prev());
// }

// #[test]
// fn wrapping_prev_works() {
//     assert_eq!(ABC::D, ABC::A.wrapping_prev());
//     assert_eq!(ABC::A, ABC::B.wrapping_prev());
//     assert_eq!(ABC::B, ABC::C.wrapping_prev());
//     assert_eq!(ABC::C, ABC::D.wrapping_prev());
// }
