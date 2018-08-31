use std::ops::Add;

use measure::Measured;
use monoid::Monoid;
use node::{Node, NodeInner};
use reference::Refs;

#[derive(Clone)]
pub enum Digit<V> {
    One([V; 1]),
    Two([V; 2]),
    Three([V; 3]),
    Four([V; 4]),
}

impl<V> Digit<V>
where
    V: Measured,
{
    pub(crate) fn split<F>(&self, measure: &V::Measure, pred: &mut F) -> (&[V], &V, &[V])
    where
        F: FnMut(&V::Measure) -> bool,
    {
        let slice = self.as_ref();
        if slice.len() == 1 {
            (&[], &slice[0], &[])
        } else {
            let slice = self.as_ref();
            let mut measure = measure.clone();
            for (index, item) in slice.iter().enumerate() {
                measure = measure.join(&item.measure());
                if pred(&measure) {
                    return (&slice[..index], &slice[index], &slice[index + 1..]);
                }
            }
            let index = slice.len() - 1;
            (&slice[..index], &slice[index], &[])
        }
    }
}

impl<'a, V, R> Add<R> for &'a Digit<V>
where
    V: Clone,
    R: AsRef<[V]>,
{
    type Output = Digit<V>;

    fn add(self, other: R) -> Self::Output {
        match (self.as_ref(), other.as_ref()) {
            (_, []) => self.clone(),
            ([v0], [v1]) => Digit::Two([v0.clone(), v1.clone()]),
            ([v0], [v1, v2]) => Digit::Three([v0.clone(), v1.clone(), v2.clone()]),
            ([v0], [v1, v2, v3]) => Digit::Four([v0.clone(), v1.clone(), v2.clone(), v3.clone()]),
            ([v0, v1], [v2]) => Digit::Three([v0.clone(), v1.clone(), v2.clone()]),
            ([v0, v1], [v2, v3]) => Digit::Four([v0.clone(), v1.clone(), v2.clone(), v3.clone()]),
            ([v0, v1, v2], [v3]) => Digit::Four([v0.clone(), v1.clone(), v2.clone(), v3.clone()]),
            _ => panic!(
                "impossible to create digit of size: {}",
                self.as_ref().len() + other.as_ref().len(),
            ),
        }
    }
}

impl<V: Measured> Measured for Digit<V> {
    type Measure = V::Measure;

    fn measure(&self) -> Self::Measure {
        self.as_ref()
            .iter()
            .fold(Self::Measure::unit(), |measure, val| {
                measure.join(&val.measure())
            })
    }
}

impl<V> AsRef<[V]> for Digit<V> {
    fn as_ref(&self) -> &[V] {
        match self {
            Digit::One(v) => v,
            Digit::Two(v) => v,
            Digit::Three(v) => v,
            Digit::Four(v) => v,
        }
    }
}

impl<'a, V> From<&'a [V]> for Digit<V>
where
    V: Clone,
{
    fn from(slice: &'a [V]) -> Digit<V> {
        match slice {
            [v0] => Digit::One([v0.clone()]),
            [v0, v1] => Digit::Two([v0.clone(), v1.clone()]),
            [v0, v1, v2] => Digit::Three([v0.clone(), v1.clone(), v2.clone()]),
            [v0, v1, v2, v3] => Digit::Four([v0.clone(), v1.clone(), v2.clone(), v3.clone()]),
            _ => panic!("immposible to create digit from of size: {}", slice.len()),
        }
    }
}

impl<'a, R, V: Measured> From<&'a Node<R, V>> for Digit<Node<R, V>>
where
    R: Refs<V>,
    V: Measured,
{
    fn from(node: &'a Node<R, V>) -> Digit<Node<R, V>> {
        match node.as_ref() {
            NodeInner::Leaf(..) => Digit::One([node.clone()]),
            NodeInner::Node2 { left, right, .. } => Digit::Two([left.clone(), right.clone()]),
            NodeInner::Node3 {
                left,
                middle,
                right,
                ..
            } => Digit::Three([left.clone(), middle.clone(), right.clone()]),
        }
    }
}
