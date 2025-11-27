# Core-RS Testing Documentation / コアRSテストドキュメント

## Test Coverage Summary / テストカバレッジ概要

This document provides an overview of the testing infrastructure for the Sikuli-D core-rs library.
このドキュメントは、Sikuli-D core-rsライブラリのテストインフラストラクチャの概要を提供します。

## Test Organization / テスト構成

### Unit Tests / ユニットテスト

Unit tests are located inline with the source code using `#[cfg(test)]` modules.
ユニットテストは `#[cfg(test)]` モジュールを使用してソースコードに内包されています。

#### lib.rs Tests

- **Region Tests (47 tests)**: Comprehensive tests for Region struct
  - Basic creation and properties (new, from_corners, center, area)
  - Containment checks (contains, intersects)
  - Geometric operations (offset, expand, intersection)
  - Edge cases (negative expansion, reversed coordinates)

- **Pattern Tests (10 tests)**: Pattern creation and validation
  - Builder pattern with similarity and target_offset
  - Validation (is_valid, data_size)
  - Boundary conditions (clamping similarity to 0.0-1.0)

- **Match Tests (5 tests)**: Match result operations
  - Score comparison (is_good_match)
  - Position queries (center, target)
  - Formatting (score_percent)

- **Color Tests (6 tests)**: Color manipulation
  - Creation (new, rgb)
  - Conversion (to_hex)
  - Equality comparison

- **RawCapture Tests (3 tests)**: Raw image data handling
  - Pixel format support (RGBA, RGB)
  - Stride calculation
  - Default DPI settings

- **SikulixError Tests (3 tests)**: Error type verification
  - Error message formatting
  - Error variant construction

**Total in lib.rs: 74 unit tests**

#### image/matcher.rs Tests

- **ImageMatcher Configuration (6 tests)**: Builder pattern and defaults
- **Overlap Calculation (3 tests)**: IoU (Intersection over Union) tests
- **Change Detection (9 tests)**: Pixel difference calculation
  - Identical images (0% change)
  - Completely different images (100% change)
  - Partial changes
  - Threshold boundary testing (20-pixel threshold)
  - Edge cases (different dimensions, empty images)

**Total in matcher.rs: 18 unit tests**

#### python/mod.rs Tests

- **PythonVersion Tests (2 tests)**: Display and equality
- **Python 2 Detection (9 tests)**:
  - print statement, xrange, raw_input
  - Long literals (123L), basestring, execfile
  - Exception handling (except Exception, e)
- **Python 3 Detection (11 tests)**:
  - f-strings, async/await, walrus operator
  - Type hints, yield from, nonlocal
  - Print function with keywords (end=, sep=, file=)
  - Keyword-only arguments
- **Mixed/Unknown Detection (6 tests)**:
  - Mixed syntax error cases
  - Unknown version (neutral syntax)
  - Empty/comment-only code
- **Edge Cases (5 tests)**:
  - Comments, strings containing syntax
  - Long literals in words vs numbers
- **Validation Tests (5 tests)**:
  - validate() method behavior
  - Error message content
- **Real-World Examples (3 tests)**:
  - Sikuli Python 2/3 scripts
  - Async Sikuli scripts

**Total in python/mod.rs: 41 unit tests**

#### location.rs Tests

- **Basic Operations (13 tests)**:
  - Creation, getters, setters
  - Directional movement (left, right, above, below)
  - Offset calculation
- **Conversions (3 tests)**:
  - Tuple to/from Location
  - Round-trip conversion
- **Trait Implementation (4 tests)**:
  - Copy, Clone, Debug, PartialEq
- **Edge Cases (3 tests)**:
  - Negative coordinates
  - Zero coordinates
  - Method chaining

**Total in location.rs: 23 unit tests**

#### timeout/mod.rs Tests

The timeout module already has comprehensive tests (16 tests) covering:
- DefaultTimeouts builder pattern
- CancellationToken operations
- with_timeout and with_timeout_and_cancel
- wait_for_condition variants
- TimeoutGuard functionality

**Total in timeout/mod.rs: 16 unit tests**

### Integration Tests / 統合テスト

Located in `core-rs/tests/`:
- `test_ocr_api.rs`: OCR functionality tests
- `observer_integration_test.rs`: Observer pattern tests

### Test Fixtures / テストフィクスチャ

Located in `core-rs/tests/fixtures/`:
- `images/`: Sample images for pattern matching tests
- `scripts/`: Python test scripts

## Running Tests / テストの実行

### All Tests / 全テスト

```bash
cd core-rs
cargo test
```

### Specific Module / 特定モジュール

```bash
cargo test --lib lib::tests
cargo test --lib matcher::tests
cargo test --lib python::tests
```

### With Output / 出力付き

```bash
cargo test -- --nocapture
```

### Ignored Tests (Integration) / 無視されたテスト（統合）

```bash
cargo test -- --ignored
```

## Test Categories / テストカテゴリ

### Fast Unit Tests (Default) / 高速ユニットテスト（デフォルト）

Pure logic tests with no I/O or OS interaction:
I/OやOS相互作用のない純粋なロジックテスト：

- All tests in lib.rs, python/mod.rs, location.rs, timeout/mod.rs
- Image matcher algorithm tests (NCC, overlap calculation)
- Change detection algorithm tests

### Integration Tests (Ignored by Default) / 統合テスト（デフォルトで無視）

Tests marked with `#[ignore]` that require:
`#[ignore]` でマークされた以下を必要とするテスト：

- Actual screen capture
- Python runtime
- File system access

Run with: `cargo test -- --ignored`

### Platform-Specific Tests / プラットフォーム固有テスト

Tests marked with `#[cfg(target_os = "...")]`:
- Windows-specific features
- macOS-specific features
- Linux-specific features

## Coverage Goals / カバレッジ目標

### Target Coverage / 目標カバレッジ

- **Overall core-rs**: 85%+
- **lib.rs (basic types)**: 95%+
- **image/matcher.rs (algorithms)**: 90%+
- **python/mod.rs (syntax detection)**: 85%+
- **location.rs (coordinates)**: 90%+
- **timeout/mod.rs (timeout handling)**: 85%+

### Running Coverage / カバレッジの実行

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --html --open

# Generate LCOV for CI/CD
cargo llvm-cov --lcov --output-path lcov.info
```

## Test Design Principles / テスト設計原則

### 1. Logic/I/O Separation / ロジック/I/O分離

✅ **Good**: Pure functions tested with mock data
✅ **良い例**: モックデータでテストされる純粋関数

```rust
fn calculate_overlap(a: &Region, b: &Region) -> f64 {
    // Pure calculation, easy to test
}
```

❌ **Bad**: Functions tightly coupled to OS APIs
❌ **悪い例**: OS APIに密結合した関数

```rust
fn capture_and_find() -> Result<Match> {
    let screen = capture_screen();  // Hard to test
    // ...
}
```

### 2. Comprehensive Coverage / 包括的カバレッジ

Each test should cover:
各テストは以下をカバーすべき：

- **Normal cases**: Expected input and behavior
- **Edge cases**: Boundary values, empty inputs
- **Error cases**: Invalid input, error conditions
- **正常系**: 期待される入力と動作
- **エッジケース**: 境界値、空入力
- **異常系**: 無効な入力、エラー条件

### 3. Clear Test Names / 明確なテスト名

Test names should be descriptive and follow a pattern:
テスト名は説明的で、パターンに従うべき：

```rust
#[test]
fn test_<module>_<function>_<scenario>() {
    // Example: test_region_intersection_no_overlap
}
```

### 4. Bilingual Documentation / 二言語ドキュメント

Tests include English and Japanese comments:
テストには英語と日本語のコメントが含まれます：

```rust
// Test that identical images have 0% change
// 同一画像の変化率が0%であることをテスト
```

## Continuous Integration / 継続的インテグレーション

Tests are automatically run in CI/CD (GitHub Actions) on:
テストはCI/CD（GitHub Actions）で以下の際に自動実行されます：

- Every push to master/develop branches
- Pull requests
- Release tags

See `.github/workflows/ci.yml` for configuration.
設定については `.github/workflows/ci.yml` を参照してください。

## Test Statistics / テスト統計

### Current Coverage / 現在のカバレッジ

- **Total Unit Tests**: 172+
- **Integration Tests**: 10+
- **Test Files**: 6+
- **Lines of Test Code**: 1500+

### Module Breakdown / モジュール別内訳

| Module | Tests | Coverage Target |
|--------|-------|----------------|
| lib.rs | 74 | 95% |
| image/matcher.rs | 18 | 90% |
| python/mod.rs | 41 | 85% |
| location.rs | 23 | 90% |
| timeout/mod.rs | 16 | 85% |

## Contributing Tests / テストへの貢献

When adding new features, please:
新機能を追加する際は、以下を行ってください：

1. Write unit tests for pure logic
   純粋なロジックのユニットテストを書く
2. Add integration tests for I/O operations
   I/O操作の統合テストを追加する
3. Document test purpose with bilingual comments
   テストの目的を二言語コメントで文書化する
4. Ensure tests pass locally before committing
   コミット前にローカルでテストが通ることを確認する
5. Aim for 85%+ coverage for new code
   新しいコードで85%以上のカバレッジを目指す

## Known Limitations / 既知の制限

1. **Platform-specific code**: Some OS-specific functions are difficult to test in unit tests
   **プラットフォーム固有コード**: 一部のOS固有関数はユニットテストでテストが困難

2. **UI operations**: Click, type, and screen capture require integration tests
   **UI操作**: クリック、タイプ、画面キャプチャには統合テストが必要

3. **Python runtime**: Tests requiring actual Python interpreter are marked `#[ignore]`
   **Pythonランタイム**: 実際のPythonインタープリタが必要なテストは `#[ignore]` でマーク

## Future Improvements / 今後の改善

- [ ] Add performance benchmarks using criterion
- [ ] Implement property-based testing with proptest
- [ ] Add mutation testing with cargo-mutants
- [ ] Create visual regression tests for UI components
- [ ] Add fuzzing tests for image processing

---

**Last Updated / 最終更新**: 2025-11-27
**Maintainer / メンテナー**: Claude + User
