use std::fmt;
use std::ops::{Add, Deref};

/// `Measured::Measure` must form `Monoid`
pub trait Measured {
    type Measure: Add<Output = Self::Measure> + Clone;
    fn measure_zero() -> Self::Measure;
    fn measure(&self) -> Self::Measure;
}

#[derive(PartialEq, Eq)]
pub struct Sized<T>(pub T);

impl<T> fmt::Debug for Sized<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Measured for Sized<T> {
    type Measure = usize;
    fn measure_zero() -> Self::Measure {
        0
    }
    fn measure(&self) -> Self::Measure {
        1
    }
}

impl<T> Deref for Sized<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
