# CI/CD Pipeline Guide / CI/CDパイプラインガイド

**Version / バージョン**: 1.0
**Last Updated / 最終更新**: 2025-11-27

---

## Overview / 概要

This document describes the CI/CD pipeline configuration for Sikuli-D project.
本ドキュメントでは、Sikuli-DプロジェクトのCI/CDパイプライン設定について説明します。

### Key Features / 主な機能

- Multi-platform testing (Windows, macOS, Linux) / マルチプラットフォームテスト
- Automated code quality checks / 自動コード品質チェック
- Security vulnerability scanning / セキュリティ脆弱性スキャン
- Code coverage reporting / コードカバレッジレポート
- Performance benchmarking / パフォーマンスベンチマーク
- Automated dependency updates / 自動依存関係更新
- Automated release builds / 自動リリースビルド

---

## CI Pipeline Stages / CIパイプラインステージ

### Stage 1: Code Quality Checks / コード品質チェック

**Runs on**: Ubuntu Latest
**Duration**: ~2-3 minutes

**Checks performed / 実行されるチェック**:

1. **Format Check** - `cargo fmt --check`
   - Ensures code follows Rust formatting standards
   - コードがRustフォーマット基準に準拠していることを確認

2. **Clippy** - `cargo clippy -- -D warnings`
   - Static analysis for common mistakes
   - 一般的なミスの静的解析

3. **Security Audit** - `cargo audit`
   - Scans for known security vulnerabilities in dependencies
   - 依存関係の既知のセキュリティ脆弱性をスキャン

4. **License Check** - `cargo deny check`
   - Verifies all dependencies have acceptable licenses
   - すべての依存関係が許容可能なライセンスを持つことを確認

**Required for merge**: Yes / マージ必須

---

### Stage 2: Unit Tests / ユニットテスト

**Runs on**: Windows, macOS, Linux (matrix)
**Duration**: ~5-7 minutes per platform

**Tests executed / 実行されるテスト**:

```bash
# Library and binary tests
cargo test --lib --bins --verbose

# Documentation tests
cargo test --doc --verbose
```

**Required for merge**: Yes / マージ必須

---

### Stage 3: Integration Tests / 統合テスト

**Runs on**: Windows, macOS, Linux (matrix)
**Duration**: ~8-10 minutes per platform

**Tests executed / 実行されるテスト**:

```bash
# All integration tests
cargo test --test '*' --verbose
```

**Required for merge**: Yes / マージ必須

---

### Stage 4: Code Coverage / コードカバレッジ

**Runs on**: Ubuntu Latest
**Duration**: ~3-4 minutes

**Coverage generation / カバレッジ生成**:

```bash
# Generate LCOV report for Codecov
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML report for artifact
cargo llvm-cov report --html --output-dir coverage-report
```

**Target**: 85% coverage for core-rs

**Artifacts**:
- Coverage report uploaded to Codecov
- HTML report available as GitHub Actions artifact

**Required for merge**: Yes / マージ必須

---

### Stage 5: Performance Benchmarks / パフォーマンスベンチマーク

**Runs on**: Ubuntu Latest (push to main branches only)
**Duration**: ~10-15 minutes

**Benchmarks executed / 実行されるベンチマーク**:

```bash
cargo bench --no-fail-fast
```

**Artifacts**:
- Criterion benchmark results uploaded as artifact

**Required for merge**: No (informational only) / いいえ（情報提供のみ）

---

### Stage 6: Build Artifacts / ビルド成果物

**Runs on**: Windows, macOS, Linux (matrix)
**Duration**: ~10-12 minutes per platform

**Builds / ビルド**:

1. **core-rs** (Release mode)
   ```bash
   cargo build --release --verbose
   ```

2. **ide-rs-tauri** (Debug mode)
   ```bash
   cargo build --verbose
   ```

**Artifacts**:
- Compiled libraries (.so, .dll, .dylib, .rlib)
- Retained for 7 days

**Required for merge**: Yes / マージ必須

---

## Release Pipeline / リリースパイプライン

### Triggering a Release / リリースのトリガー

Release pipeline is triggered when a version tag is pushed:
バージョンタグがプッシュされるとリリースパイプラインがトリガーされます：

```bash
git tag v1.0.0
git push origin v1.0.0
```

### Release Workflow Steps / リリースワークフローの手順

1. **Create GitHub Release**
   - Extract version from tag
   - Generate changelog from commits since last tag
   - Create release with changelog

2. **Build Release Artifacts**
   - Build for multiple platforms and architectures:
     - Windows x64
     - macOS x64
     - macOS ARM64
     - Linux x64
   - Create archives (.zip for Windows, .tar.gz for Unix)
   - Upload to GitHub Release

3. **Build IDE Application**
   - Build Tauri application for all platforms
   - Upload as artifacts (when ide-rs-tauri is ready)

4. **Post-Release Actions**
   - Create release summary
   - Provide next steps checklist

### Pre-release Versions / プレリリースバージョン

Tags containing `alpha`, `beta`, or `rc` are marked as pre-releases:
`alpha`、`beta`、または`rc`を含むタグはプレリリースとしてマークされます：

```bash
git tag v1.0.0-beta.1
git push origin v1.0.0-beta.1
```

---

## Configuration Files / 設定ファイル

### rustfmt.toml

Location: Project root / プロジェクトルート

**Key settings / 主要設定**:
- Edition: 2021
- Max width: 100 characters
- Tab spaces: 4
- Import organization enabled
- Comment wrapping enabled

**Usage / 使用法**:

```bash
# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

---

### clippy.toml

Location: Project root / プロジェクトルート

**Key settings / 主要設定**:
- Cognitive complexity threshold: 30
- Function arguments threshold: 7
- Type complexity threshold: 250

**Usage / 使用法**:

```bash
# Run clippy
cargo clippy --all-targets --all-features

# Deny warnings (CI mode)
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix issues
cargo clippy --fix --all-targets --all-features
```

---

### deny.toml

Location: Project root / プロジェクトルート

**Key settings / 主要設定**:
- Deny security vulnerabilities
- Warn about unmaintained crates
- Allow MIT, Apache-2.0, BSD licenses
- Deny GPL-3.0, AGPL-3.0
- Warn about multiple versions

**Usage / 使用法**:

```bash
# Install cargo-deny
cargo install cargo-deny

# Run all checks
cargo deny check

# Check specific category
cargo deny check licenses
cargo deny check advisories
cargo deny check bans
```

---

### codecov.yml

Location: Project root / プロジェクトルート

**Coverage targets / カバレッジ目標**:
- Project overall: 85%
- Patch (PR changes): 80%

**Ignored paths / 無視されるパス**:
- Benchmarks
- Examples
- Tests
- Build scripts

---

### .github/dependabot.yml

Location: `.github/` directory

**Update schedule / 更新スケジュール**:
- **Day**: Monday / 月曜日
- **Time**: 09:00 JST
- **Frequency**: Weekly / 週次

**Managed dependencies / 管理される依存関係**:
- Cargo dependencies (core-rs)
- Cargo dependencies (ide-rs-tauri)
- GitHub Actions

**Settings / 設定**:
- Max open PRs: 5 (Cargo), 3 (Actions)
- Auto-assigned to: daitamu
- Labels: dependencies, rust/github-actions

---

## Running CI Checks Locally / ローカルでのCIチェック実行

### Prerequisites / 前提条件

```bash
# Install Rust toolchain
rustup update stable
rustup component add clippy rustfmt

# Install additional tools
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-llvm-cov
```

### Run All Checks / すべてのチェックを実行

```bash
#!/bin/bash
# run-ci-checks.sh

set -e

echo "=== Format Check ==="
cargo fmt --all -- --check

echo "=== Clippy ==="
cargo clippy --all-targets --all-features -- -D warnings

echo "=== Security Audit ==="
cargo audit

echo "=== License Check ==="
cargo deny check

echo "=== Unit Tests ==="
cargo test --lib --bins --verbose

echo "=== Doc Tests ==="
cargo test --doc --verbose

echo "=== Integration Tests ==="
cargo test --test '*' --verbose

echo "=== Coverage ==="
cargo llvm-cov --all-features --workspace --html

echo "✅ All CI checks passed!"
```

### Quick Pre-Commit Check / クイック・プレコミットチェック

```bash
# Format, lint, and test
cargo fmt --all && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --verbose
```

---

## GitHub Actions Secrets / GitHubアクションシークレット

Currently, no secrets are required for the CI/CD pipeline.
The following may be needed in the future:
現在、CI/CDパイプラインにシークレットは必要ありません。
将来的に以下が必要になる可能性があります：

- `CODECOV_TOKEN` - For private repositories (currently using GitHub token)
- `CARGO_REGISTRY_TOKEN` - For publishing to crates.io

---

## Troubleshooting / トラブルシューティング

### CI Failures / CI失敗

#### Format Check Failed / フォーマットチェック失敗

```bash
# Fix locally
cargo fmt --all

# Commit the changes
git add .
git commit -m "Fix formatting"
git push
```

#### Clippy Warnings / Clippy警告

```bash
# Auto-fix where possible
cargo clippy --fix --all-targets --all-features

# Review remaining warnings
cargo clippy --all-targets --all-features
```

#### Security Vulnerabilities / セキュリティ脆弱性

```bash
# Check details
cargo audit

# Update vulnerable dependencies
cargo update

# If updates don't fix it, check for patches or alternatives
```

#### License Issues / ライセンス問題

```bash
# Check which dependency has the issue
cargo deny check licenses

# Review the dependency tree
cargo tree | grep <problematic-crate>

# Consider alternatives or add exception in deny.toml
```

#### Test Failures / テスト失敗

```bash
# Run specific test
cargo test test_name -- --nocapture

# Run tests with full output
cargo test -- --nocapture --test-threads=1

# Check platform-specific issues
cargo test --target <platform-target>
```

#### Coverage Below Target / カバレッジが目標未達

1. Check coverage report artifact in GitHub Actions
2. Identify uncovered code sections
3. Add missing tests
4. Consider if some code should be excluded

---

## Best Practices / ベストプラクティス

### Before Pushing / プッシュ前

1. Run local checks: `cargo fmt && cargo clippy && cargo test`
2. Check for security issues: `cargo audit`
3. Ensure tests pass on your platform
4. Review your changes: `git diff`

### Pull Request Guidelines / プルリクエストガイドライン

1. Wait for all CI checks to pass before requesting review
2. Address any clippy warnings or format issues
3. Add tests for new functionality
4. Update documentation if needed
5. Keep PRs focused and reasonably sized

### Release Process / リリースプロセス

1. Update VERSION file if needed (MAJOR.MINOR only)
2. Ensure all tests pass on main branch
3. Create and push version tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. Monitor release workflow in GitHub Actions
5. Verify release artifacts are uploaded correctly
6. Update release notes if needed
7. Announce the release

---

## Monitoring and Maintenance / モニタリングとメンテナンス

### Regular Tasks / 定期タスク

**Weekly / 週次**:
- Review Dependabot PRs
- Check for security advisories
- Monitor CI performance

**Monthly / 月次**:
- Review coverage trends
- Update CI/CD configurations if needed
- Check for deprecated GitHub Actions

**Quarterly / 四半期**:
- Audit all dependencies
- Review and update deny.toml rules
- Evaluate CI/CD pipeline efficiency

---

## Future Enhancements / 今後の強化

### Planned Improvements / 計画された改善

1. **Mutation Testing**
   - Use cargo-mutants to verify test effectiveness
   - テストの有効性を検証するためにcargo-mutantsを使用

2. **Visual Regression Testing**
   - Screenshot comparison for IDE
   - IDE用スクリーンショット比較

3. **Performance Tracking**
   - Track benchmark results over time
   - ベンチマーク結果を経時的に追跡

4. **Fuzzing**
   - Add cargo-fuzz for image processing
   - 画像処理にcargo-fuzzを追加

5. **Multi-architecture Builds**
   - Add ARM64 Linux support
   - ARM64 Linuxサポートを追加

---

## Support / サポート

For issues with CI/CD pipeline:
CI/CDパイプラインの問題については：

1. Check this documentation
2. Review GitHub Actions logs
3. Check [TEST-CICD-DESIGN.md](../.local/doc/spec/TEST-CICD-DESIGN.md)
4. Open an issue on GitHub

---

**Document Version / ドキュメントバージョン**: 1.0
**Last Updated / 最終更新**: 2025-11-27
