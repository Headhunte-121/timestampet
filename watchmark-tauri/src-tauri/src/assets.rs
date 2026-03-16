use crate::db::get_app_data_dir;

const FONT_URL: &str = "https://github.com/rsms/inter/releases/download/v4.0/Inter-Variable.woff2";

#[tracing::instrument]
pub async fn ensure_fonts() -> Result<(), crate::error::AppError> {
    let fonts_dir = get_app_data_dir().join("fonts");
    if !fonts_dir.exists() {
        std::fs::create_dir_all(&fonts_dir)?;
    }

    let font_path = fonts_dir.join("Inter-Variable.woff2");

    if font_path.exists() {
        tracing::info!(action = "ensure_fonts", "[ASSETS] 🔤 Inter font already exists locally. Skipping download.");
        return Ok(());
    }

    tracing::info!(action = "ensure_fonts", "[ASSETS] ⬇️ Downloading Inter font for offline support...");

    // Download the font
    let response = reqwest::get(FONT_URL).await?;

    if !response.status().is_success() {
        tracing::error!(action = "ensure_fonts_failed", "[ASSETS] ❌ Failed to download font: HTTP {}", response.status());
        return Err(crate::error::AppError::Custom(format!("Failed to download font: HTTP {}", response.status())));
    }

    let bytes = response.bytes().await?;
    std::fs::write(&font_path, bytes)?;

    tracing::info!(action = "ensure_fonts", "[ASSETS] ✅ Inter font downloaded and saved successfully.");
    Ok(())
}
