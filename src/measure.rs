//! [`Measured`](measure/trait.Measured.html) trait and implementations
use std::fmt;
use std::ops::Deref;

use crate::monoid::{Monoid, Sum};

/// Measured definition
///
/// Type implementing `Measured` are basically stating that they have assocated
/// monoidal measure with them. And both type itself and measure must be
/// **cheaply clonable**, otherwise you can just wrap them in `Arc|Rc`
pub trait Measured: Clone {
    /// Measure type
    type Measure: Monoid + Clone;

    /// Assocated measure with given value
    fn measure(&self) -> Self::Measure;
}

// impl<T> Measured for T
// where
//     T: Deref,
//     T::Target: Measured,
// {
//     type Measure = <T::Target as Measured>::Measure;

//     fn measure(&self) -> Self::Measure {
//         (*self).measure()
//     }
// }

///
#[derive(Clone, PartialEq, Eq)]
pub struct Size<T>(pub T);

impl<T> fmt::Debug for Size<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Measured for Size<T>
where
    T: Clone,
{
    type Measure = Sum<usize>;

    fn measure(&self) -> Self::Measure {
        Sum(1)
    }
}

impl<T> Deref for Size<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
