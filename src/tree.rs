use std::fmt;
use std::iter::FromIterator;
use std::ops::{Add, Deref};
use std::rc::Rc;

use self::FingerTreeInner::{Deep, Empty, Single};
use digit::Digit;
use measured::Measured;
use node::{Node, NodeInner};

enum FingerTreeInner<V: Measured> {
    Empty,
    Single(Node<V>),
    Deep {
        measure: V::Measure,
        left: Digit<Node<V>>,
        spine: FingerTreeRec<V>, //TODO: lazy spine
        right: Digit<Node<V>>,
    },
}

impl<V: Measured> Measured for FingerTreeInner<V> {
    type Measure = V::Measure;
    fn measure_zero() -> Self::Measure {
        V::measure_zero()
    }
    fn measure(&self) -> Self::Measure {
        match self {
            Empty => Self::measure_zero(),
            Single(node) => node.measure(),
            Deep { measure, .. } => measure.clone(),
        }
    }
}

struct FingerTreeRec<V: Measured> {
    inner: Rc<FingerTreeInner<V>>,
}

impl<V: Measured> Measured for FingerTreeRec<V> {
    type Measure = V::Measure;
    fn measure_zero() -> Self::Measure {
        V::measure_zero()
    }
    fn measure(&self) -> Self::Measure {
        self.inner.measure()
    }
}

impl<V: Measured> Clone for FingerTreeRec<V> {
    fn clone(&self) -> Self {
        FingerTreeRec {
            inner: self.inner.clone(),
        }
    }
}

impl<V: Measured> FingerTreeRec<V> {
    fn empty() -> Self {
        FingerTreeRec {
            inner: Rc::new(FingerTreeInner::Empty),
        }
    }

    fn single(value: Node<V>) -> Self {
        FingerTreeRec {
            inner: Rc::new(FingerTreeInner::Single(value)),
        }
    }

    fn deep(left: Digit<Node<V>>, spine: FingerTreeRec<V>, right: Digit<Node<V>>) -> Self {
        let measure = left.measure() + spine.inner.measure() + right.measure();
        FingerTreeRec {
            inner: Rc::new(FingerTreeInner::Deep {
                measure,
                left,
                spine,
                right,
            }),
        }
    }

    fn push_left(&self, value: Node<V>) -> Self {
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

    fn push_right(&self, value: Node<V>) -> Self {
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
    fn deep_left(left: &[Node<V>], spine: &FingerTreeRec<V>, right: &Digit<Node<V>>) -> Self {
        if left.is_empty() {
            match spine.view_left() {
                Some((head, tail)) => Self::deep((&head).into(), tail, right.clone()),
                None => FingerTreeRec::from(right),
            }
        } else {
            Self::deep(left.into(), spine.clone(), right.clone())
        }
    }

    fn view_left(&self) -> Option<(Node<V>, Self)> {
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

    fn deep_right(left: &Digit<Node<V>>, spine: &FingerTreeRec<V>, right: &[Node<V>]) -> Self {
        if right.is_empty() {
            match spine.view_right() {
                Some((head, tail)) => Self::deep(left.clone(), tail, (&head).into()),
                None => FingerTreeRec::from(left),
            }
        } else {
            Self::deep(left.clone(), spine.clone(), right.into())
        }
    }

    fn view_right(&self) -> Option<(Node<V>, Self)> {
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
    ) -> (FingerTreeRec<V>, Node<V>, FingerTreeRec<V>)
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
                let left_measure = measure.clone() + left.measure();
                if pred(&left_measure) {
                    let (l, x, r) = left.split(measure, pred);
                    return (
                        FingerTreeRec::from(l),
                        x.clone(),
                        Self::deep_left(r, spine, right),
                    );
                }
                // spine
                let spine_measure = left_measure.clone() + spine.measure();
                if pred(&spine_measure) {
                    let (sl, sx, sr) = spine.split(&left_measure, pred);
                    let sx = Digit::from(&sx);
                    let (l, x, r) = sx.split(&(left_measure + sl.measure()), pred);
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

    fn concat(left: &Self, mid: &[Node<V>], right: &Self) -> Self {
        match (left.inner.deref(), right.inner.deref()) {
            (Empty, _) => mid.iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone())),
            (_, Empty) => mid.iter()
                .fold(left.clone(), |ft, item| ft.push_right(item.clone())),
            (Single(l), _) => mid.iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone()))
                .push_left(l.clone()),
            (_, Single(r)) => mid.iter()
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

impl<T, V> From<T> for FingerTreeRec<V>
where
    T: AsRef<[Node<V>]>,
    V: Measured,
{
    fn from(slice: T) -> Self {
        slice
            .as_ref()
            .iter()
            .fold(FingerTreeRec::empty(), |ft, v| ft.push_right(v.clone()))
    }
}

pub struct FingerTree<V: Measured> {
    rec: FingerTreeRec<V>,
}

impl<V: Measured> Clone for FingerTree<V> {
    fn clone(&self) -> Self {
        FingerTree {
            rec: self.rec.clone(),
        }
    }
}

impl<V: Measured> FingerTree<V> {
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
            rec: self.rec.push_left(Node::leaf(value.into())),
        }
    }

    pub fn push_right(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_right(Node::leaf(value.into())),
        }
    }

    pub fn view_left(&self) -> Option<(Rc<V>, Self)> {
        let (head, tail) = self.rec.view_left()?;
        match head.deref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    pub fn view_right(&self) -> Option<(Rc<V>, Self)> {
        let (head, tail) = self.rec.view_right()?;
        match head.deref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    pub fn split<F>(&self, mut pred: F) -> (FingerTree<V>, FingerTree<V>)
    where
        F: FnMut(&V::Measure) -> bool,
    {
        if self.is_empty() {
            (Self::new(), Self::new())
        } else if (&mut pred)(&self.measure()) {
            let (l, x, r) = self.rec.split(&V::measure_zero(), &mut pred);
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

    pub fn iter(&self) -> FingerTreeIter<V> {
        FingerTreeIter { tail: self.clone() }
    }
}

impl<V: Measured> Measured for FingerTree<V> {
    type Measure = V::Measure;
    fn measure_zero() -> Self::Measure {
        V::measure_zero()
    }
    fn measure(&self) -> Self::Measure {
        self.rec.measure()
    }
}

impl<'a, 'b, V: Measured> Add<&'b FingerTree<V>> for &'a FingerTree<V> {
    type Output = FingerTree<V>;

    fn add(self, other: &'b FingerTree<V>) -> Self::Output {
        self.concat(other)
    }
}

impl<V> PartialEq for FingerTree<V>
where
    V: Measured + PartialEq,
{
    fn eq(&self, other: &FingerTree<V>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
}

impl<V> Eq for FingerTree<V>
where
    V: Measured + Eq,
{
}

pub struct FingerTreeIter<V: Measured> {
    tail: FingerTree<V>,
}

impl<V: Measured> Iterator for FingerTreeIter<V> {
    type Item = Rc<V>;

    fn next(&mut self) -> Option<Self::Item> {
        let (head, tail) = self.tail.view_left()?;
        self.tail = tail;
        Some(head)
    }
}

impl<V: Measured> DoubleEndedIterator for FingerTreeIter<V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (head, tail) = self.tail.view_right()?;
        self.tail = tail;
        Some(head)
    }
}

impl<'a, V: Measured> IntoIterator for &'a FingerTree<V> {
    type Item = Rc<V>;
    type IntoIter = FingerTreeIter<V>;
    fn into_iter(self) -> Self::IntoIter {
        FingerTreeIter { tail: self.clone() }
    }
}

impl<V: Measured> IntoIterator for FingerTree<V> {
    type Item = Rc<V>;
    type IntoIter = FingerTreeIter<V>;
    fn into_iter(self) -> Self::IntoIter {
        (&self).into_iter()
    }
}

impl<V: Measured> FromIterator<V> for FingerTree<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        iter.into_iter()
            .fold(FingerTree::new(), |ft, item| ft.push_right(item))
    }
}

impl<V: Measured> fmt::Debug for FingerTree<V>
where
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FingerTree")?;
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<V: Measured> Default for FingerTree<V> {
    fn default() -> Self {
        FingerTree::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use measured::Sized;

    const COUNT: usize = 256;

    // constraint that is dynamic in current implementation but static in
    // original algorithm due to the fact that rust does not support
    // non-regualr recursive types. Each level of spine should add one level
    // of depth to all nodes in current level.
    fn validate<V>(ft: &FingerTree<V>)
    where
        V: Measured,
        V::Measure: Eq + PartialEq + fmt::Debug,
    {
        fn validate_node_rec<V>(depth: usize, node: &Node<V>)
        where
            V: Measured,
            V::Measure: Eq + PartialEq + fmt::Debug,
        {
            if depth == 0 {
                match **node {
                    NodeInner::Leaf(..) => (),
                    _ => panic!("all zero depth nodes must be leafs"),
                }
            } else {
                match **node {
                    NodeInner::Leaf(..) => panic!("leaf node with depth: {}", depth),
                    NodeInner::Node2 {
                        ref left,
                        ref right,
                        ref measure,
                    } => {
                        validate_node_rec(depth - 1, left);
                        validate_node_rec(depth - 1, right);
                        assert_eq!(measure.clone(), left.measure() + right.measure());
                    }
                    NodeInner::Node3 {
                        ref left,
                        ref middle,
                        ref right,
                        ref measure,
                    } => {
                        validate_node_rec(depth - 1, left);
                        validate_node_rec(depth - 1, middle);
                        validate_node_rec(depth - 1, right);
                        assert_eq!(
                            measure.clone(),
                            left.measure() + middle.measure() + right.measure()
                        );
                    }
                }
            }
        }
        fn validate_ft_rec<V>(depth: usize, ft: &FingerTreeRec<V>)
        where
            V: Measured,
            V::Measure: Eq + PartialEq + fmt::Debug,
        {
            match *ft.inner {
                FingerTreeInner::Empty => (),
                FingerTreeInner::Single(ref node) => validate_node_rec(depth, node),
                FingerTreeInner::Deep {
                    ref left,
                    ref spine,
                    ref right,
                    ref measure,
                } => {
                    let mut m = V::measure_zero();

                    for node in left.as_ref() {
                        validate_node_rec(depth, node);
                        m = m + node.measure();
                    }

                    validate_ft_rec(depth + 1, spine);
                    m = m + spine.measure();

                    for node in right.as_ref() {
                        validate_node_rec(depth, node);
                        m = m + node.measure();
                    }

                    assert_eq!(measure.clone(), m);
                }
            }
        }
        validate_ft_rec(0, &ft.rec)
    }

    #[test]
    fn queue() {
        let ft: FingerTree<_> = (0..COUNT).map(Sized).collect();
        validate(&ft);
        assert_eq!(ft.measure(), COUNT);

        let mut count = 0;
        for (value, expected) in ft.iter().zip(0..) {
            assert_eq!(**value, expected);
            count += 1;
        }
        assert_eq!(ft.measure(), count);
    }

    #[test]
    fn concat() {
        for split in 0..COUNT {
            let left: FingerTree<_> = (0..split).map(Sized).collect();
            let right: FingerTree<_> = (split..COUNT).map(Sized).collect();

            let ft = &left + &right;
            assert_eq!(ft.measure(), left.measure() + right.measure());
            validate(&left);
            validate(&right);
            validate(&ft);

            for (value, expected) in ft.iter().zip(0..) {
                assert_eq!(
                    **value, expected,
                    "failed to concat: {:?} {:?}",
                    left, right
                );
            }
        }
    }

    #[test]
    fn split() {
        let ft: FingerTree<_> = (0..COUNT).map(Sized).collect();
        for split in 0..COUNT {
            let (left, right) = ft.split(|m| m > &split);
            validate(&left);
            validate(&right);
            assert_eq!(left.measure(), split);
            assert_eq!(right.measure(), COUNT - split);
            assert_eq!(ft, &left + &right);
        }
    }

    #[test]
    fn reversed() {
        let ft: FingerTree<_> = (0..COUNT).map(Sized).collect();
        assert_eq!(
            ft.iter().rev().collect::<Vec<_>>(),
            (0..COUNT)
                .map(|v| Rc::new(Sized(v)))
                .rev()
                .collect::<Vec<_>>()
        );
    }
}
