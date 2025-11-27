//! Script runner module
//! スクリプト実行モジュール

use std::path::Path;
use anyhow::{Result, Context, bail};
use sikulid::{Region, Screen};

/// Run a Sikuli-D script
/// Sikuli-Dスクリプトを実行
pub fn run_script(
    script: &Path,
    args: &[String],
    workdir: Option<&Path>,
    timeout: u64,
) -> Result<()> {
    log::info!("Running script: {}", script.display());

    // Check if script exists
    if !script.exists() {
        bail!("Script not found: {}", script.display());
    }

    // Determine script type
    let is_bundle = script.extension()
        .map(|ext| ext == "sikuli")
        .unwrap_or(false);

    let script_path = if is_bundle {
        // Find main script in bundle
        crate::bundle::find_main_script(script)?
    } else {
        script.to_path_buf()
    };

    log::debug!("Script path: {}", script_path.display());
    log::debug!("Arguments: {:?}", args);
    log::debug!("Working directory: {:?}", workdir);
    log::debug!("Timeout: {} seconds", timeout);

    // Execute with Python
    crate::python::execute_script(&script_path, args, workdir, timeout)
}

/// Find an image on screen
/// 画面上で画像を検索
pub fn find_image(image_path: &Path, similarity: f64, find_all_matches: bool) -> Result<()> {
    log::info!("Finding image: {} (similarity: {})", image_path.display(), similarity);

    if !image_path.exists() {
        bail!("Image not found: {}", image_path.display());
    }

    // Capture screen
    let screen = Screen::primary();
    let screen_capture = screen.capture()
        .map_err(|e| anyhow::anyhow!("Failed to capture screen: {}", e))?;

    // Load template as Pattern
    let pattern = sikulid::Pattern::from_file(image_path.to_str().unwrap_or(""))
        .map_err(|e| anyhow::anyhow!("Failed to read template image: {}", e))?
        .similar(similarity);

    // Create matcher
    let matcher = sikulid::ImageMatcher::new().with_min_similarity(similarity);

    if find_all_matches {
        match matcher.find_all(&screen_capture, &pattern) {
            Ok(matches) => {
                println!("Found {} matches:", matches.len());
                for (i, m) in matches.iter().enumerate() {
                    println!(
                        "  [{}] x={}, y={}, w={}, h={}, score={:.3}",
                        i + 1, m.region.x, m.region.y, m.region.width, m.region.height, m.score
                    );
                }
            }
            Err(e) => {
                bail!("Match failed: {}", e);
            }
        }
    } else {
        match matcher.find(&screen_capture, &pattern) {
            Ok(Some(m)) => {
                println!(
                    "Found: x={}, y={}, w={}, h={}, score={:.3}",
                    m.region.x, m.region.y, m.region.width, m.region.height, m.score
                );
            }
            Ok(None) => {
                println!("Not found (similarity threshold: {})", similarity);
            }
            Err(e) => {
                bail!("Match failed: {}", e);
            }
        }
    }

    Ok(())
}

/// Capture screen to file
/// 画面をファイルにキャプチャ
pub fn capture_screen(output: Option<&Path>, region_str: Option<&str>) -> Result<()> {
    let output_path = output.unwrap_or(Path::new("screenshot.png"));

    let screen = Screen::primary();

    let screen_image = if let Some(region_str) = region_str {
        // Parse region: "x,y,w,h"
        let parts: Vec<i32> = region_str
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid region format. Use: x,y,w,h")?;

        if parts.len() != 4 {
            bail!("Invalid region format. Use: x,y,w,h");
        }

        let region = Region::new(parts[0], parts[1], parts[2] as u32, parts[3] as u32);
        log::info!("Capturing region: {:?}", region);
        screen.capture_region(&region)
            .map_err(|e| anyhow::anyhow!("Failed to capture region: {}", e))?
    } else {
        log::info!("Capturing full screen");
        screen.capture()
            .map_err(|e| anyhow::anyhow!("Failed to capture screen: {}", e))?
    };

    // Save image
    screen_image.save(output_path)
        .context("Failed to save screenshot")?;

    println!("Screenshot saved: {}", output_path.display());
    Ok(())
}

/// Show system information
/// システム情報を表示
pub fn show_info() -> Result<()> {
    println!("=== SikuliX Runtime Info ===");
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Platform: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!();

    // Screen info
    println!("=== Screen Info ===");
    let mut screen = Screen::primary();
    match screen.dimensions() {
        Ok((width, height)) => {
            println!("Primary screen: {}x{}", width, height);
        }
        Err(e) => {
            println!("Failed to get screen info: {}", e);
        }
    }
    println!();

    // Python info
    println!("=== Python Info ===");
    match sikulid::python::detect_system_python() {
        Ok(python) => {
            println!(
                "Python: {}.{}.{} ({})",
                python.version.0, python.version.1, python.version.2,
                python.path.display()
            );
        }
        Err(e) => {
            println!("Python not found: {}", e);
        }
    }

    Ok(())
}
