use std::fs;
use std::path::Path;

fn main() {
    // Auto-increment build number on each build
    // ビルドごとにビルド番号を自動インクリメント
    let build_number_path = Path::new("BUILD_NUMBER");

    let build_number = if build_number_path.exists() {
        let content = fs::read_to_string(build_number_path).unwrap_or_else(|_| "0".to_string());
        let num: u32 = content.trim().parse().unwrap_or(0);
        num + 1
    } else {
        1
    };

    // Write back incremented build number
    // インクリメントしたビルド番号を書き戻す
    fs::write(build_number_path, format!("{}\n", build_number))
        .expect("Failed to write BUILD_NUMBER");

    // Expose build number as compile-time environment variable
    // ビルド番号をコンパイル時環境変数として公開
    println!("cargo:rustc-env=SIKULID_BUILD_NUMBER={}", build_number);

    // Also expose full version (0.8.BUILD_NUMBER)
    // フルバージョンも公開 (0.8.BUILD_NUMBER)
    println!("cargo:rustc-env=SIKULID_VERSION=0.8.{}", build_number);

    // Re-run if BUILD_NUMBER changes (shouldn't normally happen externally)
    println!("cargo:rerun-if-changed=BUILD_NUMBER");

    tauri_build::build()
}
