# Performance Optimization Guide
# パフォーマンス最適化ガイド

**Version**: 1.0.0
**Date**: 2025-11-27

---

## Overview / 概要

This document describes the performance optimizations implemented in sikulix-core and provides guidelines for benchmarking and further optimization.

本ドキュメントは、sikulix-core に実装されたパフォーマンス最適化について説明し、ベンチマークとさらなる最適化のためのガイドラインを提供します。

---

## Performance Targets / パフォーマンス目標

| Operation | Condition | Target | Optimized Target |
|-----------|-----------|--------|------------------|
| **Screen Capture** | | | |
| Full screen capture | 1920x1080 | < 50ms | < 30ms |
| Region capture | 500x500 | < 10ms | < 5ms |
| **Image Matching (NCC)** | | | |
| NCC calculation | 1920x1080 screen, 50x50 template | < 100ms | < 50ms |
| find() | 1920x1080 screen, 50x50 template | < 150ms | < 80ms |
| find_all() | 1920x1080 screen, multiple matches | < 300ms | < 150ms |
| **Non-Maximum Suppression** | | | |
| NMS | 10 matches | < 1ms | < 0.5ms |
| NMS | 100 matches | < 10ms | < 5ms |
| NMS | 1000 matches | < 500ms | < 100ms |

---

## Optimizations Implemented / 実装された最適化

### 1. NCC Calculation Optimization / NCC計算の最適化

#### 1.1 Memory Access Patterns / メモリアクセスパターン

**Optimization / 最適化**:
- Row-major access pattern for better cache locality
- Use of `unsafe_get_pixel()` for bounds-checked fast access
- Pre-computed template statistics to avoid redundant calculations

**実装**:
- キャッシュ局所性向上のための行優先アクセスパターン
- 境界チェック済み高速アクセスのための `unsafe_get_pixel()` の使用
- 冗長な計算を避けるためのテンプレート統計の事前計算

**Code Location / コードの場所**:
`core-rs/src/image/matcher.rs` - `calculate_ncc_with_stats()` function

```rust
// Optimized NCC calculation with bounds checking
for ty in 0..th {
    let screen_row_offset = offset_y + ty;

    for tx in 0..tw {
        // SAFETY: Bounds are checked at function entry
        let s = unsafe {
            screen.unsafe_get_pixel(offset_x + tx, screen_row_offset)[0] as f64
        };
        let t = unsafe {
            template.gray.unsafe_get_pixel(tx, ty)[0] as f64
        };

        sum_st += s * t;
        sum_s2 += s * s;
    }
}
```

#### 1.2 Pre-computed Template Statistics / テンプレート統計の事前計算

**Optimization / 最適化**:
- Compute `sum_t2` (Σ(t²)) once per template instead of per position
- Store grayscale conversion result to avoid repeated conversions

**実装**:
- 位置ごとではなくテンプレートごとに `sum_t2` (Σ(t²)) を一度だけ計算
- 繰り返し変換を避けるためグレースケール変換結果を保存

**Memory Trade-off / メモリトレードオフ**:
- Additional memory: `(width × height) + 24` bytes per template
- Performance gain: Eliminates `sqrt(sum_t2)` calculation at each position
- For 1920×1080 search with 100×100 template: ~1.7M positions saved

---

### 2. Parallel Processing / 並列処理

#### 2.1 Row-level Parallelization / 行レベルの並列化

**Implementation / 実装**:
- Uses Rayon for parallel processing of screen rows
- Each thread processes one row independently
- Optimal granularity to balance thread overhead vs. parallelism

**実装詳細**:
- 画面行の並列処理に Rayon を使用
- 各スレッドが独立して1行を処理
- スレッドオーバーヘッドと並列性のバランスを取る最適な粒度

**Code Location / コードの場所**:
`core-rs/src/image/matcher.rs` - `find()` and `find_all()` methods

```rust
let results: Vec<(f64, u32, u32)> = (0..search_height)
    .into_par_iter()  // Parallel iteration over rows
    .map(|y| {
        // Process each row independently
        // ...
    })
    .collect();
```

**Performance Impact / パフォーマンス影響**:
- Near-linear speedup on multi-core CPUs
- Typical speedup: 3-6x on 4-8 core systems

---

### 3. Non-Maximum Suppression Optimization / 非最大値抑制の最適化

#### 3.1 Memory Allocation Reduction / メモリ割り当ての削減

**Optimizations / 最適化**:
1. Use `sort_unstable_by()` instead of `sort_by()` for faster sorting
2. Move semantics to avoid cloning Match objects
3. Pre-allocate result vector with estimated capacity
4. Early return for small inputs (≤ 1 match)

**実装**:
1. より高速なソートのため `sort_by()` ではなく `sort_unstable_by()` を使用
2. Match オブジェクトのクローンを避けるための移動セマンティクス
3. 推定容量での結果ベクトルの事前割り当て
4. 小さい入力（≤ 1マッチ）の場合は早期リターン

**Code Location / コードの場所**:
`core-rs/src/image/matcher.rs` - `non_maximum_suppression()` method

#### 3.2 Overlap Calculation Optimization / 重複計算の最適化

**Optimizations / 最適化**:
1. Early exit for non-overlapping regions
2. Inline function for reduced call overhead
3. Integer arithmetic where possible

**実装**:
1. 重ならない領域の早期終了
2. 呼び出しオーバーヘッド削減のためのインライン関数
3. 可能な限り整数演算を使用

**Code Location / コードの場所**:
`core-rs/src/image/matcher.rs` - `calculate_overlap_fast()` function

```rust
#[inline(always)]
fn calculate_overlap_fast(a: &Region, b: &Region) -> f64 {
    // Early exit if regions don't overlap at all
    if a.x + a.width as i32 <= b.x || b.x + b.width as i32 <= a.x
        || a.y + a.height as i32 <= b.y || b.y + b.height as i32 <= a.y {
        return 0.0;
    }

    // ... IoU calculation ...
}
```

---

## Compiler Optimizations / コンパイラ最適化

### Release Profile / リリースプロファイル

**Configuration / 設定** (`Cargo.toml`):

```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower compile
strip = true            # Remove debug symbols
opt-level = 3           # Maximum optimization

[profile.bench]
inherits = "release"
debug = true            # Keep debug info for profiling
strip = false           # Keep symbols for benchmark analysis
```

**Benefits / 効果**:
- LTO enables cross-crate optimizations
- Single codegen unit allows better inlining
- opt-level 3 enables aggressive optimizations including auto-vectorization

**効果**:
- LTO はクレート間最適化を有効化
- 単一のコード生成ユニットでより良いインライン化が可能
- opt-level 3 は自動ベクトル化を含む積極的な最適化を有効化

---

## Benchmarking / ベンチマーク

### Running Benchmarks / ベンチマークの実行

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench matching
cargo bench --bench screen_capture
cargo bench --bench ncc_calculation

# Run and save results
cargo bench > benchmark_results.txt
```

### Benchmark Suites / ベンチマークスイート

#### 1. Image Matching Benchmarks / 画像マッチングベンチマーク
**File / ファイル**: `benches/matching.rs`

**Tests / テスト**:
- `benchmark_find` - find() performance with different screen sizes
- `benchmark_find_all` - find_all() with multiple matches
- `benchmark_ncc_calculation` - NCC calculation with different template sizes

#### 2. Screen Capture Benchmarks / 画面キャプチャベンチマーク
**File / ファイル**: `benches/screen_capture.rs`

**Tests / テスト**:
- `benchmark_screen_capture_full` - Full screen capture performance
- `benchmark_screen_capture_region` - Region capture with various sizes
- `benchmark_screen_dimensions` - Screen dimension query performance

#### 3. NCC Calculation Benchmarks / NCC計算ベンチマーク
**File / ファイル**: `benches/ncc_calculation.rs`

**Tests / テスト**:
- `benchmark_ncc_by_screen_size` - NCC performance across resolutions
- `benchmark_ncc_by_template_size` - NCC performance with different template sizes
- `benchmark_ncc_by_similarity` - Impact of similarity threshold
- `benchmark_find_vs_find_all` - Comparison of find() vs find_all()

---

## Profiling / プロファイリング

### Using perf (Linux) / perf の使用 (Linux)

```bash
# Record performance data
cargo build --release
perf record --call-graph dwarf ./target/release/your_binary

# Analyze results
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### Using Instruments (macOS) / Instruments の使用 (macOS)

```bash
# Build with debug symbols
cargo build --release

# Launch with Instruments
instruments -t "Time Profiler" ./target/release/your_binary
```

### Using Visual Studio Profiler (Windows)

```bash
# Build with debug symbols in release mode
cargo build --release

# Open in Visual Studio and use Performance Profiler
```

---

## Future Optimization Opportunities / 今後の最適化機会

### 1. SIMD Optimization / SIMD最適化

**Status / 状態**: Planned for Ver.2
**計画**: Ver.2 で実装予定

**Approach / アプローチ**:
- Use `std::arch` or `packed_simd` for explicit SIMD
- Target AVX2 on x86_64 for 8x f32 parallel operations
- Fallback to scalar implementation for other architectures

**Expected Benefit / 期待される効果**:
- 4-8x speedup for NCC calculation
- Requires careful implementation and testing

### 2. Image Pyramid / 画像ピラミッド

**Status / 状態**: Research phase
**状態**: 調査段階

**Approach / アプローチ**:
- Perform coarse search at lower resolution
- Refine matches at full resolution
- Trade accuracy for speed

**Expected Benefit / 期待される効果**:
- 2-4x speedup for large templates
- May miss small matches

### 3. Spatial Indexing for NMS / NMSの空間インデックス

**Status / 状態**: Planned for Ver.2
**計画**: Ver.2 で実装予定

**Approach / アプローチ**:
- Use grid-based spatial partitioning
- Only compare matches in neighboring cells
- Reduces O(n²) to O(n×k) where k = avg matches per cell

**Expected Benefit / 期待される効果**:
- Significant speedup for > 1000 matches
- Negligible overhead for small match sets

### 4. Template Caching / テンプレートキャッシング

**Status / 状態**: Design phase
**状態**: 設計段階

**Approach / アプローチ**:
- Cache template statistics by image hash
- Reuse across multiple find operations
- LRU eviction policy

**Expected Benefit / 期待される効果**:
- Eliminates repeated template preprocessing
- Memory vs. speed trade-off

---

## Performance Measurement Guidelines / パフォーマンス測定ガイドライン

### 1. Consistent Environment / 一貫した環境

- Close background applications
- Use same hardware for comparisons
- Run benchmarks multiple times for statistical significance

### 2. Baseline Establishment / ベースラインの確立

- Record baseline performance before optimization
- Use representative workloads
- Document system specifications

### 3. Incremental Validation / 段階的な検証

- Measure impact of each optimization independently
- Verify correctness after each change
- Roll back if performance degrades

---

## Troubleshooting / トラブルシューティング

### Slow Performance / パフォーマンスが遅い

**Check / 確認事項**:
1. Are you running in release mode? (`cargo build --release`)
2. Is LTO enabled in Cargo.toml?
3. Are debug assertions disabled?
4. Is the CPU governor set to performance mode? (Linux)

### Memory Usage / メモリ使用量

**Monitor / モニタリング**:
```bash
# Linux
/usr/bin/time -v ./your_binary

# Use valgrind for detailed analysis
valgrind --tool=massif ./your_binary
```

**Optimization Tips / 最適化のヒント**:
- Reuse image buffers when possible
- Avoid unnecessary clones
- Use `Vec::with_capacity()` to pre-allocate

---

## References / 参考資料

1. **Rust Performance Book**: https://nnethercote.github.io/perf-book/
2. **Rayon Documentation**: https://docs.rs/rayon/
3. **Criterion.rs Guide**: https://bheisler.github.io/criterion.rs/book/
4. **L3-IMPL-DESIGN.md**: Internal implementation design document
5. **Image Matching Algorithms**:
   - Lewis, J.P. "Fast Template Matching" (1995)
   - Normalized Cross-Correlation (NCC) method

---

## Changelog / 変更履歴

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-27 | Initial performance optimization documentation |

---

**Note / 注記**: This is a living document. Update it as new optimizations are implemented or performance characteristics change.

このドキュメントは継続的に更新されます。新しい最適化が実装されたり、パフォーマンス特性が変化した場合は更新してください。
