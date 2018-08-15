//! `Monoid` trait and implementations
use std::ops::{Add, Deref};

/// Monoid definition
///
/// Monoid is a tuple of `(S, O, I)` where:
///   - `S` - set of elements
///   - `O` - binary operation on S `S x S -> S`, here called `plus`
///   - `I` - identity element of this monoid, here called `zero`
///
/// Every monoid implementation should satisfy following laws:
///  - **associativity**: `a + (b + c) == (a + b) + c`
///  -  **identity element**: `zero + a == a + zero == a`
pub trait Monoid {
    /// `zero` or `identity` elemnt of monoid
    fn zero() -> Self;
    /// `plus` operation of `Monoid`
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

/// Monoid formed by `Add::add` operation and `Default::default()` identity element
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sum<T> {
    /// New type wrapped value
    pub value: T,
}

impl<T> Sum<T> {
    /// Create `Sum` monoid from any value
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
