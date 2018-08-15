use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use measure::Measured;
use node::NodeInner;
use tree::FingerTreeInner;

/// Interface that all reference types should impelmenet
pub trait Ref: Clone + Deref
where
    Self::Target: Sized,
{
    /// Construct reference from target type
    fn new(value: Self::Target) -> Self;
}

/// Interface which defines all reference types needed by finger tree
/// implementation.
pub trait Refs<V>: Sized
where
    V: Measured,
{
    /// Reference on a `Node`
    type Node: Ref<Target = NodeInner<Self, V>>;
    /// Reference on a `Tree`
    type Tree: Ref<Target = FingerTreeInner<Self, V>>;
}

macro_rules! define_refs {
    ($ref:ident, $refs:ident) => {
        impl<T> $crate::reference::Ref for $ref<T> {
            fn new(value: T) -> Self {
                $ref::new(value)
            }
        }

        /// References type family
        pub enum $refs {}

        impl<V> $crate::reference::Refs<V> for $refs
        where
            V: $crate::measure::Measured,
        {
            type Node = $ref<$crate::node::NodeInner<Self, V>>;
            type Tree = $ref<$crate::tree::FingerTreeInner<Self, V>>;
        }
    };
}

define_refs!(Rc, RcRefs);
define_refs!(Arc, ArcRefs);
