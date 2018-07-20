use std::fmt;
use std::ops::{Add, Deref};
use std::rc::Rc;
// NOTES:
//  - this implementation does not use non-regular recursive types, as I was not able
//    to make it types check. Structure itself compiles but any implementaoin fails.
//
// TODO:
//  - lazy spine?
//  - use sized array for `Node{2,3}`
//  - implement `Digit` as named sized array
//  - add measure

/*
trait Monoid {
    fn zero() -> Self;
    fn add(&self, other: &Self) -> Self;
}

trait Measure {
}
*/

enum NodeInner<V> {
    Leaf(Rc<V>),
    Node2 {
        left: Node<V>,
        right: Node<V>,
    },
    Node3 {
        left: Node<V>,
        middle: Node<V>,
        right: Node<V>,
    },
}

struct Node<V> {
    inner: Rc<NodeInner<V>>,
}

impl<V> Node<V> {
    fn leaf(value: Rc<V>) -> Self {
        Node {
            inner: Rc::new(NodeInner::Leaf(value)),
        }
    }

    fn node2(left: Self, right: Self) -> Self {
        Node {
            inner: Rc::new(NodeInner::Node2 { left, right }),
        }
    }

    fn node3(left: Self, middle: Self, right: Self) -> Self {
        Node {
            inner: Rc::new(NodeInner::Node3 {
                left,
                middle,
                right,
            }),
        }
    }
}

impl<V> Deref for Node<V> {
    type Target = NodeInner<V>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<V> Clone for Node<V> {
    fn clone(&self) -> Self {
        Node {
            inner: self.inner.clone(),
        }
    }
}

// TOOD: implement Digit::{One, Two, Three, Four}
#[derive(Clone)]
struct Digit<V>(Vec<V>);

impl<V> Digit<V> {
    fn slice(&self) -> &[V] {
        self.0.as_slice()
    }

    fn one(v0: V) -> Self {
        Digit(vec![v0])
    }

    fn two(v0: V, v1: V) -> Self {
        Digit(vec![v0, v1])
    }

    fn three(v0: V, v1: V, v2: V) -> Self {
        Digit(vec![v0, v1, v2])
    }

    fn four(v0: V, v1: V, v2: V, v3: V) -> Self {
        Digit(vec![v0, v1, v2, v3])
    }
}

impl<V> AsRef<[V]> for Digit<V> {
    fn as_ref(&self) -> &[V] {
        self.0.as_ref()
    }
}

impl<'a, V, R> Add<R> for &'a Digit<V>
where
    V: Clone,
    R: AsRef<[V]>,
{
    type Output = Digit<V>;

    fn add(self, other: R) -> Self::Output {
        let mut digit = Vec::with_capacity(4);
        digit.extend_from_slice(&self.0);
        digit.extend_from_slice(other.as_ref());
        Digit(digit)
    }
}

impl<'a, V> From<&'a [V]> for Digit<V>
where
    V: Clone,
{
    fn from(slice: &'a [V]) -> Digit<V> {
        let mut digit = Vec::with_capacity(slice.len());
        digit.extend_from_slice(slice);
        Digit(digit)
    }
}

impl<'a, V> From<&'a Node<V>> for Digit<Node<V>> {
    fn from(node: &'a Node<V>) -> Digit<Node<V>> {
        match &node.deref() {
            NodeInner::Leaf(..) => Digit(vec![node.clone()]),
            NodeInner::Node2 { left, right } => Digit(vec![left.clone(), right.clone()]),
            NodeInner::Node3 {
                left,
                middle,
                right,
            } => Digit(vec![left.clone(), middle.clone(), right.clone()]),
        }
    }
}

trait Nat {}
struct Zero {}
impl Nat for Zero {}
struct Succ<N> {
    n: std::marker::PhantomData<N>,
}
impl<N> Nat for Succ<N>
where
    N: Nat,
{
}

enum FingerTreeInner<V> {
    Empty,
    Single(Node<V>),
    Deep {
        left: Digit<Node<V>>,
        spine: FingerTree<V>, //TODO: lazy spine
        right: Digit<Node<V>>,
    },
}

struct FingerTree<V> {
    inner: Rc<FingerTreeInner<V>>,
}

impl<V> Clone for FingerTree<V> {
    fn clone(&self) -> Self {
        FingerTree {
            inner: self.inner.clone(),
        }
    }
}

impl<V> FingerTree<V> {
    pub fn new() -> Self {
        FingerTree {
            inner: Rc::new(FingerTreeInner::Empty),
        }
    }

    fn single(value: Node<V>) -> Self {
        FingerTree {
            inner: Rc::new(FingerTreeInner::Single(value)),
        }
    }

    fn deep(left: Digit<Node<V>>, spine: FingerTree<V>, right: Digit<Node<V>>) -> Self {
        FingerTree {
            inner: Rc::new(FingerTreeInner::Deep { left, spine, right }),
        }
    }

    fn push_left(&self, value: Node<V>) -> Self {
        use FingerTreeInner::{Deep, Empty, Single};
        match self.inner.deref() {
            Empty => Self::single(value),
            Single(other) => Self::deep(Digit::one(value), Self::new(), Digit::one(other.clone())),
            Deep { left, spine, right } => {
                if let [l0, l1, l2, l3] = left.slice() {
                    Self::deep(
                        Digit::two(value, l0.clone()),
                        spine.push_left(Node::node3(l1.clone(), l2.clone(), l3.clone())),
                        right.clone(),
                    )
                } else {
                    Self::deep(&Digit::one(value) + left, spine.clone(), right.clone())
                }
            }
        }
    }

    fn push_right(&self, value: Node<V>) -> Self {
        use FingerTreeInner::{Deep, Empty, Single};
        match self.inner.deref() {
            Empty => Self::single(value),
            Single(other) => Self::deep(Digit::one(other.clone()), Self::new(), Digit::one(value)),
            Deep { left, spine, right } => {
                if let [r0, r1, r2, r3] = right.slice() {
                    Self::deep(
                        left.clone(),
                        spine.push_right(Node::node3(r0.clone(), r1.clone(), r2.clone())),
                        Digit::two(value, r3.clone()),
                    )
                } else {
                    Self::deep(left.clone(), spine.clone(), right + Digit::one(value))
                }
            }
        }
    }

    // left element is not `Digit` because `Digit` cannot be empty, but left in current
    // postion can be.
    fn deep_left(left: &[Node<V>], spine: FingerTree<V>, right: Digit<Node<V>>) -> Self {
        if left.is_empty() {
            match spine.view_left() {
                Some((head, tail)) => Self::deep((&head).into(), tail, right),
                None => right
                    .as_ref()
                    .iter()
                    .cloned()
                    .fold(FingerTree::new(), |ft, node| ft.push_right(node)),
            }
        } else {
            Self::deep(left.into(), spine, right)
        }
    }

    fn view_left(&self) -> Option<(Node<V>, Self)> {
        use FingerTreeInner::{Deep, Empty, Single};
        match self.inner.deref() {
            Empty => None,
            Single(value) => Some((value.clone(), FingerTree::new())),
            Deep { left, spine, right } => match left.as_ref().split_first() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((
                    head.clone(),
                    Self::deep_left(tail, spine.clone(), right.clone()),
                )),
            },
        }
    }

    // pub fn iter(&self) -> impl Iterator<Item = Rc<V>> {
    //     return FingerTreeIter { ft: self.clone() };
    // }
}

// struct FingerTreeIter<V> {
//     ft: FingerTree<V>,
// }

// impl<V> Iterator for FingerTreeIter<V> {
//     type Item = Rc<V>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.ft.view_left() {
//             Some((first, rest)) => {
//                 self.ft = rest;
//                 Some(first)
//             }
//             None => None,
//         }
//     }
// }

impl<V> fmt::Debug for FingerTree<V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FingerTree")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
