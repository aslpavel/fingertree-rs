//! Finger Trees
//! [![Build Status](https://travis-ci.org/aslpavel/fingertree-rs.svg?branch=master)](https://travis-ci.org/aslpavel/fingertree-rs)
//! [![Coverage Status](https://coveralls.io/repos/github/aslpavel/fingertree-rs/badge.svg?branch=master)](https://coveralls.io/github/aslpavel/fingertree-rs?branch=master)
//!
//! Finger trees is a functional representation of persistent sequences
//! supporting access to the ends in amortized constant time, and concatenation
//! and splitting in time logarithmic in the size of the smaller piece. It also
//! has [`split`](struct.FingerTree.html#method.split) operation defined in general
//! form, which can be used to implement sequence, priority queue, search tree,
//! priority search queue and more datastructures.
//!
//! ## Links:
//!  - Original paper: [Finger Trees: A Simple General-purpose Data Structure](http://www.staff.city.ac.uk/~ross/papers/FingerTree.html)
//!  - Wikipedia article: [FingerTree](https://en.wikipedia.org/wiki/Finger_tree)
//!
//! ## Notes:
//!  - This implementation does not use non-regular recursive types as implementation
//!    described in the paper. As rust's monomorphization does not play well with such types.
//!  - Implmentation abstracts over reference counted types `Rc/Arc`. Using type family trick.
//!  - Uses strict spine in implementation.
//!  - Iterator returns cloned value, and in general this implementation assumes that value
//!    stored in a tree is cheaply clonable, if it is not you can always put it in a `Rc/Arc` or
//!    anything else.
//!
//! ## Examples:
//! ```
//! # extern crate fingertree;
//! # use std::iter::FromIterator;
//! use fingertree::measure::Size;
//! use fingertree::monoid::Sum;
//! use fingertree::{FingerTree, Measured, RcRefs};
//!
//! // construct `Rc` based finger tree with `Size` measure
//! let ft: FingerTree<RcRefs, _> = vec!["one", "two", "three", "four", "five"]
//!     .into_iter()
//!     .map(Size)
//!     .collect();
//! assert_eq!(ft.measure(), Sum(5));
//!
//! // split with predicate
//! let (left, right) = ft.split(|measure| *measure > Sum(2));
//! assert_eq!(left.measure(), Sum(2));
//! assert_eq!(Vec::from_iter(&left), vec![Size("one"), Size("two")]);
//! assert_eq!(right.measure(), Sum(3));
//! assert_eq!(Vec::from_iter(&right), vec![Size("three"), Size("four"), Size("five")]);
//!
//! // concatinate
//! assert_eq!(ft, left + right);
//!
//! // push values
//! assert_eq!(
//!     ft.push_left(Size("left")).push_right(Size("right")),
//!     vec!["left", "one", "two", "three", "four", "five", "right"]
//!          .into_iter()
//!          .map(Size)
//!          .collect(),
//! );
//! ```
#![doc(test(no_crate_inject))]
#![deny(missing_docs)]
#![deny(warnings)]

#[cfg(test)]
#[macro_use]
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
pub use node::NodeInner;
pub use reference::{ArcRefs, RcRefs, Ref, Refs};
pub use tree::{FingerTree, FingerTreeInner};

pub mod rc {
    //! `Rc` based implementation of `FingerTree`

    /// FingerTree based on `Rc` references
    pub type FingerTree<V> = super::FingerTree<super::RcRefs, V>;
}

pub mod sync {
    //! `Arc` based implementation of `FingerTree`

    /// FingerTree based on `Arc` references
    ///
    /// This implementation becomes `{Send|Sync}` if `V: Send + Sync, V::Measure: Send + Sync`
    pub type FingerTree<V> = super::FingerTree<super::ArcRefs, V>;
}
