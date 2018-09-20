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
//! ```rust
//! # use std::iter::FromIterator;
//! use fingertrees::measure::Size;
//! use fingertrees::monoid::Sum;
//! use fingertrees::{FingerTree, Measured, RcRefs};
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
#![deny(missing_docs)]
#![deny(warnings)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod digit;
mod iter;
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
pub use tree::TreeInner;

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

use std::fmt;
use std::iter::FromIterator;
use std::ops::Add;

use iter::Iter;
use node::Node;
use tree::Tree;

/// FingerTree implemenetation
///
/// FingerTree is parametrized by two type parpameters
///   - `R` - type family trick which determines type of references used in
///           implementation. This crate implementes [`ArcRefs`](enum.ArcRefs.html) which is based
///           on `Arc` atomic reference counter, and [`RcRefs`](enum.RcRefs.html) which is based
///           on `Rc`.
///   - `V` - value type which must be measurable and cheaply clonable.
pub struct FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) rec: Tree<R, V>,
}

impl<R, V> Clone for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn clone(&self) -> Self {
        FingerTree {
            rec: self.rec.clone(),
        }
    }
}

impl<R, V> FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    /// Constructs a new, empty `FingerTree`
    ///
    /// Complexity: `O(1)`
    pub fn new() -> Self {
        FingerTree { rec: Tree::empty() }
    }

    /// Returns `true` if finger tree is empty
    ///
    /// Complexity: `O(1)`
    pub fn is_empty(&self) -> bool {
        match self.rec {
            Tree::Empty => true,
            _ => false,
        }
    }

    /// Creates new tree with value prepended to the left side of the tree
    ///
    /// Amortized complexity: `O(1)`
    pub fn push_left(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_left(Node::leaf(value)),
        }
    }

    /// Creates new tree with value prepended to the right side of the tree
    ///
    /// Amortized complexity: `O(1)`
    pub fn push_right(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_right(Node::leaf(value)),
        }
    }

    /// Destrutures tree into a tuple with first element of it containing first
    /// element from the left side of the tree, and second element contains tree
    /// with reset of the elements
    ///
    /// Amortized complexity: `O(1)`
    pub fn view_left(&self) -> Option<(V, Self)> {
        let (head, tail) = self.rec.view_left()?;
        match head.as_ref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    /// Destrutures tree into a tuple with first element of it containing first
    /// element from the left side of the tree, and second element contains tree
    /// with reset of the elements
    ///
    /// Amortized complexity: `O(1)`
    pub fn view_right(&self) -> Option<(V, Self)> {
        let (head, tail) = self.rec.view_right()?;
        match head.as_ref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    /// Destructures tree into two three, using provided predicate.
    ///
    /// Predicate must be monotinic function accepting accumulated measure of elments
    /// and changing its value from `true` to `false`. This function basically behave
    /// as if we would iterate all elements from left to right, and accumlating measure
    /// of all iterated elements, calling predicate on this accumulated value and once
    /// its value flips from `true` to `false` we stop iteration and form two threes
    /// from already iterated elements and the rest of the elements.
    ///
    /// Complexity: `O(ln(N))`
    pub fn split<F>(&self, mut pred: F) -> (FingerTree<R, V>, FingerTree<R, V>)
    where
        F: FnMut(&V::Measure) -> bool,
    {
        if self.is_empty() {
            (Self::new(), Self::new())
        } else if (&mut pred)(&self.measure()) {
            let (l, x, r) = self.rec.split(&V::Measure::unit(), &mut pred);
            (
                FingerTree { rec: l },
                FingerTree {
                    rec: r.push_left(x),
                },
            )
        } else {
            (self.clone(), Self::new())
        }
    }

    /// Construct new finger tree wich is concatination of `self` and `other`
    ///
    /// Complexity: `O(ln(N))`
    pub fn concat(&self, other: &Self) -> Self {
        FingerTree {
            rec: Tree::concat(&self.rec, &mut ::std::iter::empty(), &other.rec),
        }
    }

    /// Double ended iterator visiting all elements of the tree from left to right
    pub fn iter(&self) -> Iter<R, V> {
        Iter::new(self)
    }
}

impl<R, V> Measured for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        self.rec.measure()
    }
}

impl<'a, 'b, R, V> Add<&'b FingerTree<R, V>> for &'a FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Output = FingerTree<R, V>;

    fn add(self, other: &'b FingerTree<R, V>) -> Self::Output {
        self.concat(other)
    }
}

impl<R, V> Add<FingerTree<R, V>> for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Output = FingerTree<R, V>;

    fn add(self, other: Self) -> Self::Output {
        self.concat(&other)
    }
}

impl<R, V> PartialEq for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured + PartialEq,
{
    fn eq(&self, other: &FingerTree<R, V>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
}

impl<R, V> Eq for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured + Eq,
{
}

impl<'a, R, V> IntoIterator for &'a FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;
    type IntoIter = Iter<R, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R, V> IntoIterator for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;
    type IntoIter = Iter<R, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R, V> FromIterator<V> for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        iter.into_iter()
            .fold(FingerTree::new(), |ft, item| ft.push_right(item))
    }
}

impl<R, V> fmt::Debug for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FingerTree")?;
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<R, V> Default for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn default() -> Self {
        FingerTree::new()
    }
}
