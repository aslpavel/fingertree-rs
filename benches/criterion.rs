use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput};
use fingertrees::measure::Size;
use fingertrees::{rc, ArcRefs, FingerTree, Measured, RcRefs, Refs};
use std::collections::HashMap;

const KB: usize = 1024;
const SPLIT_1024: &[usize] = &[211, 384, 557, 730, 903];

fn ft_from(c: &mut Criterion) {
    let mut group = c.benchmark_group("from");
    for size in [KB, 2 * KB, 4 * KB, 16 * KB] {
        let vals: Vec<_> = (0..size).map(Size).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("iter", size), &vals, |b, vals| {
            b.iter(|| vals.iter().cloned().collect::<rc::FingerTree<_>>())
        });
        group.bench_with_input(BenchmarkId::new("slice", size), &vals, |b, vals| {
            b.iter(|| rc::FingerTree::from(vals.as_slice()))
        });
    }
    group.finish();
}

fn ft_split(c: &mut Criterion) {
    let ft: rc::FingerTree<_> = (0..1024).map(Size).collect();

    let mut group = c.benchmark_group("split");
    for position in SPLIT_1024 {
        group.bench_with_input(
            BenchmarkId::new("position", position),
            position,
            |b, position| b.iter(|| ft.split(|m| **m > *position)),
        );
    }
    group.finish();
}

fn ft_split_left(c: &mut Criterion) {
    let ft: rc::FingerTree<_> = (0..1024).map(Size).collect();

    let mut group = c.benchmark_group("split_left");
    for position in SPLIT_1024 {
        group.bench_with_input(
            BenchmarkId::new("position", position),
            position,
            |b, position| b.iter(|| ft.split_left(|m| **m > *position)),
        );
    }
    group.finish();
}

fn ft_split_right(c: &mut Criterion) {
    let ft: rc::FingerTree<_> = (0..1024).map(Size).collect();

    let mut group = c.benchmark_group("split_right");
    for position in SPLIT_1024 {
        group.bench_with_input(
            BenchmarkId::new("position", position),
            position,
            |b, position| b.iter(|| ft.split_right(|m| **m > *position)),
        );
    }
    group.finish();
}

fn ft_concat(c: &mut Criterion) {
    let ft: rc::FingerTree<_> = (0..1024).map(Size).collect();
    let ft_split: HashMap<_, _> = SPLIT_1024
        .iter()
        .map(|size| (size, ft.split(|m| **m > *size)))
        .collect();

    let mut group = c.benchmark_group("concat");
    for (position, (left, right)) in ft_split {
        group.bench_with_input(
            BenchmarkId::new("position", position),
            &(left, right),
            |b, (left, right)| b.iter(|| left.concat(right)),
        );
    }
    group.finish();
}

/// Iterator based destructuring FingerTree with `view`
struct ViewIter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    tail: FingerTree<R, V>,
}

impl<R, V> ViewIter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn new(tail: &FingerTree<R, V>) -> Self {
        ViewIter { tail: tail.clone() }
    }
}

impl<R, V> Iterator for ViewIter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let (head, tail) = self.tail.view_left()?;
        self.tail = tail;
        Some(head)
    }
}

fn ft_iter(c: &mut Criterion) {
    let len = 65536;
    let ft: rc::FingerTree<_> = (0..len).map(Size).collect();
    let mut group = c.benchmark_group("iterator");
    group.bench_with_input(BenchmarkId::new("stack", len), &ft, |b, ft| {
        b.iter(|| ft.iter().count())
    });
    group.bench_with_input(BenchmarkId::new("view", len), &ft, |b, ft| {
        b.iter(|| ViewIter::new(ft).count())
    });
    group.finish();
}

fn ft_arc_vs_rc(c: &mut Criterion) {
    fn split_concat<R>(b: &mut Bencher, size_split: &(usize, usize))
    where
        R: Refs<Size<usize>>,
    {
        let (size, split) = size_split;
        let ft: FingerTree<R, _> = (0..*size).map(Size).collect();
        b.iter(|| {
            let (left, right) = ft.split(|m| **m > *split);
            left.concat(&right)
        })
    }
    let size_split = (16384, 10923);
    let mut group = c.benchmark_group("split+concat");
    group.bench_with_input(
        BenchmarkId::new("arc", size_split.0),
        &size_split,
        split_concat::<ArcRefs>,
    );
    group.bench_with_input(
        BenchmarkId::new("rc", size_split.0),
        &size_split,
        split_concat::<RcRefs>,
    );
    group.finish();
}

criterion_group! {
    benches,
    ft_arc_vs_rc,
    ft_concat,
    ft_from,
    ft_iter,
    ft_split,
    ft_split_left,
    ft_split_right,
}

criterion_main!(benches);
