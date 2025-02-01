use std::{path::PathBuf, time::Instant};

use nalgebra::Vector2;
use winapi::um::d3d11::ID3D11Device;

use crate::dx::{buffer::ConstantBuffer, shader::ShaderResourceView, texture::Texture};

/// Definition of an item to be thrown
pub struct ItemDefinition {
    // Path to the throwable
    pub texture_path: PathBuf,
    // Whether to pixelate the texture when scaling during render
    pub pixelate: bool,
}

impl ItemDefinition {
    pub fn create_render_item(
        self,
        device: &ID3D11Device,
        mut item_texture: Texture,
        item_data: ItemDataBuffer,
        timing_data: TimingDataBuffer,
    ) -> anyhow::Result<RenderItemDefinition> {
        let srv =
            ShaderResourceView::create_from_texture(device, item_texture.texture.cast_as_mut())?;

        let item_data = ConstantBuffer::create(device, item_data)?;
        let timing_data = ConstantBuffer::create(device, timing_data)?;

        Ok(RenderItemDefinition {
            _texture: item_texture,
            shader_resource_view: srv,
            pixelate: self.pixelate,
            start_time: Instant::now(),
            item_data,
            timing_data,
        })
    }
}

/// Item definition that is ready to render
pub struct RenderItemDefinition {
    /// Reference to item texture (Maintained while the shader resource view is still around)
    pub _texture: Texture,

    /// Shader resource view for the texture
    pub shader_resource_view: ShaderResourceView,

    /// Whether to pixelate when rendering
    pub pixelate: bool,

    /// Instance the item was created at
    pub start_time: Instant,

    pub item_data: ConstantBuffer<ItemDataBuffer>,
    pub timing_data: ConstantBuffer<TimingDataBuffer>,
}

/// Buffer storing positional data for an item, this data
/// is provided as a Constant Buffer to the shader and used
/// to interpolate
///
/// Immutable between re-renders does not change
#[derive(Debug, Default)]
#[repr(C)]
pub struct ItemDataBuffer {
    /// Normalized world size for the texture (texture_size / screen_size) scaled
    /// ahead of time for the current render target size
    pub norm_texture_size: Vector2<f32>,

    /// Initial start position (Normalized to screen size)
    pub start_position: Vector2<f32>,

    /// Final end position (Normalized to screen size)
    pub end_position: Vector2<f32>,

    /// Speed to spin at (ms)
    pub spin_speed: f32,

    /// Relative scaling of the item image
    pub scale: f32,

    /// Duration the item should exist for
    pub duration: f32,

    pub _padding: [f32; 3],
}

/// Buffer storing timing data for an item, updated frequently
/// to update current time step between frames.
///
/// Updated on every frame using the latest time
#[derive(Debug, Default)]
#[repr(C)]
pub struct TimingDataBuffer {
    /// Elapsed time since the item creation
    pub elapsed_time: f32,

    /// Padding to reach 16 byte boundary
    pub _padding: [f32; 3],
}
