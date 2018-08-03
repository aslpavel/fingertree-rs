// NOTES:
//  - this implementation does not use non-regular recursive types, as I was not able
//    to make it types check. Structure itself compiles but any implementaoin fails.
//
// TODO:
//  - improve lifting values to nodes in `.concat`
//  - docs
//  - tests, and maybe quickcheck tests?
//  - Arc implementation?
//  - use more references in function signatures and call clone in the body of functions
//  - lazy spine?

mod digit;
pub mod measure;
pub mod monoid;
mod node;
mod tree;

#[cfg(test)]
mod test;

pub use measure::Measured;
pub use monoid::Monoid;
pub use tree::FingerTree;
