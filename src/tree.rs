use self::TreeInner::{Deep, Empty, Single};
use digit::Digit;

use measure::Measured;
use monoid::Monoid;
use node::{Node, NodeInner};
use reference::{Ref, Refs};

/// Only visible to defne custom [`Refs`](trait.Refs.html)
pub enum TreeInner<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    #[doc(hidden)]
    Empty,
    #[doc(hidden)]
    Single(Node<R, V>),
    #[doc(hidden)]
    Deep {
        measure: V::Measure,
        left: Digit<Node<R, V>>,
        spine: Tree<R, V>, //TODO: lazy spine
        right: Digit<Node<R, V>>,
    },
}

pub struct Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    inner: R::Tree,
}

impl<R, V> AsRef<TreeInner<R, V>> for Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn as_ref(&self) -> &TreeInner<R, V> {
        &self.inner
    }
}

impl<R, V> Measured for Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        match self.as_ref() {
            Empty => Self::Measure::unit(),
            Single(node) => node.measure(),
            Deep { measure, .. } => measure.clone(),
        }
    }
}

impl<R, V> Clone for Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn clone(&self) -> Self {
        Tree {
            inner: self.inner.clone(),
        }
    }
}

impl<R, V> Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) fn empty() -> Self {
        Tree {
            inner: R::Tree::new(TreeInner::Empty),
        }
    }

    pub(crate) fn single(value: Node<R, V>) -> Self {
        Tree {
            inner: R::Tree::new(TreeInner::Single(value)),
        }
    }

    pub(crate) fn deep(
        left: Digit<Node<R, V>>,
        spine: Tree<R, V>,
        right: Digit<Node<R, V>>,
    ) -> Self {
        let measure = left.measure().join(&spine.measure()).join(&right.measure());
        Tree {
            inner: R::Tree::new(TreeInner::Deep {
                measure,
                left,
                spine,
                right,
            }),
        }
    }

    pub(crate) fn push_left(&self, value: Node<R, V>) -> Self {
        match self.as_ref() {
            Empty => Self::single(value),
            Single(other) => Self::deep(
                Digit::One([value]),
                Self::empty(),
                Digit::One([other.clone()]),
            ),
            Deep {
                left, spine, right, ..
            } => {
                if let [l0, l1, l2, l3] = left.as_ref() {
                    Self::deep(
                        Digit::Two([value, l0.clone()]),
                        spine.push_left(Node::node3(l1.clone(), l2.clone(), l3.clone())),
                        right.clone(),
                    )
                } else {
                    Self::deep(&Digit::One([value]) + left, spine.clone(), right.clone())
                }
            }
        }
    }

    pub(crate) fn push_right(&self, value: Node<R, V>) -> Self {
        match self.as_ref() {
            Empty => Self::single(value),
            Single(other) => Self::deep(
                Digit::One([other.clone()]),
                Self::empty(),
                Digit::One([value]),
            ),
            Deep {
                left, spine, right, ..
            } => {
                if let [r0, r1, r2, r3] = right.as_ref() {
                    Self::deep(
                        left.clone(),
                        spine.push_right(Node::node3(r0.clone(), r1.clone(), r2.clone())),
                        Digit::Two([r3.clone(), value]),
                    )
                } else {
                    Self::deep(left.clone(), spine.clone(), right + Digit::One([value]))
                }
            }
        }
    }

    // left element is not `Digit` because `Digit` cannot be empty, but left in current
    // postion can be.
    fn deep_left(left: &[Node<R, V>], spine: &Tree<R, V>, right: &Digit<Node<R, V>>) -> Self {
        if left.is_empty() {
            match spine.view_left() {
                Some((head, tail)) => Self::deep((&head).into(), tail, right.clone()),
                None => Tree::from(right),
            }
        } else {
            Self::deep(left.into(), spine.clone(), right.clone())
        }
    }

    pub(crate) fn view_left(&self) -> Option<(Node<R, V>, Self)> {
        match self.as_ref() {
            Empty => None,
            Single(value) => Some((value.clone(), Tree::empty())),
            Deep {
                left, spine, right, ..
            } => match left.as_ref().split_first() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((head.clone(), Self::deep_left(tail, spine, right))),
            },
        }
    }

    fn deep_right(left: &Digit<Node<R, V>>, spine: &Tree<R, V>, right: &[Node<R, V>]) -> Self {
        if right.is_empty() {
            match spine.view_right() {
                Some((head, tail)) => Self::deep(left.clone(), tail, (&head).into()),
                None => Tree::from(left),
            }
        } else {
            Self::deep(left.clone(), spine.clone(), right.into())
        }
    }

    pub(crate) fn view_right(&self) -> Option<(Node<R, V>, Self)> {
        match self.as_ref() {
            Empty => None,
            Single(value) => Some((value.clone(), Tree::empty())),
            Deep {
                left, spine, right, ..
            } => match right.as_ref().split_last() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((head.clone(), Self::deep_right(left, spine, tail))),
            },
        }
    }

    pub(crate) fn split<F>(
        &self,
        measure: &V::Measure,
        pred: &mut F,
    ) -> (Tree<R, V>, Node<R, V>, Tree<R, V>)
    where
        F: FnMut(&V::Measure) -> bool,
    {
        match self.as_ref() {
            Empty => panic!("recursive split of finger-tree called on empty tree"),
            Single(value) => (Tree::empty(), value.clone(), Tree::empty()),
            Deep {
                left, spine, right, ..
            } => {
                // left
                let left_measure = measure.join(&left.measure());
                if pred(&left_measure) {
                    let (l, x, r) = left.split(measure, pred);
                    return (Tree::from(l), x.clone(), Self::deep_left(r, spine, right));
                }
                // spine
                let spine_measure = left_measure.join(&spine.measure());
                if pred(&spine_measure) {
                    let (sl, sx, sr) = spine.split(&left_measure, pred);
                    let sx = Digit::from(&sx);
                    let (l, x, r) = sx.split(&left_measure.join(&sl.measure()), pred);
                    return (
                        Self::deep_right(left, &sl, l),
                        x.clone(),
                        Self::deep_left(r, &sr, right),
                    );
                }
                // right
                let (l, x, r) = right.split(&spine_measure, pred);
                (Self::deep_right(left, spine, l), x.clone(), Tree::from(r))
            }
        }
    }

    pub(crate) fn concat(left: &Self, mid: &[Node<R, V>], right: &Self) -> Self {
        match (left.as_ref(), right.as_ref()) {
            (Empty, _) => mid
                .iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone())),
            (_, Empty) => mid
                .iter()
                .fold(left.clone(), |ft, item| ft.push_right(item.clone())),
            (Single(l), _) => mid
                .iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone()))
                .push_left(l.clone()),
            (_, Single(r)) => mid
                .iter()
                .fold(left.clone(), |ft, item| ft.push_right(item.clone()))
                .push_right(r.clone()),
            (
                Deep {
                    left: left0,
                    spine: spine0,
                    right: right0,
                    ..
                },
                Deep {
                    left: left1,
                    spine: spine1,
                    right: right1,
                    ..
                },
            ) => {
                // lift values to nodes
                let left = right0.as_ref();
                let right = left1.as_ref();

                let mut count = left.len() + mid.len() + right.len();
                let mut iter = left.iter().chain(mid).chain(right);
                let mut nodes = Vec::with_capacity(count / 3 + 1);
                while count != 0 {
                    match (iter.next(), iter.next(), iter.next()) {
                        (Some(v0), Some(v1), Some(v2)) => {
                            count -= 3;
                            nodes.push(Node::node3(v0.clone(), v1.clone(), v2.clone()));
                        }
                        (Some(v0), Some(v1), None) => {
                            count -= 2;
                            nodes.push(Node::node2(v0.clone(), v1.clone()));
                        }
                        (Some(v3), None, _) => {
                            count -= 1;
                            // this cannot be empty as left and right digit contain
                            // at least one element each.
                            match nodes.pop().expect("concat invariant violated").as_ref() {
                                NodeInner::Node3 {
                                    left: ref v0,
                                    middle: ref v1,
                                    right: ref v2,
                                    ..
                                } => {
                                    nodes.push(Node::node2(v0.clone(), v1.clone()));
                                    nodes.push(Node::node2(v2.clone(), v3.clone()));
                                }
                                _ => panic!("only nodes3 must be inserted before this branch"),
                            }
                        }
                        (None, _, _) => {}
                    }
                }

                Self::deep(
                    left0.clone(),
                    Self::concat(spine0, &nodes, spine1),
                    right1.clone(),
                )
            }
        }
    }
}

impl<R, T, V> From<T> for Tree<R, V>
where
    R: Refs<V>,
    T: AsRef<[Node<R, V>]>,
    V: Measured,
{
    fn from(slice: T) -> Self {
        slice
            .as_ref()
            .iter()
            .fold(Tree::empty(), |ft, v| ft.push_right(v.clone()))
    }
}
