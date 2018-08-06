use std::fmt;
use std::ops::Deref;

use monoid::{Monoid, Sum};

pub trait Measured: Clone {
    type Measure: Monoid + Clone;

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
        Sum::new(1)
    }
}

impl<T> Deref for Size<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
