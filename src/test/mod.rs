mod quickcheck;
mod simple;

use std::fmt;

use measure::Measured;
use monoid::Monoid;
use node::{Node, NodeInner};
use reference::Refs;
use tree::{FingerTree, FingerTreeInner, FingerTreeRec};

// constraint that is dynamic in current implementation but static in
// original algorithm due to the fact that rust does not support
// non-regualr recursive types. Each level of spine should add one level
// of depth to all nodes in current level.
pub fn validate<R, V>(ft: &FingerTree<R, V>)
where
    R: Refs<V>,
    V: Measured,
    V::Measure: Eq + PartialEq + fmt::Debug,
{
    fn validate_node_rec<R, V>(depth: usize, node: &Node<R, V>)
    where
        R: Refs<V>,
        V: Measured,
        V::Measure: Eq + PartialEq + fmt::Debug,
    {
        if depth == 0 {
            match node.as_ref() {
                NodeInner::Leaf(..) => (),
                _ => panic!("all zero depth nodes must be leafs"),
            }
        } else {
            match node.as_ref() {
                NodeInner::Leaf(..) => panic!("leaf node with depth: {}", depth),
                NodeInner::Node2 {
                    ref left,
                    ref right,
                    ref measure,
                } => {
                    validate_node_rec(depth - 1, left);
                    validate_node_rec(depth - 1, right);
                    assert_eq!(measure.clone(), left.measure().plus(&right.measure()));
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
                        left.measure()
                            .plus(&middle.measure())
                            .plus(&right.measure())
                    );
                }
            }
        }
    }
    fn validate_ft_rec<R, V>(depth: usize, ft: &FingerTreeRec<R, V>)
    where
        R: Refs<V>,
        V: Measured,
        V::Measure: Eq + PartialEq + fmt::Debug,
    {
        match ft.as_ref() {
            FingerTreeInner::Empty => (),
            FingerTreeInner::Single(ref node) => validate_node_rec(depth, node),
            FingerTreeInner::Deep {
                ref left,
                ref spine,
                ref right,
                ref measure,
            } => {
                let mut m = V::Measure::zero();

                for node in left.as_ref() {
                    validate_node_rec(depth, node);
                    m = m.plus(&node.measure());
                }

                validate_ft_rec(depth + 1, spine);
                m = m.plus(&spine.measure());

                for node in right.as_ref() {
                    validate_node_rec(depth, node);
                    m = m.plus(&node.measure());
                }

                assert_eq!(measure.clone(), m);
            }
        }
    }
    validate_ft_rec(0, &ft.rec)
}
