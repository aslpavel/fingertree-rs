# FingerTree
[![Build Status][build_badge]][build_url]
[![Coverage Status][coverage_badge]][coverage_url]


FingerTree implementation based on [Finger Trees: A Simple General-purpose Data Structure][paper]

## Features
- abstracted over `Rc/Arc` refernce counting
- uses strict spine (at least for now)
- do not use non-regular recursive types, as I did not manage to make them work in rust

[paper]: http://www.staff.city.ac.uk/~ross/papers/FingerTree.html
[build_badge]: https://travis-ci.org/aslpavel/fingertree-rs.svg?branch=master "build status"
[build_url]: https://travis-ci.org/aslpavel/fingertree-rs
[coverage_badge]: https://coveralls.io/repos/github/aslpavel/fingertree-rs/badge.svg?branch=master
[coverage_url]: https://coveralls.io/github/aslpavel/fingertree-rs?branch=master
