use std::ops::Deref;
use std::rc::Rc;

use measure::Measured;
use monoid::Monoid;

pub(crate) enum NodeInner<V: Measured> {
    Leaf(Rc<V>),
    Node2 {
        measure: V::Measure,
        left: Node<V>,
        right: Node<V>,
    },
    Node3 {
        measure: V::Measure,
        left: Node<V>,
        middle: Node<V>,
        right: Node<V>,
    },
}

pub(crate) struct Node<V: Measured> {
    inner: Rc<NodeInner<V>>,
}

impl<V> Node<V>
where
    V: Measured,
{
    pub(crate) fn leaf(value: Rc<V>) -> Self {
        Node {
            inner: Rc::new(NodeInner::Leaf(value)),
        }
    }

    pub(crate) fn node2(left: Self, right: Self) -> Self {
        let measure = left.measure().plus(&right.measure());
        Node {
            inner: Rc::new(NodeInner::Node2 {
                measure,
                left,
                right,
            }),
        }
    }

    pub(crate) fn node3(left: Self, middle: Self, right: Self) -> Self {
        let measure = left.measure()
            .plus(&middle.measure())
            .plus(&right.measure());
        Node {
            inner: Rc::new(NodeInner::Node3 {
                measure,
                left,
                middle,
                right,
            }),
        }
    }
}

impl<V: Measured> Clone for Node<V> {
    fn clone(&self) -> Self {
        Node {
            inner: self.inner.clone(),
        }
    }
}

impl<V> Measured for Node<V>
where
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        match **self {
            NodeInner::Leaf(ref value) => value.measure(),
            NodeInner::Node2 { ref measure, .. } => measure.clone(),
            NodeInner::Node3 { ref measure, .. } => measure.clone(),
        }
    }
}

impl<V: Measured> Deref for Node<V> {
    type Target = NodeInner<V>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
