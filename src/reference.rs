use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use measure::Measured;
use node::NodeInner;
use tree::FingerTreeInner;

pub trait Ref: Clone + Deref {
    fn new(value: Self::Target) -> Self;
}

impl<T> Ref for Rc<T> {
    fn new(value: T) -> Self {
        Rc::new(value)
    }
}

impl<T> Ref for Arc<T> {
    fn new(value: T) -> Self {
        Arc::new(value)
    }
}

pub trait Refs<V>: Sized
where
    V: Measured,
{
    type Value: Ref<Target = V>;
    type Node: Ref<Target = NodeInner<Self, V>>;
    type Tree: Ref<Target = FingerTreeInner<Self, V>>;
}

pub enum RcRefs {}

impl<V> Refs<V> for RcRefs
where
    V: Measured,
{
    type Value = Rc<V>;
    type Node = Rc<NodeInner<Self, V>>;
    type Tree = Rc<FingerTreeInner<Self, V>>;
}

pub enum ArcRefs {}

impl<V> Refs<V> for ArcRefs
where
    V: Measured,
{
    type Value = Arc<V>;
    type Node = Arc<NodeInner<Self, V>>;
    type Tree = Arc<FingerTreeInner<Self, V>>;
}
