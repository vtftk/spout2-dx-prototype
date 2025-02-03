use std::path::Path;

use image::{EncodableLayout, GenericImageView};
use nalgebra::Vector2;
use winapi::{
    shared::{
        basetsd::UINT8,
        dxgiformat::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM},
        dxgitype::DXGI_SAMPLE_DESC,
    },
    um::d3d11::{
        ID3D11BlendState, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
        ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_BLEND_DESC,
        D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA,
        D3D11_BLEND_ZERO, D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_RENDER_TARGET_BLEND_DESC,
        D3D11_RESOURCE_MISC_SHARED, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_DEFAULT,
    },
};

use crate::{com::ComPtr, hr_bail};

/// Texture and render target combined, the referenced texture
/// is the render target itself
pub struct RenderTargetTexture {
    pub texture: ComPtr<ID3D11Texture2D>,
    view: ComPtr<ID3D11RenderTargetView>,
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
            BindFlags: D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE,
            CPUAccessFlags: 0,
            MiscFlags: D3D11_RESOURCE_MISC_SHARED,
        };

        let mut texture: *mut ID3D11Texture2D = std::ptr::null_mut();
        let hr = unsafe { device.CreateTexture2D(&texture_desc, std::ptr::null(), &mut texture) };
        hr_bail!(hr, "failed to create texture for render target");

        let mut view: *mut ID3D11RenderTargetView = std::ptr::null_mut();
        let hr =
            unsafe { device.CreateRenderTargetView(texture.cast(), std::ptr::null(), &mut view) };
        hr_bail!(hr, "failed to create render target view");

        Ok(Self {
            texture: texture.into(),
            view: view.into(),
        })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.OMSetRenderTargets(1, &self.view.as_ptr(), std::ptr::null_mut());
        }
    }

    pub fn unbind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.OMSetRenderTargets(1, std::ptr::null(), std::ptr::null_mut());
        }
    }

    pub fn clear(&mut self, ctx: &ID3D11DeviceContext, color: &[f32; 4]) {
        unsafe {
            ctx.ClearRenderTargetView(self.view.as_mut(), &color);
        }
    }
}

pub struct BlendState {
    state: ComPtr<ID3D11BlendState>,
}

impl BlendState {
    /// Blend state that blends alpha layers
    pub fn alpha_blend_state(device: &ID3D11Device) -> anyhow::Result<BlendState> {
        let blend_desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: 0,
            IndependentBlendEnable: 0,
            RenderTarget: [D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: 1,
                SrcBlend: D3D11_BLEND_SRC_ALPHA,
                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D11_BLEND_OP_ADD,
                SrcBlendAlpha: D3D11_BLEND_ONE,
                DestBlendAlpha: D3D11_BLEND_ZERO,
                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL as UINT8,
            }; 8],
        };

        let mut state = std::ptr::null_mut();
        let hr = unsafe { device.CreateBlendState(&blend_desc, &mut state) };

        hr_bail!(hr, "failed to create alpha blend state");

        Ok(Self {
            state: state.into(),
        })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        let blend_factor = [0.0f32; 4];
        let sample_mask = 0xffffffff;

        unsafe {
            ctx.OMSetBlendState(self.state.as_mut(), &blend_factor, sample_mask);
        }
    }
}

pub struct Texture {
    pub texture: ComPtr<ID3D11Texture2D>,
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
    ) -> anyhow::Result<ComPtr<ID3D11Texture2D>> {
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
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: data.as_ptr().cast(),
            SysMemPitch: width * 4, /* R8G8B8A8 = 4 bytes */
            SysMemSlicePitch: 0,
        };

        let mut texture = std::ptr::null_mut();
        let hr = unsafe { device.CreateTexture2D(&texture_desc, &init_data, &mut texture) };
        hr_bail!(hr, "failed to create texture");
        Ok(texture.into())
    }
}
