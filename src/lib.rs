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
mod measured;
mod node;
mod tree;

pub use measured::{Measured, Sized};
pub use tree::FingerTree;
