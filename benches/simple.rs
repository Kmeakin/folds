use criterion::{black_box, criterion_group, criterion_main, Criterion};
use folds::Fold;

fn sum_and_product_std(xs: &[u32]) -> (u32, u32) {
    let sum = xs.iter().copied().sum();
    let prod = xs.iter().copied().product();
    (sum, prod)
}

fn sum_and_product_fused(xs: &[u32]) -> (u32, u32) {
    let sum = folds::from_fn(0_u32, u32::wrapping_add);
    let prod = folds::from_fn(1_u32, u32::wrapping_mul);
    let mut fold = sum.try_zip(prod);
    fold.fold(xs.iter().copied())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let xs: Vec<_> = (1..).take(10_000_000).collect();

    c.bench_function("sum and product, std iterator folds", |b| {
        b.iter(|| sum_and_product_std(black_box(&xs)));
    });

    c.bench_function("sum and product, fused folds", |b| {
        b.iter(|| sum_and_product_fused(black_box(&xs)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
