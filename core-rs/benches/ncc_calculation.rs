//! Benchmarks for NCC (Normalized Cross-Correlation) calculation
//! NCC（正規化相互相関）計算のパフォーマンスベンチマーク

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{DynamicImage, ImageEncoder, Rgba, RgbaImage};
use sikulix_core::{ImageMatcher, Pattern};

/// Create a test screen image with random noise
/// ランダムノイズを含むテストスクリーン画像を作成
fn create_test_screen(width: u32, height: u32, seed: u64) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);

    // Simple pseudo-random pattern based on position
    for y in 0..height {
        for x in 0..width {
            let val = ((x * 17 + y * 31 + seed as u32) % 256) as u8;
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }

    DynamicImage::ImageRgba8(img)
}

/// Create a template pattern
/// テンプレートパターンを作成
fn create_template(size: u32) -> Vec<u8> {
    let mut img = RgbaImage::new(size, size);

    // Create a distinctive pattern
    for y in 0..size {
        for x in 0..size {
            let val = if (x + y) % 2 == 0 { 200u8 } else { 100u8 };
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }

    let mut buffer = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();

    buffer
}

fn benchmark_ncc_by_screen_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("ncc_screen_size");
    group.sample_size(10);

    let template_data = create_template(50);
    let pattern = Pattern::new(template_data).similar(0.7);
    let matcher = ImageMatcher::new();

    // Different screen sizes
    for (width, height) in [(800, 600), (1920, 1080), (2560, 1440), (3840, 2160)].iter() {
        let screen = create_test_screen(*width, *height, 12345);

        group.bench_with_input(
            BenchmarkId::new("resolution", format!("{}x{}", width, height)),
            &screen,
            |b, screen| {
                b.iter(|| {
                    let _ = matcher.find(black_box(screen), black_box(&pattern));
                })
            },
        );
    }

    group.finish();
}

fn benchmark_ncc_by_template_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("ncc_template_size");
    group.sample_size(10);

    let screen = create_test_screen(1920, 1080, 12345);
    let matcher = ImageMatcher::new();

    // Different template sizes
    for size in [16, 32, 50, 64, 100, 128, 200].iter() {
        let template_data = create_template(*size);
        let pattern = Pattern::new(template_data).similar(0.7);

        group.bench_with_input(
            BenchmarkId::new("template_size", format!("{}x{}", size, size)),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let _ = matcher.find(black_box(&screen), black_box(pattern));
                })
            },
        );
    }

    group.finish();
}

fn benchmark_ncc_by_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("ncc_similarity_threshold");
    group.sample_size(10);

    let screen = create_test_screen(1920, 1080, 12345);
    let template_data = create_template(50);
    let matcher = ImageMatcher::new();

    // Different similarity thresholds
    for similarity in [0.5, 0.7, 0.8, 0.9, 0.95].iter() {
        let pattern = Pattern::new(template_data.clone()).similar(*similarity);

        group.bench_with_input(
            BenchmarkId::new("similarity", format!("{:.2}", similarity)),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let _ = matcher.find(black_box(&screen), black_box(pattern));
                })
            },
        );
    }

    group.finish();
}

fn benchmark_find_vs_find_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_vs_find_all");
    group.sample_size(10);

    // Create screen with multiple matches
    let mut img = RgbaImage::new(1920, 1080);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([128, 128, 128, 255]);
    }

    // Add 3 matching patterns
    for i in 0..3 {
        let x_start = 200 + i * 600;
        for y in 200..250 {
            for x in x_start..(x_start + 50) {
                if x < 1920 && y < 1080 {
                    img.put_pixel(x, y, Rgba([200, 200, 200, 255]));
                }
            }
        }
    }

    let screen = DynamicImage::ImageRgba8(img);
    let template_data = create_template(50);
    let pattern = Pattern::new(template_data).similar(0.7);
    let matcher = ImageMatcher::new();

    group.bench_function("find_first", |b| {
        b.iter(|| {
            let _ = matcher.find(black_box(&screen), black_box(&pattern));
        })
    });

    group.bench_function("find_all", |b| {
        b.iter(|| {
            let _ = matcher.find_all(black_box(&screen), black_box(&pattern));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ncc_by_screen_size,
    benchmark_ncc_by_template_size,
    benchmark_ncc_by_similarity,
    benchmark_find_vs_find_all
);

criterion_main!(benches);
