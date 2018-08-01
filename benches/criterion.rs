#[macro_use]
extern crate criterion;
extern crate fingertree;

use criterion::Criterion;
use fingertree::measure::Size;
use fingertree::FingerTree;
use std::collections::HashMap;

const KB: usize = 1024;
const SPLIT_1024: &[usize] = &[211, 384, 557, 730, 903];

fn bench_from(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "create from iter",
        |b, &&size| b.iter(|| (0..size).map(Size).collect::<FingerTree<_>>()),
        &[1 * KB, 2 * KB, 4 * KB, 16 * KB],
    );
}

fn bench_split(c: &mut Criterion) {
    let ft: FingerTree<_> = (0..1024).map(Size).collect();
    c.bench_function_over_inputs(
        "split",
        move |b, &size| b.iter(|| ft.split(|m| m.value > *size)),
        SPLIT_1024,
    );
}

fn bench_concat(c: &mut Criterion) {
    let ft: FingerTree<_> = (0..1024).map(Size).collect();
    let ft_split: HashMap<_, _> = SPLIT_1024
        .iter()
        .map(|size| (size, ft.split(|m| m.value > *size)))
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

criterion_group! {
    benches,
    bench_from,
    bench_split,
    bench_concat,
}

criterion_main!(benches);
