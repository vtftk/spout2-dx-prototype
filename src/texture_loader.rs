use image::{GenericImageView, ImageBuffer, Rgba};
use tokio::{
    sync::{mpsc, oneshot},
    task::spawn_blocking,
};

pub struct TextureData {
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub width: u32,
    pub height: u32,
}
pub async fn load_texture_data(data: Vec<u8>) -> anyhow::Result<TextureData> {
    spawn_blocking(|| -> anyhow::Result<TextureData> {
        let data = data;
        let img = image::load_from_memory(&data)?;
        let (width, height) = img.dimensions();
        let img = img.to_rgba8(); // Convert to RGBA8 format

        Ok(TextureData {
            buffer: img,
            width,
            height,
        })
    })
    .await?
}
