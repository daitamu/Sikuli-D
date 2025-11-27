@echo off
REM Benchmark execution script for sikulix-core
REM sikulix-core のベンチマーク実行スクリプト

setlocal enabledelayedexpansion

echo === SikuliX Core Benchmarks ===
echo Starting benchmark suite...
echo.

REM Navigate to core-rs directory
cd /d "%~dp0..\core-rs"

REM Check if Rust is installed
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Cargo not found. Please install Rust.
    exit /b 1
)

REM Create results directory
set RESULTS_DIR=benchmark_results
if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM Generate timestamp
for /f "tokens=2 delims==" %%I in ('wmic os get localdatetime /value') do set datetime=%%I
set TIMESTAMP=%datetime:~0,8%_%datetime:~8,6%
set RESULTS_FILE=%RESULTS_DIR%\benchmark_%TIMESTAMP%.txt

echo Results will be saved to: %RESULTS_FILE%
echo.

REM System information
echo === System Information === > "%RESULTS_FILE%"
echo Date: %date% %time% >> "%RESULTS_FILE%"
rustc --version >> "%RESULTS_FILE%"
cargo --version >> "%RESULTS_FILE%"
echo OS: Windows >> "%RESULTS_FILE%"
echo Number of processors: %NUMBER_OF_PROCESSORS% >> "%RESULTS_FILE%"
echo. >> "%RESULTS_FILE%"

REM Build in release mode first
echo Building in release mode...
cargo build --release
if %errorlevel% neq 0 (
    echo Error: Build failed
    exit /b 1
)

echo.
echo Running benchmarks...
echo.

REM Run all benchmarks
echo === Benchmark Results === >> "%RESULTS_FILE%"
echo. >> "%RESULTS_FILE%"

REM Matching benchmarks
echo [1/3] Running image matching benchmarks...
cargo bench --bench matching 2>&1 | tee -a "%RESULTS_FILE%"

echo. >> "%RESULTS_FILE%"

REM Screen capture benchmarks
echo [2/3] Running screen capture benchmarks...
cargo bench --bench screen_capture 2>&1 | tee -a "%RESULTS_FILE%"

echo. >> "%RESULTS_FILE%"

REM NCC calculation benchmarks
echo [3/3] Running NCC calculation benchmarks...
cargo bench --bench ncc_calculation 2>&1 | tee -a "%RESULTS_FILE%"

echo.
echo === Benchmarks Complete ===
echo.
echo Results saved to: %RESULTS_FILE%
echo.

echo === Performance Summary === >> "%RESULTS_FILE%"
echo. >> "%RESULTS_FILE%"

echo.
echo Tip: Compare results with BENCHMARK_RESULTS.md targets
echo Tip: Run 'cargo bench -- --save-baseline <name>' to save a baseline for comparison
echo.

REM Optional: Show Criterion HTML report location
if exist "target\criterion" (
    echo Criterion reports available at:
    echo   file:///%CD%\target\criterion\report\index.html
    echo.
)

endlocal
exit /b 0
