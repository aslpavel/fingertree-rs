use self::Tree::{Deep, Empty, Single};
use crate::digit::Digit;
use crate::measure::Measured;
use crate::monoid::Monoid;
use crate::node::Node;
use crate::reference::{Ref, Refs};

/// Only visible to defne custom [`Refs`](trait.Refs.html)
pub struct TreeInner<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) measure: V::Measure,
    pub(crate) left: Digit<Node<R, V>>,
    pub(crate) spine: Tree<R, V>, //TODO: lazy spine
    pub(crate) right: Digit<Node<R, V>>,
}

pub enum Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    Empty,
    Single(Node<R, V>),
    Deep(R::Tree),
}

impl<R, V> Measured for Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        match self {
            Empty => Self::Measure::unit(),
            Single(node) => node.measure(),
            Deep(deep) => deep.measure.clone(),
        }
    }
}

impl<R, V> Clone for Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn clone(&self) -> Self {
        match self {
            Empty => Empty,
            Single(node) => Single(node.clone()),
            Deep(deep) => Deep(deep.clone()),
        }
    }
}

impl<R, V> Tree<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) fn empty() -> Self {
        Tree::Empty
    }

    pub(crate) fn single(node: Node<R, V>) -> Self {
        Tree::Single(node)
    }

    pub(crate) fn deep(
        left: Digit<Node<R, V>>,
        spine: Tree<R, V>,
        right: Digit<Node<R, V>>,
    ) -> Self {
        let measure = left.measure().join(&spine.measure()).join(&right.measure());
        Tree::Deep(R::Tree::new(TreeInner {
            measure,
            left,
            spine,
            right,
        }))
    }

    pub(crate) fn push_left(&self, value: Node<R, V>) -> Self {
        match self {
            Empty => Self::single(value),
            Single(other) => Self::deep(
                Digit::One([value]),
                Self::empty(),
                Digit::One([other.clone()]),
            ),
            Deep(deep) => {
                if let [l0, l1, l2, l3] = deep.left.as_ref() {
                    Self::deep(
                        Digit::Two([value, l0.clone()]),
                        deep.spine
                            .push_left(Node::node3(l1.clone(), l2.clone(), l3.clone())),
                        deep.right.clone(),
                    )
                } else {
                    Self::deep(
                        &Digit::One([value]) + &deep.left,
                        deep.spine.clone(),
                        deep.right.clone(),
                    )
                }
            }
        }
    }

    pub(crate) fn push_right(&self, value: Node<R, V>) -> Self {
        match self {
            Empty => Self::single(value),
            Single(other) => Self::deep(
                Digit::One([other.clone()]),
                Self::empty(),
                Digit::One([value]),
            ),
            Deep(deep) => {
                if let [r0, r1, r2, r3] = deep.right.as_ref() {
                    Self::deep(
                        deep.left.clone(),
                        deep.spine
                            .push_right(Node::node3(r0.clone(), r1.clone(), r2.clone())),
                        Digit::Two([r3.clone(), value]),
                    )
                } else {
                    Self::deep(
                        deep.left.clone(),
                        deep.spine.clone(),
                        &deep.right + Digit::One([value]),
                    )
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
        match self {
            Empty => None,
            Single(value) => Some((value.clone(), Tree::empty())),
            Deep(deep) => match deep.left.as_ref().split_first() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((
                    head.clone(),
                    Self::deep_left(tail, &deep.spine, &deep.right),
                )),
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
        match self {
            Empty => None,
            Single(value) => Some((value.clone(), Tree::empty())),
            Deep(deep) => match deep.right.as_ref().split_last() {
                None => panic!("digit cannot be empty"),
                Some((head, tail)) => Some((
                    head.clone(),
                    Self::deep_right(&deep.left, &deep.spine, tail),
                )),
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
        match self {
            Empty => panic!("recursive split of finger-tree called on empty tree"),
            Single(value) => (Tree::empty(), value.clone(), Tree::empty()),
            Deep(deep) => {
                // left
                let left_measure = measure.join(&deep.left.measure());
                if pred(&left_measure) {
                    let (l, x, r) = deep.left.split(measure, pred);
                    return (
                        Tree::from(l),
                        x.clone(),
                        Self::deep_left(r, &deep.spine, &deep.right),
                    );
                }
                // spine
                let spine_measure = left_measure.join(&deep.spine.measure());
                if pred(&spine_measure) {
                    let (sl, sx, sr) = deep.spine.split(&left_measure, pred);
                    let sx = Digit::from(&sx);
                    let (l, x, r) = sx.split(&left_measure.join(&sl.measure()), pred);
                    return (
                        Self::deep_right(&deep.left, &sl, l),
                        x.clone(),
                        Self::deep_left(r, &sr, &deep.right),
                    );
                }
                // right
                let (l, x, r) = deep.right.split(&spine_measure, pred);
                (
                    Self::deep_right(&deep.left, &deep.spine, l),
                    x.clone(),
                    Tree::from(r),
                )
            }
        }
    }

    fn push_left_many(self, iter: &mut dyn Iterator<Item = Node<R, V>>) -> Self {
        match iter.next() {
            None => self,
            Some(node) => self.push_left_many(iter).push_left(node),
        }
    }

    fn push_right_many(self, iter: &mut dyn Iterator<Item = Node<R, V>>) -> Self {
        match iter.next() {
            None => self,
            Some(node) => self.push_right(node).push_right_many(iter),
        }
    }

    pub(crate) fn concat(
        left: &Self,
        mid: &mut dyn Iterator<Item = Node<R, V>>,
        right: &Self,
    ) -> Self {
        match (left, right) {
            (Empty, _) => right.clone().push_left_many(mid),
            (_, Empty) => left.clone().push_right_many(mid),
            (Single(left), _) => right.clone().push_left_many(mid).push_left(left.clone()),
            (_, Single(right)) => left.clone().push_right_many(mid).push_right(right.clone()),
            (Deep(deep0), Deep(deep1)) => {
                let left = deep0.right.as_ref().iter().cloned();
                let right = deep1.left.as_ref().iter().cloned();
                Self::deep(
                    deep0.left.clone(),
                    Self::concat(
                        &deep0.spine,
                        &mut Node::lift(left.chain(mid).chain(right)),
                        &deep1.spine,
                    ),
                    deep1.right.clone(),
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
