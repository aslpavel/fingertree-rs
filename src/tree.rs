use std::collections::VecDeque;
use std::fmt;
use std::iter::{FromIterator, FusedIterator};
use std::ops::Add;

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
            Empty => Self::Measure::zero(),
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
    fn empty() -> Self {
        Tree {
            inner: R::Tree::new(TreeInner::Empty),
        }
    }

    fn single(value: Node<R, V>) -> Self {
        Tree {
            inner: R::Tree::new(TreeInner::Single(value)),
        }
    }

    fn deep(left: Digit<Node<R, V>>, spine: Tree<R, V>, right: Digit<Node<R, V>>) -> Self {
        let measure = left.measure().plus(&spine.measure()).plus(&right.measure());
        Tree {
            inner: R::Tree::new(TreeInner::Deep {
                measure,
                left,
                spine,
                right,
            }),
        }
    }

    fn push_left(&self, value: Node<R, V>) -> Self {
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

    fn push_right(&self, value: Node<R, V>) -> Self {
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

    fn view_left(&self) -> Option<(Node<R, V>, Self)> {
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

    fn view_right(&self) -> Option<(Node<R, V>, Self)> {
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

    fn split<F>(&self, measure: &V::Measure, pred: &mut F) -> (Tree<R, V>, Node<R, V>, Tree<R, V>)
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
                let left_measure = measure.plus(&left.measure());
                if pred(&left_measure) {
                    let (l, x, r) = left.split(measure, pred);
                    return (Tree::from(l), x.clone(), Self::deep_left(r, spine, right));
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
                (Self::deep_right(left, spine, l), x.clone(), Tree::from(r))
            }
        }
    }

    fn concat(left: &Self, mid: &[Node<R, V>], right: &Self) -> Self {
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

/// FingerTree implemenetation
///
/// FingerTree is parametrized by two type parpameters
///   - `R` - type family trick which determines type of references used in
///           implementation. This crate implementes [`ArcRefs`](enum.ArcRefs.html) which is based
///           on `Arc` atomic reference counter, and [`RcRefs`](enum.RcRefs.html) which is based
///           on `Rc`.
///   - `V` - value type which must be measurable and cheaply clonable.
pub struct FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) rec: Tree<R, V>,
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
    /// Constructs a new, empty `FingerTree`
    ///
    /// Complexity: `O(1)`
    pub fn new() -> Self {
        FingerTree { rec: Tree::empty() }
    }

    /// Returns `true` if finger tree is empty
    ///
    /// Complexity: `O(1)`
    pub fn is_empty(&self) -> bool {
        match self.rec.as_ref() {
            TreeInner::Empty => true,
            _ => false,
        }
    }

    /// Creates new tree with value prepended to the left side of the tree
    ///
    /// Amortized complexity: `O(1)`
    pub fn push_left(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_left(Node::leaf(value)),
        }
    }

    /// Creates new tree with value prepended to the right side of the tree
    ///
    /// Amortized complexity: `O(1)`
    pub fn push_right(&self, value: V) -> Self {
        FingerTree {
            rec: self.rec.push_right(Node::leaf(value)),
        }
    }

    /// Destrutures tree into a tuple with first element of it containing first
    /// element from the left side of the tree, and second element contains tree
    /// with reset of the elements
    ///
    /// Amortized complexity: `O(1)`
    pub fn view_left(&self) -> Option<(V, Self)> {
        let (head, tail) = self.rec.view_left()?;
        match head.as_ref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    /// Destrutures tree into a tuple with first element of it containing first
    /// element from the left side of the tree, and second element contains tree
    /// with reset of the elements
    ///
    /// Amortized complexity: `O(1)`
    pub fn view_right(&self) -> Option<(V, Self)> {
        let (head, tail) = self.rec.view_right()?;
        match head.as_ref() {
            NodeInner::Leaf(value) => Some((value.clone(), FingerTree { rec: tail })),
            _ => panic!("not leaf returned from to level finger-tree"),
        }
    }

    /// Destructures tree into two three, using provided predicate.
    ///
    /// Predicate must be monotinic function accepting accumulated measure of elments
    /// and changing its value from `true` to `false`. This function basically behave
    /// as if we would iterate all elements from left to right, and accumlating measure
    /// of all iterated elements, calling predicate on this accumulated value and once
    /// its value flips from `true` to `false` we stop iteration and form two threes
    /// from already iterated elements and the rest of the elements.
    ///
    /// Complexity: `O(ln(N))`
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

    /// Construct new finger tree wich is concatination of `self` and `other`
    ///
    /// Complexity: `O(ln(N))`
    pub fn concat(&self, other: &Self) -> Self {
        FingerTree {
            rec: Tree::concat(&self.rec, &[], &other.rec),
        }
    }

    /// Double ended iterator visiting all elements of the tree from left to right
    pub fn iter(&self) -> Iter<R, V> {
        Iter::new(self)
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

impl<R, V> Add<FingerTree<R, V>> for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Output = FingerTree<R, V>;

    fn add(self, other: Self) -> Self::Output {
        self.concat(&other)
    }
}

impl<R, V> PartialEq for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured + PartialEq,
{
    fn eq(&self, other: &FingerTree<R, V>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
}

impl<R, V> Eq for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured + Eq,
{
}

enum IterFrame<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    Node(Node<R, V>),
    Tree(Tree<R, V>),
}

pub struct Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    frames: VecDeque<IterFrame<R, V>>,
}

impl<R, V> Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn new(ft: &FingerTree<R, V>) -> Self {
        let mut frames = VecDeque::new();
        frames.push_back(IterFrame::Tree(ft.rec.clone()));
        Iter { frames }
    }
}

impl<R, V> FusedIterator for Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
}

impl<R, V> Iterator for Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.frames.pop_back()? {
                IterFrame::Node(node) => match node.as_ref() {
                    NodeInner::Leaf(value) => return Some(value.clone()),
                    NodeInner::Node2 { left, right, .. } => {
                        self.frames.push_back(IterFrame::Node(right.clone()));
                        self.frames.push_back(IterFrame::Node(left.clone()));
                        continue;
                    }
                    NodeInner::Node3 {
                        left,
                        middle,
                        right,
                        ..
                    } => {
                        self.frames.push_back(IterFrame::Node(right.clone()));
                        self.frames.push_back(IterFrame::Node(middle.clone()));
                        self.frames.push_back(IterFrame::Node(left.clone()));
                        continue;
                    }
                },
                IterFrame::Tree(tree) => match tree.as_ref() {
                    Empty => continue,
                    Single(node) => {
                        self.frames.push_back(IterFrame::Node(node.clone()));
                        continue;
                    }
                    Deep {
                        left, spine, right, ..
                    } => {
                        for node in right.as_ref().iter().rev() {
                            self.frames.push_back(IterFrame::Node(node.clone()));
                        }
                        self.frames.push_back(IterFrame::Tree(spine.clone()));
                        for node in left.as_ref().iter().rev() {
                            self.frames.push_back(IterFrame::Node(node.clone()));
                        }
                        continue;
                    }
                },
            }
        }
    }
}

impl<R, V> DoubleEndedIterator for Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.frames.pop_front()? {
                IterFrame::Node(node) => match node.as_ref() {
                    NodeInner::Leaf(value) => return Some(value.clone()),
                    NodeInner::Node2 { left, right, .. } => {
                        self.frames.push_front(IterFrame::Node(left.clone()));
                        self.frames.push_front(IterFrame::Node(right.clone()));
                        continue;
                    }
                    NodeInner::Node3 {
                        left,
                        middle,
                        right,
                        ..
                    } => {
                        self.frames.push_front(IterFrame::Node(left.clone()));
                        self.frames.push_front(IterFrame::Node(middle.clone()));
                        self.frames.push_front(IterFrame::Node(right.clone()));
                        continue;
                    }
                },
                IterFrame::Tree(tree) => match tree.as_ref() {
                    Empty => continue,
                    Single(node) => {
                        self.frames.push_front(IterFrame::Node(node.clone()));
                        continue;
                    }
                    Deep {
                        left, spine, right, ..
                    } => {
                        for node in left.as_ref() {
                            self.frames.push_front(IterFrame::Node(node.clone()));
                        }
                        self.frames.push_front(IterFrame::Tree(spine.clone()));
                        for node in right.as_ref() {
                            self.frames.push_front(IterFrame::Node(node.clone()));
                        }
                        continue;
                    }
                },
            }
        }
    }
}

impl<'a, R, V> IntoIterator for &'a FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;
    type IntoIter = Iter<R, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R, V> IntoIterator for FingerTree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;
    type IntoIter = Iter<R, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    V: Measured + fmt::Debug,
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
