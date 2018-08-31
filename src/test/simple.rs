use measure::{Measured, Size};
use monoid::Monoid;
use rc::FingerTree as RcFingerTree;
use sync::FingerTree as ArcFingerTree;
use test::validate;

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
fn sync_send() {
    trait TestSend: Send {}
    impl<V> TestSend for ArcFingerTree<V>
    where
        V: Measured + Send + Sync,
        V::Measure: Send + Sync,
    {}

    trait TestSync: Sync {}
    impl<V> TestSync for ArcFingerTree<V>
    where
        V: Measured + Send + Sync,
        V::Measure: Send + Sync,
    {}

    fn is_sync<T: Sync>() {}
    fn is_send<T: Send>() {}
    is_sync::<ArcFingerTree<Size<i32>>>();
    is_send::<ArcFingerTree<Size<i32>>>();
}
