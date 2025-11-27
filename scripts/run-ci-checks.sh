#!/bin/bash
# Run CI Checks Locally / ローカルでCIチェックを実行
#
# This script runs the same checks that CI runs on GitHub Actions
# このスクリプトはGitHub Actionsで実行されるのと同じチェックを実行します

set -e

# Colors for output / 出力用の色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project directory / プロジェクトディレクトリ
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CORE_RS_DIR="$PROJECT_DIR/core-rs"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Sikuli-D CI Checks / CI チェック${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if we're in the right directory / 正しいディレクトリにいるか確認
if [ ! -d "$CORE_RS_DIR" ]; then
    echo -e "${RED}❌ Error: core-rs directory not found${NC}"
    echo -e "${RED}   エラー: core-rsディレクトリが見つかりません${NC}"
    exit 1
fi

# Function to run a check / チェックを実行する関数
run_check() {
    local name="$1"
    local command="$2"
    local dir="${3:-$CORE_RS_DIR}"

    echo -e "${YELLOW}▶ $name${NC}"
    echo -e "${YELLOW}  $name${NC}"

    if (cd "$dir" && eval "$command"); then
        echo -e "${GREEN}✅ $name passed${NC}"
        echo -e "${GREEN}   成功${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}❌ $name failed${NC}"
        echo -e "${RED}   失敗${NC}"
        echo ""
        return 1
    fi
}

# Track failures / 失敗を追跡
FAILED_CHECKS=0

# Stage 1: Format Check / フォーマットチェック
if ! run_check "Format Check / フォーマットチェック" "cargo fmt --all -- --check"; then
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
    echo -e "${YELLOW}💡 Tip: Run 'cargo fmt --all' to auto-fix formatting${NC}"
    echo -e "${YELLOW}   ヒント: 'cargo fmt --all' でフォーマットを自動修正${NC}"
    echo ""
fi

# Stage 2: Clippy / Clippy
if ! run_check "Clippy Lints / Clippy リント" "cargo clippy --all-targets --all-features -- -D warnings"; then
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
    echo -e "${YELLOW}💡 Tip: Run 'cargo clippy --fix --all-targets --all-features' to auto-fix${NC}"
    echo -e "${YELLOW}   ヒント: 'cargo clippy --fix --all-targets --all-features' で自動修正${NC}"
    echo ""
fi

# Stage 3: Security Audit / セキュリティ監査
echo -e "${YELLOW}▶ Security Audit / セキュリティ監査${NC}"
if command -v cargo-audit &> /dev/null; then
    if ! run_check "Security Audit" "cargo audit"; then
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        echo -e "${YELLOW}💡 Tip: Run 'cargo update' to update vulnerable dependencies${NC}"
        echo -e "${YELLOW}   ヒント: 'cargo update' で脆弱な依存関係を更新${NC}"
        echo ""
    fi
else
    echo -e "${YELLOW}⚠️  cargo-audit not installed, skipping${NC}"
    echo -e "${YELLOW}   cargo-audit がインストールされていません、スキップします${NC}"
    echo -e "${YELLOW}   Install with: cargo install cargo-audit${NC}"
    echo ""
fi

# Stage 4: License Check / ライセンスチェック
echo -e "${YELLOW}▶ License Check / ライセンスチェック${NC}"
if command -v cargo-deny &> /dev/null; then
    if ! run_check "License Check" "cargo deny check"; then
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
else
    echo -e "${YELLOW}⚠️  cargo-deny not installed, skipping${NC}"
    echo -e "${YELLOW}   cargo-deny がインストールされていません、スキップします${NC}"
    echo -e "${YELLOW}   Install with: cargo install cargo-deny${NC}"
    echo ""
fi

# Stage 5: Build / ビルド
if ! run_check "Build / ビルド" "cargo build --verbose"; then
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Stage 6: Unit Tests / ユニットテスト
if ! run_check "Unit Tests / ユニットテスト" "cargo test --lib --bins --verbose"; then
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Stage 7: Doc Tests / ドキュメントテスト
if ! run_check "Doc Tests / ドキュメントテスト" "cargo test --doc --verbose"; then
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Stage 8: Integration Tests / 統合テスト
if ! run_check "Integration Tests / 統合テスト" "cargo test --test '*' --verbose"; then
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Summary / サマリー
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Summary / サマリー${NC}"
echo -e "${BLUE}========================================${NC}"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! / すべてのチェックが成功しました！${NC}"
    echo -e "${GREEN}   Ready to push / プッシュ可能です${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_CHECKS check(s) failed / $FAILED_CHECKS 個のチェックが失敗${NC}"
    echo -e "${RED}   Please fix the issues before pushing${NC}"
    echo -e "${RED}   プッシュ前に問題を修正してください${NC}"
    exit 1
fi
