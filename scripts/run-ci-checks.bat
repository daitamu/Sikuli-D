@echo off
REM Run CI Checks Locally (Windows) / ローカルでCIチェックを実行 (Windows)
REM
REM This script runs the same checks that CI runs on GitHub Actions
REM このスクリプトはGitHub Actionsで実行されるのと同じチェックを実行します

setlocal enabledelayedexpansion

REM Project directory / プロジェクトディレクトリ
set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "CORE_RS_DIR=%PROJECT_DIR%\core-rs"

echo ========================================
echo Sikuli-D CI Checks / CI チェック
echo ========================================
echo.

REM Check if we're in the right directory / 正しいディレクトリにいるか確認
if not exist "%CORE_RS_DIR%" (
    echo [ERROR] core-rs directory not found
    echo [エラー] core-rsディレクトリが見つかりません
    exit /b 1
)

REM Track failures / 失敗を追跡
set FAILED_CHECKS=0

REM Stage 1: Format Check / フォーマットチェック
echo [1] Format Check / フォーマットチェック
cd /d "%CORE_RS_DIR%"
cargo fmt --all -- --check
if errorlevel 1 (
    echo [FAIL] Format check failed / フォーマットチェック失敗
    echo [TIP] Run 'cargo fmt --all' to auto-fix formatting
    echo [ヒント] 'cargo fmt --all' でフォーマットを自動修正
    set /a FAILED_CHECKS+=1
) else (
    echo [PASS] Format check passed / 成功
)
echo.

REM Stage 2: Clippy / Clippy
echo [2] Clippy Lints / Clippy リント
cd /d "%CORE_RS_DIR%"
cargo clippy --all-targets --all-features -- -D warnings
if errorlevel 1 (
    echo [FAIL] Clippy failed / Clippy失敗
    echo [TIP] Run 'cargo clippy --fix --all-targets --all-features' to auto-fix
    echo [ヒント] 'cargo clippy --fix --all-targets --all-features' で自動修正
    set /a FAILED_CHECKS+=1
) else (
    echo [PASS] Clippy passed / 成功
)
echo.

REM Stage 3: Security Audit / セキュリティ監査
echo [3] Security Audit / セキュリティ監査
where cargo-audit >nul 2>&1
if errorlevel 1 (
    echo [SKIP] cargo-audit not installed, skipping
    echo [スキップ] cargo-audit がインストールされていません
    echo [INFO] Install with: cargo install cargo-audit
) else (
    cd /d "%CORE_RS_DIR%"
    cargo audit
    if errorlevel 1 (
        echo [FAIL] Security audit failed / セキュリティ監査失敗
        echo [TIP] Run 'cargo update' to update vulnerable dependencies
        echo [ヒント] 'cargo update' で脆弱な依存関係を更新
        set /a FAILED_CHECKS+=1
    ) else (
        echo [PASS] Security audit passed / 成功
    )
)
echo.

REM Stage 4: License Check / ライセンスチェック
echo [4] License Check / ライセンスチェック
where cargo-deny >nul 2>&1
if errorlevel 1 (
    echo [SKIP] cargo-deny not installed, skipping
    echo [スキップ] cargo-deny がインストールされていません
    echo [INFO] Install with: cargo install cargo-deny
) else (
    cd /d "%CORE_RS_DIR%"
    cargo deny check
    if errorlevel 1 (
        echo [FAIL] License check failed / ライセンスチェック失敗
        set /a FAILED_CHECKS+=1
    ) else (
        echo [PASS] License check passed / 成功
    )
)
echo.

REM Stage 5: Build / ビルド
echo [5] Build / ビルド
cd /d "%CORE_RS_DIR%"
cargo build --verbose
if errorlevel 1 (
    echo [FAIL] Build failed / ビルド失敗
    set /a FAILED_CHECKS+=1
) else (
    echo [PASS] Build passed / 成功
)
echo.

REM Stage 6: Unit Tests / ユニットテスト
echo [6] Unit Tests / ユニットテスト
cd /d "%CORE_RS_DIR%"
cargo test --lib --bins --verbose
if errorlevel 1 (
    echo [FAIL] Unit tests failed / ユニットテスト失敗
    set /a FAILED_CHECKS+=1
) else (
    echo [PASS] Unit tests passed / 成功
)
echo.

REM Stage 7: Doc Tests / ドキュメントテスト
echo [7] Doc Tests / ドキュメントテスト
cd /d "%CORE_RS_DIR%"
cargo test --doc --verbose
if errorlevel 1 (
    echo [FAIL] Doc tests failed / ドキュメントテスト失敗
    set /a FAILED_CHECKS+=1
) else (
    echo [PASS] Doc tests passed / 成功
)
echo.

REM Stage 8: Integration Tests / 統合テスト
echo [8] Integration Tests / 統合テスト
cd /d "%CORE_RS_DIR%"
cargo test --test * --verbose
if errorlevel 1 (
    echo [FAIL] Integration tests failed / 統合テスト失敗
    set /a FAILED_CHECKS+=1
) else (
    echo [PASS] Integration tests passed / 成功
)
echo.

REM Summary / サマリー
echo ========================================
echo Summary / サマリー
echo ========================================

if !FAILED_CHECKS! equ 0 (
    echo [SUCCESS] All checks passed! / すべてのチェックが成功しました！
    echo           Ready to push / プッシュ可能です
    exit /b 0
) else (
    echo [FAILURE] !FAILED_CHECKS! check(s) failed / !FAILED_CHECKS! 個のチェックが失敗
    echo           Please fix the issues before pushing
    echo           プッシュ前に問題を修正してください
    exit /b 1
)
