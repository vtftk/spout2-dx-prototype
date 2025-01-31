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
use nalgebra::Vector2;
use nalgebra::Vector3;
use spout::SpoutSender;
use winapi::shared::dxgiformat::*;
use winapi::shared::winerror::*;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

mod com;
mod dx;
mod spout;

#[repr(C)]
pub struct Vertex {
    pos: Vector3<f32>,
    tex: Vector2<f32>,
}

pub struct Throwable {
    /// ID of the texture used by this item
    pub texture: Texture,
    /// Whether to pixelate the texture
    pub pixelate: bool,
    /// Current position of the item
    pub position: Vector2<f32>,
    /// Velocity of the item
    pub velocity: Vector2<f32>,
    /// Scale of the item
    pub scale: f32,
    /// Rotation of the item
    pub rotation: f32,
}

// Constant buffer structure
#[repr(C)]
struct ConstantBufferData {
    size: Vector2<f32>,
    position: Vector2<f32>,
    yaw: f32,
    padding: [f32; 3],
}

fn main() -> anyhow::Result<()> {
    let screen_size: Vector2<u32> = Vector2::new(1920, 1080);

    let mut sender = SpoutSender::create()?;
    sender.set_sender_name("VTFTK")?;
    sender.set_sender_format()?;

    let (mut device, ctx) = create_device_and_context()?;
    let mut rtv = RenderTargetTexture::create(&device, screen_size.x, screen_size.y)?;

    sender.open_directx11(device.as_mut())?;

    unsafe {
        let mut item_texture = Texture::load_from_path(&device, "./assets/test1.png")?;
        let mut srv =
            ShaderResourceView::create_from_texture(&device, item_texture.texture.cast_as_mut())?;

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
        let vertex_shader_blob =
            ShaderBlob::compile(include_bytes!("shader.vert"), "vs_5_0", "VSMain")?;
        let pixel_shader_blob =
            ShaderBlob::compile(include_bytes!("shader.frag"), "ps_5_0", "PSMain")?;

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

        let pixelate = true;

        rtv.bind(&ctx);
        viewport.bind(&ctx);
        blend_state.bind(&ctx);

        let clear_color = [1.0f32, 0.0, 0.0, 1.0];

        let mut throwables: Vec<Throwable> = vec![
        //  Throwable {
        //     texture,
        //     pixelate,
        //     position: todo!(),
        //     velocity: todo!(),
        //     scale: todo!(),
        //     rotation: todo!(),
        // }
        ];

        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: std::mem::size_of::<ConstantBufferData>() as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: 0,
            StructureByteStride: 0,
        };

        let mut initial_data = ConstantBufferData {
            size: Vector2::zeros(),
            position: Vector2::zeros(),
            yaw: 0.0,
            padding: [0.0f32; 3],
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: &mut initial_data as *const _ as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let mut constant_buffer = std::ptr::null_mut();
        let hr = device.CreateBuffer(&buffer_desc, &init_data, &mut constant_buffer);
        hr_bail!(hr, "failed to create constant buffer");

        loop {
            // Update throwables
            for item in &mut throwables {
                // item.position += item.velocity;

                // Apply rotation to the throwable object (just for example purposes)
                item.rotation += 1.0;
                if item.rotation >= 360.0 {
                    item.rotation = 0.0;
                }
            }

            // Clear to red
            rtv.clear(&ctx, &clear_color);

            // Set the vertex shader to current
            vertex_shader.set_shader(&ctx);
            pixel_shader.set_shader(&ctx);

            // Set current sampler
            if pixelate {
                pixelate_sampler.bind(&ctx);
            } else {
                linear_sampler.bind(&ctx);
            }

            // Set shader resources
            shader_input_layout.bind(&ctx);

            // Bind vertex and index buffers
            vertex_buffer.bind(&ctx);
            index_buffer.bind(&ctx);

            // Set drawing mode and draw from index buffer
            ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            let screen_position = Vector2::new(250.0, 250.0);

            let screen_size_f32 = screen_size.cast::<f32>();

            let norm_size = item_texture
                .size
                .cast::<f32>()
                .component_div(&screen_size_f32)
                .component_mul(&Vector2::new(16.0, 16.0));

            let norm_pos = screen_position
                .cast::<f32>()
                .component_div(&screen_size_f32)
                .component_mul(&Vector2::new(1.0, -1.0))
                + Vector2::new(-1.0, 1.0);

            // Update constant buffer
            let cb_data = ConstantBufferData {
                size: norm_size,
                position: norm_pos,
                yaw: 0.0,
                padding: [0.0f32; 3],
            };

            // Inside the loop where you update the constant buffer:
            let mut mapped_resource = std::mem::zeroed();
            let hr = ctx.Map(
                constant_buffer.cast(),
                0,
                D3D11_MAP_WRITE_DISCARD,
                0,
                &mut mapped_resource,
            );

            hr_bail!(hr, "failed to map constant buffer");

            // Copy the new data into the mapped buffer
            std::ptr::copy_nonoverlapping(
                &cb_data as *const _ as *const _,
                mapped_resource.pData as *mut ConstantBufferData,
                1,
            );
            ctx.Unmap(constant_buffer.cast(), 0);

            // Set constant buffer
            ctx.VSSetConstantBuffers(0, 1, &constant_buffer);
            srv.bind(&ctx);
            ctx.DrawIndexed(6, 0, 0);

            sender.send_texture(rtv.texture.as_mut())?;

            sender.hold_fps(30.into())?;
        }
    }
}
