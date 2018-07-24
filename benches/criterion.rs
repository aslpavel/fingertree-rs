#[macro_use]
extern crate criterion;
extern crate fingertree;

use criterion::Criterion;
use fingertree::{FingerTree, Sized};

const KB: usize = 1024;

fn bench_from(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "create from iter",
        |b, &&size| b.iter(|| (0..size).map(Sized).collect::<FingerTree<_>>()),
        &[1 * KB, 2 * KB, 4 * KB, 16 * KB],
    );
}

fn bench_split_1024(c: &mut Criterion) {
    let ft: FingerTree<_> = (0..1024).map(Sized).collect();
    c.bench_function("split 1024 at 111", move |b| {
        b.iter(|| ft.split(|m| m > &111))
    });
}

criterion_group! {
    benches,
    bench_from,
    bench_split_1024,
}

criterion_main!(benches);
