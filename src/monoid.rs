//! [`Monoid`](monoid/trait.Monoid.html) trait and implementations
use std::ops::{Add, Deref};

/// Monoid definition
///
/// Monoid is a tuple of `(S, O, I)` where:
///   - `S` - set of elements
///   - `O` - binary operation on S `S x S -> S`, here called `join`
///   - `I` - identity element of this monoid, here called `unit`
///
/// Every monoid implementation should satisfy following laws:
///  - **associativity**: `a + (b + c) == (a + b) + c`
///  -  **identity element**: `unit + a == a + unit == a`
pub trait Monoid {
    /// `unit` or `identity` element of monoid
    fn unit() -> Self;
    /// `join` operation of `Monoid`
    fn join(&self, other: &Self) -> Self;
}

// impl<A, B> Monoid for (A, B)
// where
//     A: Monoid,
//     B: Monoid,
// {
//     fn unit() -> Self {
//         (A::unit(), B::unit())
//     }

//     fn join(&self, other: &Self) -> Self {
//         (self.0.join(&other.0), self.1.join(&other.1))
//     }
// }

/// Monoid formed by `Add::add` operation and `Default::default()` identity element
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Sum<T>(pub T);

impl<T> Monoid for Sum<T>
where
    for<'a> &'a T: Add<Output = T>,
    T: Default,
{
    fn unit() -> Self {
        Sum(T::default())
    }

    fn join(&self, other: &Self) -> Self {
        Sum(&self.0 + &other.0)
    }
}

impl<T> Deref for Sum<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
