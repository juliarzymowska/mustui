use image::DynamicImage;

use crate::{client::Backend, error::Result};

pub async fn fetch_artwork(backend: &Backend, url: &str) -> Result<DynamicImage> {
    let bytes = backend
        .http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}
