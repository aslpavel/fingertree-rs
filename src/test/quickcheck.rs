use measure::{Measured, Size};
use quickcheck::{Arbitrary, Gen};
use sync::FingerTree;
use test::validate;

impl<V> Arbitrary for FingerTree<V>
where
    V: Arbitrary + Measured + Sync,
    V::Measure: Send + Sync,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let vec: Vec<V> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<Iterator<Item = Self>> {
        let vec: Vec<_> = self.into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<Self>()))
    }
}

impl<V> Arbitrary for Size<V>
where
    V: Arbitrary,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Size(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<Iterator<Item = Self>> {
        Box::new((**self).shrink().map(|v| Size(v)))
    }
}

quickcheck! {
    fn split_and_concat(ft: FingerTree<Size<i32>>, index: usize) -> bool {
        let len = *ft.measure();
        let index = if len != 0 { index % len } else { 0 };
        let (left, right) = ft.split(|m| m.value > index);
        validate(&left);
        validate(&right);
        true
            // correct split
            && *left.measure() == index
            && *right.measure() == len - index
            // concat is inverse to split
            && left.concat(&right) == ft
    }
}
