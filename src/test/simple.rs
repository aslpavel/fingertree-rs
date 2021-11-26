use crate::measure::{Measured, Size};
use crate::monoid::Monoid;
use crate::rc::FingerTree as RcFingerTree;
use crate::sync::FingerTree as ArcFingerTree;
use crate::test::validate;

const TEST_SIZE: usize = 512;

#[test]
fn queue() {
    let ft: RcFingerTree<_> = (0..TEST_SIZE).map(Size).collect();
    validate(&ft);
    assert_eq!(*ft.measure(), TEST_SIZE);

    let mut count = 0;
    for (value, expected) in ft.iter().zip(0..) {
        assert_eq!(*value, expected);
        count += 1;
    }
    assert_eq!(*ft.measure(), count);
}

#[test]
fn concat() {
    for split in 0..TEST_SIZE {
        let left: RcFingerTree<_> = (0..split).map(Size).collect();
        let right: RcFingerTree<_> = (split..TEST_SIZE).map(Size).collect();

        let ft = &left + &right;
        assert_eq!(ft.measure(), left.measure().join(&right.measure()));
        validate(&left);
        validate(&right);
        validate(&ft);

        for (value, expected) in ft.iter().zip(0..) {
            assert_eq!(*value, expected, "failed to concat: {:?} {:?}", left, right);
        }
    }
}

#[test]
fn split() {
    let ft: RcFingerTree<_> = (0..TEST_SIZE).map(Size).collect();
    for split in 0..TEST_SIZE {
        let (left, right) = ft.split(|m| **m > split);
        validate(&left);
        validate(&right);
        assert_eq!(*left.measure(), split);
        assert_eq!(*right.measure(), TEST_SIZE - split);
        assert_eq!(ft, &left + &right);
    }
}

#[test]
fn split_left() {
    let ft: RcFingerTree<_> = (0..TEST_SIZE).map(Size).collect();
    for split in 0..TEST_SIZE {
        let left = ft.split_left(|m| **m > split);
        validate(&left);
        assert_eq!(*left.measure(), split);
    }
}

#[test]
fn split_right() {
    let ft: RcFingerTree<_> = (0..TEST_SIZE).map(Size).collect();
    for split in 0..TEST_SIZE {
        let right = ft.split_right(|m| **m > split);
        validate(&right);
        assert_eq!(*right.measure(), TEST_SIZE - split);
    }
}

#[test]
fn reversed() {
    let ft: RcFingerTree<_> = (0..TEST_SIZE).map(Size).collect();
    assert_eq!(
        ft.iter().rev().collect::<Vec<_>>(),
        (0..TEST_SIZE).map(Size).rev().collect::<Vec<_>>()
    );

    let mut iter = ft.iter();
    assert_eq!(
        iter.by_ref().take(TEST_SIZE / 2).collect::<Vec<_>>(),
        (0..TEST_SIZE / 2).map(Size).collect::<Vec<_>>(),
    );
    assert_eq!(
        iter.rev().collect::<Vec<_>>(),
        (TEST_SIZE / 2..TEST_SIZE)
            .rev()
            .map(Size)
            .collect::<Vec<_>>(),
    );
}

#[test]
fn find() {
    let ft: RcFingerTree<_> = (0..TEST_SIZE).map(Size).collect();
    for index in 0..TEST_SIZE {
        assert_eq!(ft.find(|m| **m > index), Some(&Size(index)))
    }
    assert!(ft.find(|m| **m > TEST_SIZE).is_none())
}

#[test]
fn sync_send() {
    trait TestSend: Send {}
    impl<V> TestSend for ArcFingerTree<V>
    where
        V: Measured + Send + Sync,
        V::Measure: Send + Sync,
    {
    }

    trait TestSync: Sync {}
    impl<V> TestSync for ArcFingerTree<V>
    where
        V: Measured + Send + Sync,
        V::Measure: Send + Sync,
    {
    }

    fn is_sync<T: Sync>() {}
    fn is_send<T: Send>() {}
    is_sync::<ArcFingerTree<Size<i32>>>();
    is_send::<ArcFingerTree<Size<i32>>>();
}

#[test]
fn from_slice() {
    for size in 0..TEST_SIZE {
        let vals: Vec<_> = (0..size).map(Size).collect();
        let one: RcFingerTree<_> = vals.iter().cloned().collect();
        let two = RcFingerTree::from(vals.as_slice());
        validate(&one);
        validate(&two);
        assert_eq!(one.measure(), two.measure());
        assert_eq!(one, two);
    }
}
