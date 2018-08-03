use std::fmt;
use std::rc::Rc;

use measure::{Measured, Size};
use monoid::Monoid;
use node::{Node, NodeInner};
use reference::{RcRefs, Refs};
use sync::FingerTree as SyncFingerTree;
use tree::{FingerTree, FingerTreeInner, FingerTreeRec};

const TEST_SIZE: usize = 512;

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
        match *ft.inner {
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

#[test]
fn queue() {
    let ft: FingerTree<RcRefs, _> = (0..TEST_SIZE).map(Size).collect();
    validate(&ft);
    assert_eq!(ft.measure().value, TEST_SIZE);

    let mut count = 0;
    for (value, expected) in ft.iter().zip(0..) {
        assert_eq!(**value, expected);
        count += 1;
    }
    assert_eq!(ft.measure().value, count);
}

#[test]
fn concat() {
    for split in 0..TEST_SIZE {
        let left: FingerTree<RcRefs, _> = (0..split).map(Size).collect();
        let right: FingerTree<RcRefs, _> = (split..TEST_SIZE).map(Size).collect();

        let ft = &left + &right;
        assert_eq!(ft.measure(), left.measure().plus(&right.measure()));
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
    let ft: FingerTree<RcRefs, _> = (0..TEST_SIZE).map(Size).collect();
    for split in 0..TEST_SIZE {
        let (left, right) = ft.split(|m| m.value > split);
        validate(&left);
        validate(&right);
        assert_eq!(left.measure().value, split);
        assert_eq!(right.measure().value, TEST_SIZE - split);
        assert_eq!(ft, &left + &right);
    }
}

#[test]
fn reversed() {
    let ft: FingerTree<RcRefs, _> = (0..TEST_SIZE).map(Size).collect();
    assert_eq!(
        ft.iter().rev().collect::<Vec<_>>(),
        (0..TEST_SIZE)
            .map(|v| Rc::new(Size(v)))
            .rev()
            .collect::<Vec<_>>()
    );
}

#[test]
fn sync_send() {
    fn is_sync<T: Sync>() {}
    fn is_send<T: Send>() {}

    is_sync::<SyncFingerTree<Size<i32>>>();
    is_send::<SyncFingerTree<Size<i32>>>();
}
