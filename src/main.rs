use std::ops::Deref;

use app::render;
use app::setup_render_world;
use app::RenderContext;
use dx::shader::ShaderResourceView;
use dx::texture::Texture;
use item::ItemDataBuffer;
use item::ItemDefinition;
use item::TimingDataBuffer;
use nalgebra::Vector2;
use spout::SpoutSender;

mod app;
mod dx;
mod item;
mod spout;

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

    let mut render_ctx = RenderContext::create(screen_size)?;

    let device = render_ctx.device.clone();

    sender.open_directx11(render_ctx.device.clone())?;

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

        let timing_data = TimingDataBuffer {
            elapsed_time: 0.0,
            _padding: [0.0, 0.0, 0.0],
        };

        let data = definition.create_render_item(&device, item_texture, item_data, timing_data)?;

        items.push(data);
    }

    setup_render_world(&mut render_ctx);

    loop {
        render(&mut render_ctx, &mut items)?;

        sender.send_texture(&render_ctx.rtv.texture)?;
        sender.hold_fps(30.into())?;
    }
}
