#[macro_use]
extern crate criterion;
extern crate fingertree;

use criterion::{Bencher, Criterion, Fun};
use fingertree::measure::Size;
use fingertree::{rc, ArcRefs, FingerTree, RcRefs, Refs};
use std::collections::HashMap;

const KB: usize = 1024;
const SPLIT_1024: &[usize] = &[211, 384, 557, 730, 903];

fn bench_from(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "create from iter",
        |b, &&size| b.iter(|| (0..size).map(Size).collect::<rc::FingerTree<_>>()),
        &[1 * KB, 2 * KB, 4 * KB, 16 * KB],
    );
}

fn bench_split(c: &mut Criterion) {
    let ft: rc::FingerTree<_> = (0..1024).map(Size).collect();
    c.bench_function_over_inputs(
        "split",
        move |b, &size| b.iter(|| ft.split(|m| **m > *size)),
        SPLIT_1024,
    );
}

fn bench_concat(c: &mut Criterion) {
    let ft: rc::FingerTree<_> = (0..1024).map(Size).collect();
    let ft_split: HashMap<_, _> = SPLIT_1024
        .iter()
        .map(|size| (size, ft.split(|m| **m > *size)))
        .collect();

    c.bench_function_over_inputs(
        "concat",
        move |b, k| {
            let (ref left, ref right) = ft_split[*k];
            b.iter(|| left.concat(right))
        },
        SPLIT_1024,
    );
}

fn bench_arc_vs_rc(c: &mut Criterion) {
    fn split_concat<R>(b: &mut Bencher, i: &(usize, usize))
    where
        R: Refs<Size<usize>>,
    {
        let (size, split) = i;
        let ft: FingerTree<R, _> = (0..*size).map(Size).collect();
        b.iter(|| {
            let (left, right) = ft.split(|m| **m > *split);
            left.concat(&right)
        })
    }
    c.bench_functions(
        "split+concat",
        vec![
            Fun::new("arc", split_concat::<ArcRefs>),
            Fun::new("rc", split_concat::<RcRefs>),
        ],
        (16384, 10923),
    );
}

criterion_group! {
    benches,
    bench_from,
    bench_split,
    bench_concat,
    bench_arc_vs_rc,
}

criterion_main!(benches);
