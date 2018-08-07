use measure::Measured;
use monoid::Monoid;
use reference::{Ref, Refs};

pub enum NodeInner<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    Leaf(V),
    Node2 {
        measure: V::Measure,
        left: Node<R, V>,
        right: Node<R, V>,
    },
    Node3 {
        measure: V::Measure,
        left: Node<R, V>,
        middle: Node<R, V>,
        right: Node<R, V>,
    },
}

pub struct Node<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    inner: R::Node,
}

impl<R, V> Node<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) fn leaf(value: V) -> Self {
        Node {
            inner: R::Node::new(NodeInner::Leaf(value)),
        }
    }

    pub(crate) fn node2(left: Self, right: Self) -> Self {
        let measure = left.measure().plus(&right.measure());
        Node {
            inner: R::Node::new(NodeInner::Node2 {
                measure,
                left,
                right,
            }),
        }
    }

    pub(crate) fn node3(left: Self, middle: Self, right: Self) -> Self {
        let measure = left
            .measure()
            .plus(&middle.measure())
            .plus(&right.measure());
        Node {
            inner: R::Node::new(NodeInner::Node3 {
                measure,
                left,
                middle,
                right,
            }),
        }
    }
}

impl<R, V> Clone for Node<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn clone(&self) -> Self {
        Node {
            inner: self.inner.clone(),
        }
    }
}

impl<R, V> Measured for Node<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        match self.as_ref() {
            NodeInner::Leaf(value) => value.measure(),
            NodeInner::Node2 { measure, .. } => measure.clone(),
            NodeInner::Node3 { measure, .. } => measure.clone(),
        }
    }
}

impl<R, V> AsRef<NodeInner<R, V>> for Node<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn as_ref(&self) -> &NodeInner<R, V> {
        &*self.inner
    }
}
