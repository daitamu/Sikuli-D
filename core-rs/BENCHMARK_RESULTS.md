# Benchmark Results / ベンチマーク結果

**Last Updated / 最終更新**: 2025-11-27
**Version / バージョン**: 0.1.0

---

## Test Environment / テスト環境

### Hardware / ハードウェア
- **CPU**: [To be filled / 記入予定]
- **Memory**: [To be filled / 記入予定]
- **OS**: [To be filled / 記入予定]

### Software / ソフトウェア
- **Rust Version**: [To be filled / 記入予定]
- **Build Profile**: Release with LTO
- **Optimization Level**: 3

---

## Benchmark Results / ベンチマーク結果

### 1. Image Matching (matching.rs)

#### find() - Single Match / 単一マッチの検索

| Screen Size | Template Size | Time (avg) | Std Dev | Status |
|-------------|---------------|------------|---------|--------|
| 800x450 | 50x50 | TBD | TBD | ⏳ |
| 1920x1080 | 50x50 | TBD | TBD | ⏳ |
| 3840x2160 | 50x50 | TBD | TBD | ⏳ |

**Target**: < 100ms for 1920x1080 with 50x50 template
**目標**: 1920x1080、50x50テンプレートで < 100ms

#### find_all() - Multiple Matches / 複数マッチの検索

| Screen Size | Matches | Time (avg) | Std Dev | Status |
|-------------|---------|------------|---------|--------|
| 1920x1080 | ~15 | TBD | TBD | ⏳ |

**Target**: < 300ms for typical use case
**目標**: 典型的な使用例で < 300ms

#### NCC Calculation by Template Size / テンプレートサイズ別NCC計算

| Template Size | Time (avg) | Std Dev | Throughput |
|---------------|------------|---------|------------|
| 32x32 | TBD | TBD | TBD ops/sec |
| 64x64 | TBD | TBD | TBD ops/sec |
| 128x128 | TBD | TBD | TBD ops/sec |

---

### 2. Screen Capture (screen_capture.rs)

#### Full Screen Capture / 全画面キャプチャ

| Operation | Time (avg) | Std Dev | Status |
|-----------|------------|---------|--------|
| Primary Screen | TBD | TBD | ⏳ |

**Target**: < 50ms for 1920x1080
**目標**: 1920x1080で < 50ms

#### Region Capture / 領域キャプチャ

| Region Size | Time (avg) | Std Dev | Status |
|-------------|------------|---------|--------|
| 100x100 | TBD | TBD | ⏳ |
| 500x500 | TBD | TBD | ⏳ |
| 1000x1000 | TBD | TBD | ⏳ |

**Target**: < 10ms for 500x500
**目標**: 500x500で < 10ms

---

### 3. NCC Calculation (ncc_calculation.rs)

#### Performance by Screen Resolution / 画面解像度別パフォーマンス

| Resolution | Template | Time (avg) | Std Dev | Positions/sec |
|------------|----------|------------|---------|---------------|
| 800x600 | 50x50 | TBD | TBD | TBD |
| 1920x1080 | 50x50 | TBD | TBD | TBD |
| 2560x1440 | 50x50 | TBD | TBD | TBD |
| 3840x2160 | 50x50 | TBD | TBD | TBD |

#### Performance by Template Size / テンプレートサイズ別パフォーマンス

| Template Size | Time (avg) | Std Dev | Speedup vs. Baseline |
|---------------|------------|---------|----------------------|
| 16x16 | TBD | TBD | 1.0x |
| 32x32 | TBD | TBD | TBD |
| 50x50 | TBD | TBD | TBD |
| 64x64 | TBD | TBD | TBD |
| 100x100 | TBD | TBD | TBD |
| 128x128 | TBD | TBD | TBD |
| 200x200 | TBD | TBD | TBD |

#### Impact of Similarity Threshold / 類似度閾値の影響

| Similarity | Time (avg) | Std Dev | Notes |
|------------|------------|---------|-------|
| 0.50 | TBD | TBD | - |
| 0.70 | TBD | TBD | Default |
| 0.80 | TBD | TBD | - |
| 0.90 | TBD | TBD | - |
| 0.95 | TBD | TBD | - |

**Note / 注記**: Similarity threshold should not significantly affect find() performance, but may affect find_all() due to number of matches found.

類似度閾値は find() のパフォーマンスに大きく影響しませんが、見つかるマッチ数により find_all() に影響する可能性があります。

---

## Performance Trends / パフォーマンス傾向

### Optimization Impact / 最適化の影響

| Optimization | Baseline | After Optimization | Speedup | Status |
|--------------|----------|-------------------|---------|--------|
| NCC unsafe pixel access | TBD | TBD | TBD | ✅ Implemented |
| NMS memory optimization | TBD | TBD | TBD | ✅ Implemented |
| Overlap early exit | TBD | TBD | TBD | ✅ Implemented |
| SIMD vectorization | - | - | - | ⏳ Future |

---

## Comparison with Previous Versions / 以前のバージョンとの比較

### Version Comparison / バージョン比較

| Version | find() 1920x1080 | find_all() | Screen Capture | Notes |
|---------|------------------|------------|----------------|-------|
| 0.1.0 (baseline) | TBD | TBD | TBD | Current version |
| 0.2.0 (planned) | - | - | - | With SIMD |

---

## How to Reproduce / 再現方法

### Running Benchmarks / ベンチマークの実行

```bash
# Ensure you're in release mode
cd core-rs

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench ncc_calculation

# Save results to file
cargo bench > ../BENCHMARK_RESULTS_RAW.txt 2>&1
```

### Updating This Document / このドキュメントの更新

1. Run benchmarks on your system
2. Extract relevant metrics from Criterion output
3. Update the tables above with actual values
4. Add system information
5. Commit changes with benchmark results

**手順**:
1. システムでベンチマークを実行
2. Criterion 出力から関連メトリクスを抽出
3. 上記の表を実際の値で更新
4. システム情報を追加
5. ベンチマーク結果と共に変更をコミット

---

## Performance Issues / パフォーマンスの問題

### Known Issues / 既知の問題

1. **Large template performance**: Templates > 200x200 may be slow
   - **大きなテンプレートのパフォーマンス**: 200x200以上のテンプレートは遅い場合がある
   - Mitigation: Consider image pyramid approach
   - 対策: 画像ピラミッドアプローチの検討

2. **Multi-monitor support**: Screen capture only supports primary monitor
   - **マルチモニタサポート**: 画面キャプチャはプライマリモニタのみサポート
   - Status: Planned for future release
   - 状態: 将来のリリースで計画中

### Reporting Performance Issues / パフォーマンス問題の報告

If you encounter performance issues:
パフォーマンス問題が発生した場合:

1. Run benchmarks on your system
2. Include system specifications
3. Provide reproduction steps
4. Submit issue with benchmark data

1. システムでベンチマークを実行
2. システム仕様を含める
3. 再現手順を提供
4. ベンチマークデータと共に Issue を提出

---

## Legend / 凡例

- ✅ Implemented / 実装済み
- ⏳ Pending / 保留中
- ❌ Not Met / 未達成
- 🎯 Target Met / 目標達成
- TBD: To Be Determined / 測定予定

---

**Note / 注記**: Benchmark results will vary based on hardware, OS, and system load. Results shown are representative of typical performance.

ベンチマーク結果はハードウェア、OS、システム負荷により異なります。表示されている結果は典型的なパフォーマンスを表しています。
