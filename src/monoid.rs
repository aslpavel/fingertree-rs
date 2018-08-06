use std::ops::{Add, Deref};

/// Every monoid implementation should satisfy following laws
///   **associativity**: `a + (b + c) == (a + b) + c`
///   **identity element**: `zero + a == a + zero == a`
pub trait Monoid {
    fn zero() -> Self;
    fn plus(&self, other: &Self) -> Self;
}

// impl<A, B> Monoid for (A, B)
// where
//     A: Monoid,
//     B: Monoid,
// {
//     fn zero() -> Self {
//         (A::zero(), B::zero())
//     }

//     fn plus(&self, other: &Self) -> Self {
//         (self.0.plus(&other.0), self.1.plus(&other.1))
//     }
// }

/// Newtype wrapper that implements `Monoid` for any type that implements
/// `Add` and `Default` value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sum<T> {
    pub value: T,
}

impl<T> Sum<T> {
    pub fn new(value: T) -> Self {
        Sum { value }
    }
}

impl<T> Monoid for Sum<T>
where
    for<'a> &'a T: Add<Output = T>,
    T: Default,
{
    fn zero() -> Self {
        Sum::new(T::default())
    }

    fn plus(&self, other: &Self) -> Self {
        Sum::new(&self.value + &other.value)
    }
}

impl<T> Deref for Sum<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
