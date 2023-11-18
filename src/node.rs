use std::mem;

use crate::measure::Measured;
use crate::monoid::Monoid;
use crate::reference::{Ref, Refs};

/// Only visible to define custom [`Refs`](trait.Refs.html)
pub enum NodeInner<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    #[doc(hidden)]
    Leaf(V),
    #[doc(hidden)]
    Node2 {
        measure: V::Measure,
        left: Node<R, V>,
        right: Node<R, V>,
    },
    #[doc(hidden)]
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
        let measure = left.measure().join(&right.measure());
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
            .join(&middle.measure())
            .join(&right.measure());
        Node {
            inner: R::Node::new(NodeInner::Node3 {
                measure,
                left,
                middle,
                right,
            }),
        }
    }

    pub(crate) fn find<F>(&self, measure: V::Measure, pred: &mut F) -> &V
    where
        F: FnMut(&V::Measure) -> bool,
    {
        match self.as_ref() {
            NodeInner::Leaf(leaf) => leaf,
            NodeInner::Node2 { left, right, .. } => {
                let left_measure = measure.join(&left.measure());
                if pred(&left_measure) {
                    left.find(measure, pred)
                } else {
                    right.find(left_measure, pred)
                }
            }
            NodeInner::Node3 {
                left,
                middle,
                right,
                ..
            } => {
                let left_measure = measure.join(&left.measure());
                if pred(&left_measure) {
                    return left.find(measure, pred);
                }
                let middle_measure = left_measure.join(&middle.measure());
                if pred(&middle_measure) {
                    return middle.find(left_measure, pred);
                }
                right.find(middle_measure, pred)
            }
        }
    }

    /// Lift iterator of nodes into iterator of nodes, which are one level deeper
    ///
    /// NOTE: will panic on the iterator with less than two elements
    pub(crate) fn lift<I>(iter: I) -> LiftNodesIter<I::IntoIter, R, V>
    where
        I: IntoIterator<Item = Node<R, V>>,
    {
        LiftNodesIter::new(iter.into_iter())
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
        &self.inner
    }
}

// Iterator decorator which takes iterator of `Nodes` and make them one level deeper (lift)
// by combining adjacent nodes. What we want is essentially
// ```
// nodes :: [a] -> [Node a]
// nodes [a, b] = [Node2 a b]
// nodes [a, b, c] = [Node3 a b c]
// nodes [a, b, c, d] = [Node2 a b, Node2 c d]
// nodes (a : b : c : xs) = Node3 a b c : nodes xs
// ```
pub(crate) struct LiftNodesIter<I, R, V>
where
    I: Iterator<Item = Node<R, V>>,
    R: Refs<V>,
    V: Measured,
{
    buff: [Option<Node<R, V>>; 5], // look ahead ring buffer
    index: u8,                     // current index in buffer
    left: u8,                      // nodes left in buffer
    iter: I,
}

impl<I, R, V> LiftNodesIter<I, R, V>
where
    I: Iterator<Item = Node<R, V>>,
    R: Refs<V>,
    V: Measured,
{
    fn new(mut iter: I) -> Self {
        let buff = [
            iter.next(),
            iter.next(),
            iter.next(),
            iter.next(),
            iter.next(),
        ];
        let left = buff.iter().map(|n| if n.is_some() { 1 } else { 0 }).sum();
        LiftNodesIter {
            buff,
            index: 0,
            left,
            iter,
        }
    }

    fn buff_next(&mut self) -> Node<R, V> {
        let next = self.iter.next();
        if next.is_none() {
            self.left -= 1;
        }
        let node = mem::replace(&mut self.buff[self.index as usize], next).unwrap();
        self.index = (self.index + 1) % 5;
        node
    }
}

impl<I, R, V> Iterator for LiftNodesIter<I, R, V>
where
    I: Iterator<Item = Node<R, V>>,
    R: Refs<V>,
    V: Measured,
{
    type Item = Node<R, V>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.left {
            0 => None,
            1 => panic!("Node::lift is called on a single element interator"),
            2 | 4 => Some(Node::node2(self.buff_next(), self.buff_next())),
            _ => Some(Node::node3(
                self.buff_next(),
                self.buff_next(),
                self.buff_next(),
            )),
        }
    }
}
