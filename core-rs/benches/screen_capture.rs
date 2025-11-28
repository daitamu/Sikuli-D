//! Benchmarks for screen capture performance
//! 画面キャプチャのパフォーマンスベンチマーク

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sikulid::{Region, Screen};

fn benchmark_screen_capture_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_capture_full");

    let screen = Screen::primary();

    group.bench_function("primary_screen", |b| {
        b.iter(|| {
            let _ = screen.capture();
        })
    });

    group.finish();
}

fn benchmark_screen_capture_region(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_capture_region");

    let screen = Screen::primary();

    // Test different region sizes
    for size in [100, 500, 1000].iter() {
        let region = Region::new(0, 0, *size, *size);

        group.bench_with_input(
            BenchmarkId::new("region_size", format!("{}x{}", size, size)),
            &region,
            |b, region| {
                b.iter(|| {
                    let _ = screen.capture_region(black_box(region));
                })
            },
        );
    }

    group.finish();
}

fn benchmark_screen_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_dimensions");

    let mut screen = Screen::primary();

    group.bench_function("get_dimensions", |b| {
        b.iter(|| {
            let _ = screen.dimensions();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_screen_capture_full,
    benchmark_screen_capture_region,
    benchmark_screen_dimensions
);

criterion_main!(benches);
