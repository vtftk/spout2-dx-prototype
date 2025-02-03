use nalgebra::Vector2;
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext};

use crate::{
    com::ComPtr,
    dx::{
        device::{create_device_and_context, Viewport},
        texture::{BlendState, RenderTargetTexture},
    },
    item::{ItemRenderContext, RenderItemDefinition},
};

/// Rendering context with DirectX11
pub struct RenderContext {
    /// DirectX11 device
    pub device: ComPtr<ID3D11Device>,
    /// DirectX11 context
    pub ctx: ComPtr<ID3D11DeviceContext>,
    /// Rendering target texture
    pub rtv: RenderTargetTexture,
    ///  World rendering context
    pub world: WorldRenderContext,
    /// Item rendering context
    pub item: ItemRenderContext,
}

impl RenderContext {
    pub fn create(screen_size: Vector2<u32>) -> anyhow::Result<RenderContext> {
        let (device, ctx) = create_device_and_context()?;
        let rtv = RenderTargetTexture::create(&device, screen_size.x, screen_size.y)?;
        let world = WorldRenderContext::create(&device, screen_size.cast::<f32>())?;
        let item = ItemRenderContext::create(&device)?;

        Ok(RenderContext {
            device,
            ctx,
            rtv,
            world,
            item,
        })
    }
}

pub struct WorldRenderContext {
    pub screen_size: Vector2<f32>,
    pub viewport: Viewport,
    pub blend_state: BlendState,
}

impl WorldRenderContext {
    pub fn create(
        device: &ID3D11Device,
        screen_size: Vector2<f32>,
    ) -> anyhow::Result<WorldRenderContext> {
        let viewport = Viewport::new(screen_size, Vector2::new(0.0, 1.0));
        let blend_state = BlendState::alpha_blend_state(device)?;

        Ok(WorldRenderContext {
            screen_size,
            viewport,
            blend_state,
        })
    }
}

static CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub fn setup_render_world(render_ctx: &mut RenderContext) {
    let ctx = &mut render_ctx.ctx;
    let world = &mut render_ctx.world;
    let item_ctx = &mut render_ctx.item;

    // Bind the render texture
    render_ctx.rtv.bind(ctx);

    // Setup viewport
    world.viewport.bind(ctx);

    // Setup blending for layers
    world.blend_state.bind(ctx);

    // Prepare for rendering items
    item_ctx.prepare_render(ctx);

    // Bind constant buffer for item rendering
    item_ctx.bind_constants(ctx);
}

pub fn render(
    render_ctx: &mut RenderContext,
    items: &mut Vec<RenderItemDefinition>,
) -> anyhow::Result<()> {
    let ctx = &mut render_ctx.ctx;
    let item_ctx = &mut render_ctx.item;

    // Clear background color
    render_ctx.rtv.clear(ctx, &CLEAR_COLOR);

    for item in items {
        // Update item data
        item.update()?;

        // Update the constant buffer using the current data
        item_ctx.set_current_data(ctx, &item.item_data)?;

        // Set current sampler for pixelation
        item_ctx.set_sampler(ctx, item.pixelate);

        // Render item
        item.render(ctx);
    }

    Ok(())
}
