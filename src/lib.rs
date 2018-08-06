// NOTES:
//  - this implementation does not use non-regular recursive types, as I was not able
//    to make it types check. Structure itself compiles but any implementaoin fails.
//
// TODO:
//  - improve lifting values to nodes in `.concat`
//  - docs, also value must be cheaply copyable
//  - use more references in function signatures and call clone in the body of functions
//  - lazy spine?

#[cfg(test)]
extern crate quickcheck;

mod digit;
pub mod measure;
pub mod monoid;
mod node;
mod reference;
mod tree;

#[cfg(test)]
mod test;

pub use measure::Measured;
pub use monoid::Monoid;
pub use reference::{ArcRefs, RcRefs, Ref, Refs};
pub use tree::FingerTree;

pub mod rc {
    pub type FingerTree<V> = super::FingerTree<super::RcRefs, V>;
}

pub mod sync {
    pub type FingerTree<V> = super::FingerTree<super::ArcRefs, V>;
}
