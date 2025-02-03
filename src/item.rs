use std::{path::PathBuf, time::Instant};

use nalgebra::{Vector2, Vector3};
use winapi::{
    shared::dxgiformat::{
        DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32_UINT,
    },
    um::{
        d3d11::{
            ID3D11Device, ID3D11DeviceContext, D3D11_INPUT_ELEMENT_DESC,
            D3D11_INPUT_PER_VERTEX_DATA,
        },
        d3dcommon::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    },
};

use crate::dx::{
    buffer::{ConstantBuffer, IndexBuffer, VertexBuffer},
    sampler::SamplerState,
    shader::{PixelShader, ShaderBlob, ShaderInputLayout, ShaderResourceView, VertexShader},
    texture::Texture,
};

/// Definition of an item to be thrown
#[derive()]
pub struct ItemDefinition {
    // Path to the throwable
    pub texture_path: PathBuf,
    // Whether to pixelate the texture when scaling during render
    pub pixelate: bool,
    /// Scale for the image
    pub scale: f32,
}

impl ItemDefinition {
    pub fn create_render_item(
        self,
        device: &ID3D11Device,
        mut item_texture: Texture,
        item_data: ItemDataBuffer,
    ) -> anyhow::Result<RenderItemDefinition> {
        let srv =
            ShaderResourceView::create_from_texture(device, item_texture.texture.cast_as_mut())?;

        Ok(RenderItemDefinition {
            _texture: item_texture,
            shader_resource_view: srv,
            pixelate: self.pixelate,
            start_time: Instant::now(),
            item_data,
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

    pub item_data: ItemDataBuffer,
}

impl RenderItemDefinition {
    /// Updates the timing data for this item
    pub fn update(&mut self) -> anyhow::Result<()> {
        let elapsed_time = self.start_time.elapsed().as_millis() as f32;

        self.item_data.elapsed_time = elapsed_time;

        Ok(())
    }

    pub fn render(&mut self, ctx: &ID3D11DeviceContext) {
        // Bind item texture
        self.shader_resource_view.bind(ctx);

        unsafe {
            // Set drawing mode and draw from index buffer
            ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            ctx.DrawIndexed(6, 0, 0);
        }
    }
}

#[derive(Debug, Default)]
#[repr(C, align(16))]
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

    /// Elapsed time since the item creation
    pub elapsed_time: f32,
}

/// Creates a vertex buffer used to render items
pub fn create_item_vertex_buffer(device: &ID3D11Device) -> anyhow::Result<VertexBuffer> {
    #[repr(C)]
    struct Vertex {
        pos: Vector2<f32>,
        tex: Vector2<f32>,
    }

    let vertices = [
        // Top-left
        Vertex {
            pos: Vector2::new(-0.5, -0.5),
            tex: Vector2::new(0.0, 1.0),
        },
        // Bottom-left
        Vertex {
            pos: Vector2::new(-0.5, 0.5),
            tex: Vector2::new(0.0, 0.0),
        },
        // Bottom-right
        Vertex {
            pos: Vector2::new(0.5, 0.5),
            tex: Vector2::new(1.0, 0.0),
        },
        // Top-right
        Vertex {
            pos: Vector2::new(0.5, -0.5),
            tex: Vector2::new(1.0, 1.0),
        },
    ];

    let vertex_buffer = VertexBuffer::create_from_array(device, &vertices)?;
    Ok(vertex_buffer)
}

/// Creates the index buffer for rendering items
pub fn create_item_index_buffer(device: &ID3D11Device) -> anyhow::Result<IndexBuffer> {
    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
    let index_buffer = IndexBuffer::create_from_array(device, &indices, DXGI_FORMAT_R32_UINT)?;

    Ok(index_buffer)
}

/// Shader for rendering items
pub struct ItemShader {
    input_layout: ShaderInputLayout,
    vertex: VertexShader,
    pixel: PixelShader,
}

impl ItemShader {
    pub fn create(device: &ID3D11Device) -> anyhow::Result<ItemShader> {
        // Compile shaders
        let vertex_shader_blob = ShaderBlob::compile(
            include_bytes!("shaders/vertex_shader.hlsl"),
            "vs_5_0",
            "VSMain",
        )?;
        let pixel_shader_blob = ShaderBlob::compile(
            include_bytes!("shaders/fragment_shader.hlsl"),
            "ps_5_0",
            "PSMain",
        )?;

        // Create shaders
        let vertex = VertexShader::create(device, vertex_shader_blob.clone())?;
        let pixel = PixelShader::create(device, pixel_shader_blob)?;

        // Create shader input layout
        let input_layout = ShaderInputLayout::create(
            device,
            &[
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: "POSITION\0".as_ptr() as _,
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: "TEXCOORD\0".as_ptr() as _,
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 8,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
            ],
            vertex_shader_blob,
        )?;

        Ok(ItemShader {
            input_layout,
            vertex,
            pixel,
        })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        // Set shader resources
        self.input_layout.bind(ctx);

        // Set the vertex shader to current
        self.vertex.set_shader(ctx);
        self.pixel.set_shader(ctx);
    }
}

pub struct ItemRenderContext {
    pub item_shader: ItemShader,
    pub index_buffer: IndexBuffer,
    pub vertex_buffer: VertexBuffer,
    pub linear_sampler: SamplerState,
    pub pixelate_sampler: SamplerState,
    pub item_data: ConstantBuffer<ItemDataBuffer>,
}

impl ItemRenderContext {
    pub fn create(device: &ID3D11Device) -> anyhow::Result<Self> {
        let item_shader = ItemShader::create(&device)?;
        let index_buffer = create_item_index_buffer(device)?;
        let vertex_buffer = create_item_vertex_buffer(device)?;

        let linear_sampler = SamplerState::linear(device)?;
        let pixelate_sampler = SamplerState::pixelate(device)?;

        let item_data = ConstantBuffer::create_default(device)?;

        Ok(Self {
            item_shader,
            index_buffer,
            vertex_buffer,
            linear_sampler,
            pixelate_sampler,
            item_data,
        })
    }

    pub fn set_current_data(
        &mut self,
        ctx: &ID3D11DeviceContext,
        item_data: &ItemDataBuffer,
    ) -> anyhow::Result<()> {
        self.item_data.replace(ctx, item_data)?;
        Ok(())
    }

    /// Binds the constant buffers for this item
    pub fn bind_constants(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            // Bind item data and timing data
            let buffers = [self.item_data.buffer.as_ptr()];
            ctx.VSSetConstantBuffers(0, 1, buffers.as_ptr());
        }
    }

    pub fn prepare_render(&mut self, ctx: &ID3D11DeviceContext) {
        // Bind item shader
        self.item_shader.bind(ctx);

        // Bind vertex and index buffers
        self.vertex_buffer.bind(ctx);
        self.index_buffer.bind(ctx);

        self.bind_constants(ctx);
    }

    pub fn set_sampler(&mut self, ctx: &ID3D11DeviceContext, pixelate: bool) {
        // Set current sampler
        if pixelate {
            self.pixelate_sampler.bind(ctx);
        } else {
            self.linear_sampler.bind(ctx);
        }
    }
}
