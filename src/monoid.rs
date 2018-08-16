//! [`Monoid`](monoid/trait.Monoid.html) trait and implementations
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
    /// `zero` or `identity` element of monoid
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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Sum<T>(pub T);

impl<T> Monoid for Sum<T>
where
    for<'a> &'a T: Add<Output = T>,
    T: Default,
{
    fn zero() -> Self {
        Sum(T::default())
    }

    fn plus(&self, other: &Self) -> Self {
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
