use std::fmt;
use std::iter::FromIterator;
use std::ops::{Add, Deref};

use self::FingerTreeInner::{Deep, Empty, Single};
use digit::Digit;
use measure::Measured;
use monoid::Monoid;
use node::{Node, NodeInner};
use reference::{Ref, Refs};

pub enum FingerTreeInner<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    Empty,
    Single(Node<R, V>),
    Deep {
        measure: V::Measure,
        left: Digit<Node<R, V>>,
        spine: FingerTreeRec<R, V>, //TODO: lazy spine
        right: Digit<Node<R, V>>,
    },
}

impl<R, V> Measured for FingerTreeInner<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        match self {
            Empty => Self::Measure::zero(),
            Single(node) => node.measure(),
            Deep { measure, .. } => measure.clone(),
        }
    }
}

pub struct FingerTreeRec<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) inner: R::Tree,
}

impl<R, V> Measured for FingerTreeRec<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        self.inner.measure()
    }
}

impl<R, V> Clone for FingerTreeRec<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn clone(&self) -> Self {
        FingerTreeRec {
            inner: self.inner.clone(),
        }
    }
}

impl<R, V> FingerTreeRec<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn empty() -> Self {
        FingerTreeRec {
            inner: R::Tree::new(FingerTreeInner::Empty),
        }
    }

    fn single(value: Node<R, V>) -> Self {
        FingerTreeRec {
            inner: R::Tree::new(FingerTreeInner::Single(value)),
        }
    }

    fn deep(left: Digit<Node<R, V>>, spine: FingerTreeRec<R, V>, right: Digit<Node<R, V>>) -> Self {
        let measure = left
            .measure()
            .plus(&spine.inner.measure())
            .plus(&right.measure());
        FingerTreeRec {
            inner: R::Tree::new(FingerTreeInner::Deep {
                measure,
                left,
                spine,
                right,
            }),
        }
    }

    fn push_left(&self, value: Node<R, V>) -> Self {
        match self.inner.deref() {
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

    fn push_right(&self, value: Node<R, V>) -> Self {
        match self.inner.deref() {
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
    fn deep_left(
        left: &[Node<R, V>],
        spine: &FingerTreeRec<R, V>,
        right: &Digit<Node<R, V>>,
    ) -> Self {
        if left.is_empty() {
            match spine.view_left() {
                Some((head, tail)) => Self::deep((&head).into(), tail, right.clone()),
                None => FingerTreeRec::from(right),
            }
        } else {
            Self::deep(left.into(), spine.clone(), right.clone())
        }
    }

    fn view_left(&self) -> Option<(Node<R, V>, Self)> {
        match self.inner.deref() {
            Empty => None,
            Single(value) => Some((value.clone(), FingerTreeRec::empty())),
            Deep {
                left, spine, right, ..
            } => match left.as_ref().split_first() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((head.clone(), Self::deep_left(tail, spine, right))),
            },
        }
    }

    fn deep_right(
        left: &Digit<Node<R, V>>,
        spine: &FingerTreeRec<R, V>,
        right: &[Node<R, V>],
    ) -> Self {
        if right.is_empty() {
            match spine.view_right() {
                Some((head, tail)) => Self::deep(left.clone(), tail, (&head).into()),
                None => FingerTreeRec::from(left),
            }
        } else {
            Self::deep(left.clone(), spine.clone(), right.into())
        }
    }

    fn view_right(&self) -> Option<(Node<R, V>, Self)> {
        match self.inner.deref() {
            Empty => None,
            Single(value) => Some((value.clone(), FingerTreeRec::empty())),
            Deep {
                left, spine, right, ..
            } => match right.as_ref().split_last() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((head.clone(), Self::deep_right(left, spine, tail))),
            },
        }
    }

    fn split<F>(
        &self,
        measure: &V::Measure,
        pred: &mut F,
    ) -> (FingerTreeRec<R, V>, Node<R, V>, FingerTreeRec<R, V>)
    where
        F: FnMut(&V::Measure) -> bool,
    {
        match self.inner.deref() {
            Empty => panic!("recursive split of finger-tree called on empty tree"),
            Single(value) => (
                FingerTreeRec::empty(),
                value.clone(),
                FingerTreeRec::empty(),
            ),
            Deep {
                left, spine, right, ..
            } => {
                // left
                let left_measure = measure.plus(&left.measure());
                if pred(&left_measure) {
                    let (l, x, r) = left.split(measure, pred);
                    return (
                        FingerTreeRec::from(l),
                        x.clone(),
                        Self::deep_left(r, spine, right),
                    );
                }
                // spine
                let spine_measure = left_measure.plus(&spine.measure());
                if pred(&spine_measure) {
                    let (sl, sx, sr) = spine.split(&left_measure, pred);
                    let sx = Digit::from(&sx);
                    let (l, x, r) = sx.split(&left_measure.plus(&sl.measure()), pred);
                    return (
                        Self::deep_right(left, &sl, l),
                        x.clone(),
                        Self::deep_left(r, &sr, right),
                    );
                }
                // right
                let (l, x, r) = right.split(&spine_measure, pred);
                (
                    Self::deep_right(left, spine, l),
                    x.clone(),
                    FingerTreeRec::from(r),
                )
            }
        }
    }

    fn concat(left: &Self, mid: &[Node<R, V>], right: &Self) -> Self {
        match (left.inner.deref(), right.inner.deref()) {
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
                            match *nodes.pop().expect("concat variant violated") {
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

impl<R, T, V> From<T> for FingerTreeRec<R, V>
where
    R: Refs<V>,
    T: AsRef<[Node<R, V>]>,
    V: Measured,
{
    fn from(slice: T) -> Self {
        slice
            .as_ref()
            .iter()
            .fold(FingerTreeRec::empty(), |ft, v| ft.push_right(v.clone()))
    }
}

pub struct FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) rec: FingerTreeRec<R, V>,
}

impl<R, V> Clone for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn clone(&self) -> Self {
        FingerTree {
            rec: self.rec.clone(),
        }
    }
}

impl<R, V> FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub fn new() -> Self {
        FingerTree {
            rec: FingerTreeRec::empty(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self.rec.inner {
            FingerTreeInner::Empty => true,
            _ => false,
        }
    }

    pub fn push_left(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_left(Node::leaf(R::Value::new(value))),
        }
    }

    pub fn push_right(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_right(Node::leaf(R::Value::new(value))),
        }
    }

    pub fn view_left(&self) -> Option<(R::Value, Self)> {
        let (head, tail) = self.rec.view_left()?;
        match head.deref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    pub fn view_right(&self) -> Option<(R::Value, Self)> {
        let (head, tail) = self.rec.view_right()?;
        match head.deref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    pub fn split<F>(&self, mut pred: F) -> (FingerTree<R, V>, FingerTree<R, V>)
    where
        F: FnMut(&V::Measure) -> bool,
    {
        if self.is_empty() {
            (Self::new(), Self::new())
        } else if (&mut pred)(&self.measure()) {
            let (l, x, r) = self.rec.split(&V::Measure::zero(), &mut pred);
            (
                FingerTree { rec: l },
                FingerTree {
                    rec: r.push_left(x),
                },
            )
        } else {
            (self.clone(), Self::new())
        }
    }

    pub fn concat(&self, other: &Self) -> Self {
        FingerTree {
            rec: FingerTreeRec::concat(&self.rec, &[], &other.rec),
        }
    }

    pub fn iter(&self) -> FingerTreeIter<R, V> {
        FingerTreeIter { tail: self.clone() }
    }
}

impl<R, V> Measured for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        self.rec.measure()
    }
}

impl<'a, 'b, R, V> Add<&'b FingerTree<R, V>> for &'a FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Output = FingerTree<R, V>;

    fn add(self, other: &'b FingerTree<R, V>) -> Self::Output {
        self.concat(other)
    }
}

impl<R, V> PartialEq for FingerTree<R, V>
where
    R: Refs<V>,
    R::Value: PartialEq,
    V: Measured,
{
    fn eq(&self, other: &FingerTree<R, V>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
}

impl<R, V> Eq for FingerTree<R, V>
where
    R: Refs<V>,
    R::Value: Eq,
    V: Measured,
{
}

pub struct FingerTreeIter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    tail: FingerTree<R, V>,
}

impl<R, V> Iterator for FingerTreeIter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = R::Value;

    fn next(&mut self) -> Option<Self::Item> {
        let (head, tail) = self.tail.view_left()?;
        self.tail = tail;
        Some(head)
    }
}

impl<R, V> DoubleEndedIterator for FingerTreeIter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (head, tail) = self.tail.view_right()?;
        self.tail = tail;
        Some(head)
    }
}

impl<'a, R, V> IntoIterator for &'a FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = R::Value;
    type IntoIter = FingerTreeIter<R, V>;
    fn into_iter(self) -> Self::IntoIter {
        FingerTreeIter { tail: self.clone() }
    }
}

impl<R, V> IntoIterator for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = R::Value;
    type IntoIter = FingerTreeIter<R, V>;
    fn into_iter(self) -> Self::IntoIter {
        (&self).into_iter()
    }
}

impl<R, V> FromIterator<V> for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        iter.into_iter()
            .fold(FingerTree::new(), |ft, item| ft.push_right(item))
    }
}

impl<R, V> fmt::Debug for FingerTree<R, V>
where
    R: Refs<V>,
    R::Value: fmt::Debug,
    V: Measured,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FingerTree")?;
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<R, V> Default for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn default() -> Self {
        FingerTree::new()
    }
}
