use measure::Measured;
use quickcheck::{Arbitrary, Gen};
use sync::FingerTree;

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
