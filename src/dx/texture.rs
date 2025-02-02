use std::path::Path;

use anyhow::Context;
use image::{EncodableLayout, GenericImageView};
use nalgebra::Vector2;
use windows::core::Interface;
use windows::Win32::{
    Foundation::{FALSE, TRUE},
    Graphics::{
        Direct3D11::{
            ID3D11BlendState, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
            ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
            D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
            D3D11_BLEND_SRC_ALPHA, D3D11_BLEND_ZERO, D3D11_COLOR_WRITE_ENABLE_ALL,
            D3D11_RENDER_TARGET_BLEND_DESC, D3D11_RESOURCE_MISC_SHARED, D3D11_SUBRESOURCE_DATA,
            D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
        },
        Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
    },
};

/// Texture and render target combined, the referenced texture
/// is the render target itself
pub struct RenderTargetTexture {
    pub texture: ID3D11Texture2D,
    view: ID3D11RenderTargetView,
}

impl RenderTargetTexture {
    /// Creates a render target thats backed by a texture
    pub fn create(device: &ID3D11Device, width: u32, height: u32) -> anyhow::Result<Self> {
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            // Most supported format for Spout2
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: D3D11_RESOURCE_MISC_SHARED.0 as u32,
        };

        let mut texture: Option<ID3D11Texture2D> = None;
        unsafe { device.CreateTexture2D(&texture_desc, None, Some(&mut texture))? };
        let texture = texture.context("failed to create render target texture")?;

        let mut view: Option<ID3D11RenderTargetView> = None;
        unsafe { device.CreateRenderTargetView(&texture, None, Some(&mut view))? };
        let view = view.context("failed to create render target view")?;

        Ok(Self { texture, view })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.OMSetRenderTargets(Some(&[Some(self.view.clone())]), None);
        }
    }

    pub fn unbind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.OMSetRenderTargets(None, None);
        }
    }

    pub fn clear(&self, ctx: &ID3D11DeviceContext, color: &[f32; 4]) {
        unsafe {
            ctx.ClearRenderTargetView(&self.view, &color);
        }
    }
}

pub struct BlendState {
    state: ID3D11BlendState,
}

impl BlendState {
    /// Blend state that blends alpha layers
    pub fn alpha_blend_state(device: &ID3D11Device) -> anyhow::Result<BlendState> {
        let blend_desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: TRUE,
                SrcBlend: D3D11_BLEND_SRC_ALPHA,
                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D11_BLEND_OP_ADD,
                SrcBlendAlpha: D3D11_BLEND_ONE,
                DestBlendAlpha: D3D11_BLEND_ZERO,
                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
            }; 8],
        };

        let mut state: Option<ID3D11BlendState> = None;
        unsafe { device.CreateBlendState(&blend_desc, Some(&mut state))? };
        let state = state.context("failed to create alpha blend state")?;
        Ok(Self { state })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        let blend_factor = [0.0f32; 4];
        let sample_mask = 0xffffffff;

        unsafe {
            ctx.OMSetBlendState(&self.state, Some(&blend_factor), sample_mask);
        }
    }
}

pub struct Texture {
    pub texture: ID3D11Texture2D,
    pub size: Vector2<u32>,
}

impl Texture {
    /// Loads a texture from the provided path returning the texture
    /// ID of the loaded texture
    pub fn load_from_path<P: AsRef<Path>>(
        device: &ID3D11Device,
        path: P,
    ) -> anyhow::Result<Texture> {
        let img = image::open(path)?;
        let (width, height) = img.dimensions();
        let img = img.to_rgba8(); // Convert to RGBA8 format
        let texture = Self::create_from_data(device, width, height, img.as_bytes())?;

        Ok(Texture {
            texture,
            size: Vector2::new(width, height),
        })
    }

    fn create_from_data(
        device: &ID3D11Device,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> anyhow::Result<ID3D11Texture2D> {
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: data.as_ptr().cast(),
            SysMemPitch: width * 4, /* R8G8B8A8 = 4 bytes */
            SysMemSlicePitch: 0,
        };

        let mut texture: Option<ID3D11Texture2D> = None;

        unsafe { device.CreateTexture2D(&texture_desc, Some(&init_data), Some(&mut texture))? };

        texture.context("failed to create texture")
    }
}
