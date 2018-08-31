use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use measure::Measured;
use node::NodeInner;
use tree::TreeInner;

/// Interface that all reference types should impelmenet
pub trait Ref: Clone + Deref
where
    Self::Target: Sized,
{
    /// Construct reference from target type
    fn new(value: Self::Target) -> Self;
}

impl<T> Ref for Rc<T> {
    fn new(value: Self::Target) -> Self {
        Rc::new(value)
    }
}

impl<T> Ref for Arc<T> {
    fn new(value: Self::Target) -> Self {
        Arc::new(value)
    }
}

/// Interface which defines all reference types needed by finger tree implementation.
///
/// By implementing this interface for your reference type you can use finger treee
/// with your reference type.
///
/// # Example:
/// ```
/// #[macro_use]
/// extern crate fingertrees;
///
/// use std::rc::Rc;
/// use std::ops::Deref;
/// use fingertrees::measure::Size;
/// use fingertrees::{FingerTree, Measured, Ref};
///
/// // your custom reference type
/// struct MyRef<T>(Rc<T>);
///
/// impl<T> Clone for MyRef<T> {
///     fn clone(&self) -> Self {
///         MyRef(self.0.clone())
///     }
/// }
///
/// impl<T> Deref for MyRef<T> {
///     type Target = T;
///     fn deref(&self) -> &T {
///         &*self.0
///     }
/// }
///
/// impl<T> Ref for MyRef<T> {
///     fn new(value: T) -> Self {
///         MyRef(Rc::new(value))
///     }
/// }
///
/// // define type family for your reference
/// fingertree_define_refs!(MyRefs, MyRef);
///
/// # fn main() {
/// // now you can construct fingertree using your reference type
/// let _: FingerTree<MyRefs, _> = (0..128).map(Size).collect();
/// # }
/// ```
pub trait Refs<V>: Sized
where
    V: Measured,
{
    /// Reference on a `Node`
    type Node: Ref<Target = NodeInner<Self, V>>;
    /// Reference on a `Tree`
    type Tree: Ref<Target = TreeInner<Self, V>>;
}

/// Helper macro to define custom [`Refs`](trait.Refs.html) for `FingerTree`
#[macro_export]
macro_rules! fingertree_define_refs {
    (pub $refs:ident, $ref:ident) => {
        /// References type family
        pub enum $refs {}
        fingertree_define_refs!(@refs_impl $refs, $ref);
    };

    ($refs:ident, $ref:ident) => {
        /// References type family
        enum $refs {}
        fingertree_define_refs!(@refs_impl $refs, $ref);
    };

    (@refs_impl $refs:ident, $ref:ident) => {
        impl<V> $crate::Refs<V> for $refs
        where
            V: $crate::measure::Measured,
        {
            type Node = $ref<$crate::NodeInner<Self, V>>;
            type Tree = $ref<$crate::TreeInner<Self, V>>;
        }
    };
}

fingertree_define_refs!(pub RcRefs, Rc);
fingertree_define_refs!(pub ArcRefs, Arc);
