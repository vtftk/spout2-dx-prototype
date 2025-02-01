use dx::buffer::IndexBuffer;
use dx::buffer::VertexBuffer;
use dx::device::create_device_and_context;
use dx::device::Viewport;
use dx::sampler::SamplerState;
use dx::shader::PixelShader;
use dx::shader::ShaderBlob;
use dx::shader::ShaderInputLayout;
use dx::shader::ShaderResourceView;
use dx::shader::VertexShader;
use dx::texture::BlendState;
use dx::texture::RenderTargetTexture;
use dx::texture::Texture;
use item::ItemDataBuffer;
use item::ItemDefinition;
use item::TimingDataBuffer;
use nalgebra::Vector2;
use nalgebra::Vector3;
use spout::SpoutSender;
use winapi::shared::dxgiformat::*;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

mod com;
mod dx;
mod item;
mod spout;

#[repr(C)]
pub struct Vertex {
    pos: Vector3<f32>,
    tex: Vector2<f32>,
}

pub struct ThrowableRenderItem {
    /// Shader resource view for the item texture
    pub srv: ShaderResourceView,
}

fn to_screen_space(vector: Vector2<f32>, screen_size: &Vector2<f32>) -> Vector2<f32> {
    let relative_pos = vector.component_div(screen_size);

    Vector2::new(2.0 * relative_pos.x - 1.0, 1.0 - 2.0 * relative_pos.y)
}

fn main() -> anyhow::Result<()> {
    let screen_size: Vector2<u32> = Vector2::new(1920, 1080);

    let mut sender = SpoutSender::create()?;
    sender.set_sender_name("VTFTK")?;
    sender.set_sender_format()?;

    let (mut device, ctx) = create_device_and_context()?;
    let mut rtv = RenderTargetTexture::create(&device, screen_size.x, screen_size.y)?;

    sender.open_directx11(device.as_mut())?;

    let item_definitions = [
        ItemDefinition {
            texture_path: "./assets/test1.png".into(),
            pixelate: true,
        },
        ItemDefinition {
            texture_path: "./assets/test2.png".into(),
            pixelate: true,
        },
    ];

    let mut items = Vec::new();

    let screen_size_f32 = screen_size.cast::<f32>();

    let start_position = Vector2::new(0.0, 0.0);
    let end_position = Vector2::new(1.0, 1.0);

    for definition in item_definitions {
        let item_texture = Texture::load_from_path(&device, &definition.texture_path)?;
        let scale: f32 = 1.0;
        let spin_speed = 5000.0;
        let duration = 1000.0;

        let texture_size = item_texture.size.cast::<f32>();

        // Texture size relative to the window
        let norm_texture_size = texture_size.component_div(&screen_size_f32);
        let start_pos = start_position.component_mul(&screen_size_f32);
        let end_pos = end_position.component_mul(&screen_size_f32);

        let item_data = ItemDataBuffer {
            norm_texture_size,
            start_position: to_screen_space(start_pos, &screen_size_f32),
            end_position: to_screen_space(end_pos, &screen_size_f32),
            spin_speed,
            scale,
            duration,
            _padding: [0.0, 0.0, 0.0],
        };

        dbg!(&item_data);

        let timing_data = TimingDataBuffer {
            elapsed_time: 0.0,
            _padding: [0.0, 0.0, 0.0],
        };

        let data = definition.create_render_item(&device, item_texture, item_data, timing_data)?;

        items.push(data);
    }

    unsafe {
        // Create input layout
        let layout_desc = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: "POSITION\0".as_ptr() as _,
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
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
                AlignedByteOffset: 12,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];

        let vertices = [
            // Top-left
            Vertex {
                pos: Vector3::new(-0.5, -0.5, 0.0),
                tex: Vector2::new(0.0, 1.0),
            },
            // Bottom-left
            Vertex {
                pos: Vector3::new(-0.5, 0.5, 0.0),
                tex: Vector2::new(0.0, 0.0),
            },
            // Bottom-right
            Vertex {
                pos: Vector3::new(0.5, 0.5, 0.0),
                tex: Vector2::new(1.0, 0.0),
            },
            // Top-right
            Vertex {
                pos: Vector3::new(0.5, -0.5, 0.0),
                tex: Vector2::new(1.0, 1.0),
            },
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

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
        let vertex_shader = VertexShader::create(device.as_mut(), vertex_shader_blob.clone())?;
        let pixel_shader = PixelShader::create(device.as_mut(), pixel_shader_blob)?;
        let mut shader_input_layout =
            ShaderInputLayout::create(&device, &layout_desc, vertex_shader_blob)?;

        let mut vertex_buffer = VertexBuffer::create_from_array(&device, &vertices)?;
        let mut index_buffer =
            IndexBuffer::create_from_array(&device, &indices, DXGI_FORMAT_R32_UINT)?;

        let viewport = Viewport::new(
            Vector2::new(screen_size.x as f32, screen_size.y as f32),
            Vector2::new(0.0, 1.0),
        );
        let mut blend_state = BlendState::alpha_blend_state(&device)?;

        let mut linear_sampler = SamplerState::linear(&device)?;
        let mut pixelate_sampler = SamplerState::pixelate(&device)?;

        rtv.bind(&ctx);
        viewport.bind(&ctx);
        blend_state.bind(&ctx);

        let clear_color = [0.0f32, 0.0, 0.0, 0.0];

        loop {
            // Clear to red
            rtv.clear(&ctx, &clear_color);

            for item in &mut items {
                let elapsed_time = item.start_time.elapsed().as_millis() as f32;

                // Update timing data
                item.timing_data.replace(
                    &ctx,
                    &TimingDataBuffer {
                        elapsed_time,
                        _padding: [0.0, 0.0, 0.0],
                    },
                )?;

                // Set shader resources
                shader_input_layout.bind(&ctx);

                // Set the vertex shader to current
                vertex_shader.set_shader(&ctx);
                pixel_shader.set_shader(&ctx);

                // Set current sampler
                if item.pixelate {
                    pixelate_sampler.bind(&ctx);
                } else {
                    linear_sampler.bind(&ctx);
                }

                // Bind item data and timing data
                let buffers = [
                    item.item_data.buffer.as_ptr(),
                    item.timing_data.buffer.as_ptr(),
                ];
                ctx.VSSetConstantBuffers(0, 2, buffers.as_ptr());

                // Bind item texture
                item.shader_resource_view.bind(&ctx);

                // Bind vertex and index buffers
                vertex_buffer.bind(&ctx);
                index_buffer.bind(&ctx);

                // Set drawing mode and draw from index buffer
                ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

                ctx.DrawIndexed(6, 0, 0);
            }

            sender.send_texture(rtv.texture.as_mut())?;
            sender.hold_fps(30.into())?;
        }
    }
}
