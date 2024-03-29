use crate::measure::{Measured, Size};
use crate::sync::FingerTree;
use crate::test::validate;
use quickcheck::{quickcheck, Arbitrary, Gen};

impl<V> Arbitrary for FingerTree<V>
where
    V: Arbitrary + Measured + Sync,
    V::Measure: Send + Sync,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let vec: Vec<V> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let vec: Vec<_> = self.into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<Self>()))
    }
}

impl<V> Arbitrary for Size<V>
where
    V: Arbitrary,
{
    fn arbitrary(g: &mut Gen) -> Self {
        Size(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((**self).shrink().map(Size))
    }
}

quickcheck! {
    fn split_and_concat(ft: FingerTree<Size<i32>>, index: usize) -> bool {
        let len = *ft.measure();
        let index = if len != 0 { index % len } else { 0 };
        let (left, right) = ft.split(|m| **m > index);
        validate(&left);
        validate(&right);
        true
            // correct split
            && *left.measure() == index
            && *right.measure() == len - index
            // concat is inverse to split
            && left.concat(&right) == ft
    }

    fn from_slice(items: Vec<Size<usize>>) -> bool {
        let ft = FingerTree::from(items.as_slice());
        validate(&ft);
        items.as_slice().measure() == ft.measure()
    }
}
