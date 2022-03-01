# FingerTree
[![Build Status](https://travis-ci.org/aslpavel/fingertree-rs.svg?branch=master)](https://travis-ci.org/aslpavel/fingertree-rs)
[![Coverage Status](https://coveralls.io/repos/github/aslpavel/fingertree-rs/badge.svg?branch=master)](https://coveralls.io/github/aslpavel/fingertree-rs?branch=master)
[![MIT License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Crate](https://img.shields.io/crates/v/fingertrees.svg)](https://crates.io/crates/fingertrees)
[![API Docs](https://docs.rs/fingertrees/badge.svg)](https://docs.rs/fingertrees)

Finger trees is a functional representation of persistent sequences
supporting access to the ends in amortized constant time, and concatenation
and splitting in time logarithmic in the size of the smaller piece. It also
has `split` operation defined in general
form, which can be used to implement sequence, priority queue, search tree,
priority search queue and more data structures.

## Links:
 - Original paper: [Finger Trees: A Simple General-purpose Data Structure](http://www.staff.city.ac.uk/~ross/papers/FingerTree.html)
 - Wikipedia article: [FingerTree](https://en.wikipedia.org/wiki/Finger_tree)

## Notes:
 - This implementation does not use non-regular recursive types as implementation
   described in the paper. As rust's monomorphization does not play well with such types.
 - Implementation abstracts over reference counted types `Rc/Arc`. Using type family trick.
 - Uses strict spine in implementation.
 - Iterator returns cloned value, and in general this implementation assumes that value
   stored in a tree is cheaply clonable, if it is not you can always put it in a `Rc/Arc` or
   anything else.

## Examples:
```rust
use std::iter::FromIterator;
use fingertrees::measure::Size;
use fingertrees::monoid::Sum;
use fingertrees::{FingerTree, Measured, RcRefs};

// construct `Rc` based finger tree with `Size` measure
let ft: FingerTree<RcRefs, _> = vec!["one", "two", "three", "four", "five"]
    .into_iter()
    .map(Size)
    .collect();
assert_eq!(ft.measure(), Sum(5));

// split with predicate
let (left, right) = ft.split(|measure| *measure > Sum(2));
assert_eq!(left.measure(), Sum(2));
assert_eq!(Vec::from_iter(&left), vec![Size("one"), Size("two")]);
assert_eq!(right.measure(), Sum(3));
assert_eq!(Vec::from_iter(&right), vec![Size("three"), Size("four"), Size("five")]);

// concatenate
assert_eq!(ft, left + right);

// push values
assert_eq!(
    ft.push_left(Size("left")).push_right(Size("right")),
    vec!["left", "one", "two", "three", "four", "five", "right"]
         .into_iter()
         .map(Size)
         .collect(),
);
```

