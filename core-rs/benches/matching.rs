//! Benchmarks for image matching performance
//! 画像マッチングのパフォーマンスベンチマーク

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use image::{DynamicImage, ImageEncoder, Rgba, RgbaImage};
use sikulid::{ImageMatcher, Pattern};
use std::time::Duration;

/// Create a test image with a known pattern
/// 既知のパターンを持つテスト画像を作成
fn create_test_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);

    // Fill with gray background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([128, 128, 128, 255]);
    }

    // Add a recognizable pattern (white rectangle)
    for y in 100..150 {
        for x in 100..200 {
            if x < width && y < height {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }

    DynamicImage::ImageRgba8(img)
}

/// Create a template from the test image
/// テスト画像からテンプレートを作成
fn create_template() -> Vec<u8> {
    let mut img = RgbaImage::new(100, 50);

    // White rectangle template
    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 255, 255, 255]);
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

fn benchmark_find(c: &mut Criterion) {
    let mut group = c.benchmark_group("find");
    // Configure for faster benchmarking
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.sampling_mode(SamplingMode::Flat);

    // Test different screen sizes
    for size in [800, 1920, 3840].iter() {
        let screen = create_test_image(*size, *size * 9 / 16);
        let template_data = create_template();
        let pattern = Pattern::new(template_data).similar(0.7);
        let matcher = ImageMatcher::new();

        group.bench_with_input(
            BenchmarkId::new("screen_size", format!("{}x{}", size, size * 9 / 16)),
            &(&screen, &pattern),
            |b, (screen, pattern)| b.iter(|| matcher.find(black_box(*screen), black_box(*pattern))),
        );
    }

    group.finish();
}

fn benchmark_find_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_all");
    // Configure for faster benchmarking
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.sampling_mode(SamplingMode::Flat);

    // Create image with multiple matches
    let mut img = RgbaImage::new(1920, 1080);

    // Fill with gray background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([128, 128, 128, 255]);
    }

    // Add multiple white rectangles
    for i in 0..5 {
        for j in 0..3 {
            let x_start = 100 + i * 300;
            let y_start = 100 + j * 300;
            for y in y_start..(y_start + 50) {
                for x in x_start..(x_start + 100) {
                    if x < 1920 && y < 1080 {
                        img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
    }

    let screen = DynamicImage::ImageRgba8(img);
    let template_data = create_template();
    let pattern = Pattern::new(template_data).similar(0.7);
    let matcher = ImageMatcher::new();

    group.bench_function("multiple_matches", |b| {
        b.iter(|| matcher.find_all(black_box(&screen), black_box(&pattern)))
    });

    group.finish();
}

fn benchmark_ncc_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ncc");
    // Configure for faster benchmarking
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.sampling_mode(SamplingMode::Flat);

    // Test different template sizes
    for template_size in [32, 64, 128].iter() {
        let screen = create_test_image(1920, 1080);

        let mut template_img = RgbaImage::new(*template_size, *template_size);
        for pixel in template_img.pixels_mut() {
            *pixel = Rgba([200, 200, 200, 255]);
        }

        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder
            .write_image(
                template_img.as_raw(),
                template_img.width(),
                template_img.height(),
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();

        let pattern = Pattern::new(buffer).similar(0.7);
        let matcher = ImageMatcher::new();

        group.bench_with_input(
            BenchmarkId::new("template_size", template_size),
            &(&screen, &pattern),
            |b, (screen, pattern)| b.iter(|| matcher.find(black_box(*screen), black_box(*pattern))),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_find,
    benchmark_find_all,
    benchmark_ncc_calculation
);

criterion_main!(benches);
