use std::fmt;
use std::iter::FromIterator;
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
//  - use more references in function signatures and call clone in the body of functions

/// `Measured::Measure` must form `Monoid`
pub trait Measured {
    type Measure: Add<Output = Self::Measure> + Clone;
    fn measure_zero() -> Self::Measure;
    fn measure(&self) -> Self::Measure;
}

enum NodeInner<V: Measured> {
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

struct Node<V: Measured> {
    inner: Rc<NodeInner<V>>,
}

impl<V> Measured for Node<V>
where
    V: Measured,
{
    type Measure = V::Measure;
    fn measure_zero() -> Self::Measure {
        V::measure_zero()
    }
    fn measure(&self) -> Self::Measure {
        match self.deref() {
            NodeInner::Leaf(value) => value.measure(),
            NodeInner::Node2 { measure, .. } => measure.clone(),
            NodeInner::Node3 { measure, .. } => measure.clone(),
        }
    }
}

impl<V: Measured> Node<V> {
    fn leaf(value: Rc<V>) -> Self {
        Node {
            inner: Rc::new(NodeInner::Leaf(value)),
        }
    }

    fn node2(left: Self, right: Self) -> Self {
        let measure = left.measure() + right.measure();
        Node {
            inner: Rc::new(NodeInner::Node2 {
                measure,
                left,
                right,
            }),
        }
    }

    fn node3(left: Self, middle: Self, right: Self) -> Self {
        let measure = left.measure() + middle.measure() + right.measure();
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

impl<V: Measured> Deref for Node<V> {
    type Target = NodeInner<V>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<V: Measured> Clone for Node<V> {
    fn clone(&self) -> Self {
        Node {
            inner: self.inner.clone(),
        }
    }
}

// TODO: implement Digit::{One, Two, Three, Four}
#[derive(Clone, Debug)]
struct Digit<V>(Vec<V>);

impl<V: Measured> Measured for Digit<V> {
    type Measure = V::Measure;
    fn measure_zero() -> Self::Measure {
        V::measure_zero()
    }
    fn measure(&self) -> Self::Measure {
        self.as_ref()
            .iter()
            .fold(Self::measure_zero(), |measure, val| measure + val.measure())
    }
}

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

impl<V: Measured> Digit<V> {
    fn split<F>(&self, measure: &V::Measure, mut pred: F) -> (&[V], &V, &[V])
    where
        F: FnMut(&V::Measure) -> bool,
    {
        assert!(!self.0.is_empty(), "digit cannot be empty");
        if self.0.len() == 1 {
            (&[], &self.0[0], &[])
        } else {
            let slice = self.as_ref();
            let mut measure = measure.clone();
            for (index, item) in slice.iter().enumerate() {
                measure = measure + item.measure();
                if pred(&measure) {
                    return (&self.0[..index], &self.0[index], &self.0[index + 1..]);
                }
            }
            let index = self.0.len() - 1;
            (&self.0[..index], &self.0[index], &[])
        }
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

impl<'a, V: Measured> From<&'a Node<V>> for Digit<Node<V>> {
    fn from(node: &'a Node<V>) -> Digit<Node<V>> {
        match &node.deref() {
            NodeInner::Leaf(..) => Digit(vec![node.clone()]),
            NodeInner::Node2 { left, right, .. } => Digit(vec![left.clone(), right.clone()]),
            NodeInner::Node3 {
                left,
                middle,
                right,
                ..
            } => Digit(vec![left.clone(), middle.clone(), right.clone()]),
        }
    }
}

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
        use FingerTreeInner::{Deep, Empty, Single};
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
        use FingerTreeInner::{Deep, Empty, Single};
        match self.inner.deref() {
            Empty => Self::single(value),
            Single(other) => {
                Self::deep(Digit::one(value), Self::empty(), Digit::one(other.clone()))
            }
            Deep {
                left, spine, right, ..
            } => {
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
            Single(other) => {
                Self::deep(Digit::one(other.clone()), Self::empty(), Digit::one(value))
            }
            Deep {
                left, spine, right, ..
            } => {
                if let [r0, r1, r2, r3] = right.slice() {
                    Self::deep(
                        left.clone(),
                        spine.push_right(Node::node3(r0.clone(), r1.clone(), r2.clone())),
                        Digit::two(r3.clone(), value),
                    )
                } else {
                    Self::deep(left.clone(), spine.clone(), right + Digit::one(value))
                }
            }
        }
    }

    fn concat(left: &Self, mid: &[Node<V>], right: &Self) -> Self {
        use FingerTreeInner::{Deep, Empty, Single};
        match (left.inner.deref(), right.inner.deref()) {
            (Empty, _) => mid.iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone())),
            (_, Empty) => mid.iter()
                .fold(left.clone(), |ft, item| ft.push_right(item.clone())),
            (Single(l), _) => mid.iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone()))
                .push_left(l.clone()),
            (_, Single(r)) => mid.iter()
                .rfold(right.clone(), |ft, item| ft.push_left(item.clone()))
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
                let mut vals: Vec<_> = right0
                    .as_ref()
                    .iter()
                    .chain(mid.iter())
                    .chain(left1.as_ref().iter())
                    .collect();
                let mut current = vals.as_slice();
                let mut nodes = Vec::new();
                while !current.is_empty() {
                    let consumed = match current {
                        &[v0, v1] => {
                            nodes.push(Node::node2(v0.clone(), v1.clone()));
                            2
                        }
                        &[v0, v1, v2] => {
                            nodes.push(Node::node3(v0.clone(), v1.clone(), v2.clone()));
                            3
                        }
                        &[v0, v1, v2, v3] => {
                            nodes.push(Node::node2(v0.clone(), v1.clone()));
                            nodes.push(Node::node2(v2.clone(), v3.clone()));
                            4
                        }
                        _ => {
                            if let (&[v0, v1, v2], _) = current.split_at(3) {
                                nodes.push(Node::node3(v0.clone(), v1.clone(), v2.clone()));
                                3
                            } else {
                                unreachable!()
                            }
                        }
                    };
                    current = current.split_at(consumed).1;
                }
                Self::deep(
                    left0.clone(),
                    Self::concat(spine0, &nodes, spine1),
                    right1.clone(),
                )
            }
        }
    }

    // left element is not `Digit` because `Digit` cannot be empty, but left in current
    // postion can be.
    fn deep_left(left: &[Node<V>], spine: FingerTreeRec<V>, right: Digit<Node<V>>) -> Self {
        if left.is_empty() {
            match spine.view_left() {
                Some((head, tail)) => Self::deep((&head).into(), tail, right),
                None => right
                    .as_ref()
                    .iter()
                    .cloned()
                    .fold(FingerTreeRec::empty(), |ft, node| ft.push_right(node)),
            }
        } else {
            Self::deep(left.into(), spine, right)
        }
    }

    fn view_left(&self) -> Option<(Node<V>, Self)> {
        use FingerTreeInner::{Deep, Empty, Single};
        match self.inner.deref() {
            Empty => None,
            Single(value) => Some((value.clone(), FingerTreeRec::empty())),
            Deep {
                left, spine, right, ..
            } => match left.as_ref().split_first() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((
                    head.clone(),
                    Self::deep_left(tail, spine.clone(), right.clone()),
                )),
            },
        }
    }

    fn deep_right(left: Digit<Node<V>>, spine: FingerTreeRec<V>, right: &[Node<V>]) -> Self {
        if right.is_empty() {
            match spine.view_right() {
                Some((head, tail)) => Self::deep(left, tail, (&head).into()),
                None => left.as_ref()
                    .iter()
                    .cloned()
                    .fold(FingerTreeRec::empty(), |ft, node| ft.push_right(node)),
            }
        } else {
            Self::deep(left, spine, right.into())
        }
    }

    fn view_right(&self) -> Option<(Node<V>, Self)> {
        use FingerTreeInner::{Deep, Empty, Single};
        match self.inner.deref() {
            Empty => None,
            Single(value) => Some((value.clone(), FingerTreeRec::empty())),
            Deep {
                left, spine, right, ..
            } => match right.as_ref().split_last() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((
                    head.clone(),
                    Self::deep_right(left.clone(), spine.clone(), tail),
                )),
            },
        }
    }

    fn split<F>(
        &self,
        measure: &V::Measure,
        mut pred: F,
    ) -> (FingerTreeRec<V>, Node<V>, FingerTreeRec<V>)
    where
        F: FnMut(&V::Measure) -> bool,
    {
        use FingerTreeInner::{Deep, Empty, Single};
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
                let left_measure = left.measure();
                if pred(&left_measure) {
                    let (l, x, r) = left.split(measure, pred);
                    return (
                        l.as_ref()
                            .iter()
                            .fold(FingerTreeRec::empty(), |ft, v| ft.push_right(v.clone())),
                        x.clone(),
                        Self::deep_left(r, spine.clone(), right.clone()),
                    );
                }
                // spine
                let spine_measure = left_measure.clone() + spine.measure();
                if pred(&spine_measure) {
                    let (sl, sx, sr) = spine.split(&left_measure, &mut pred);
                    let sx = Digit::from(&sx);
                    let (l, x, r) = sx.split(&(left_measure + sl.measure()), pred);
                    return (
                        Self::deep_right(left.clone(), sl, l),
                        x.clone(),
                        Self::deep_left(r, sr, right.clone()),
                    );
                }
                // right
                let (l, x, r) = right.split(&spine_measure, pred);
                (
                    Self::deep_right(left.clone(), spine.clone(), l),
                    x.clone(),
                    r.as_ref()
                        .iter()
                        .fold(FingerTreeRec::empty(), |ft, v| ft.push_right(v.clone())),
                )
            }
        }
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
        } else if pred(&self.measure()) {
            let (l, x, r) = self.rec.split(&V::measure_zero(), pred);
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

impl<'a, V: Measured> Add for &'a FingerTree<V> {
    type Output = FingerTree<V>;

    fn add(self, other: &FingerTree<V>) -> Self::Output {
        FingerTree {
            rec: FingerTreeRec::concat(&self.rec, &[], &other.rec),
        }
    }
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
        return FingerTreeIter { tail: self.clone() };
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

#[derive(Debug)]
struct Sized<T>(T);

impl<T> Measured for Sized<T> {
    type Measure = usize;
    fn measure_zero() -> Self::Measure {
        0
    }
    fn measure(&self) -> Self::Measure {
        1
    }
}

fn main() {
    let ft: FingerTree<_> = (0..32).map(Sized).collect();
    let vec: Vec<_> = ft.iter().rev().collect();
    println!("{:?} {:?}", ft.measure(), &ft + &ft);
    println!("{:?}", vec);
}
