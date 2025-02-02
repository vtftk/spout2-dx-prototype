use anyhow::Context;
use windows::core::{Interface, InterfaceRef};
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11SamplerState, D3D11_COMPARISON_ALWAYS,
    D3D11_COMPARISON_NEVER, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_FILTER_MIN_MAG_MIP_POINT,
    D3D11_FLOAT32_MAX, D3D11_SAMPLER_DESC, D3D11_TEXTURE_ADDRESS_WRAP,
};

/// Texture sampler (Defines how textures are sampled and filtered)
pub struct SamplerState {
    state: ID3D11SamplerState,
}

impl SamplerState {
    pub fn linear(device: &ID3D11Device) -> anyhow::Result<SamplerState> {
        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressV: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressW: D3D11_TEXTURE_ADDRESS_WRAP,
            MipLODBias: 0.0,
            MaxAnisotropy: 1,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            BorderColor: [0.0, 0.0, 0.0, 0.0],
            MinLOD: 0.0,
            MaxLOD: D3D11_FLOAT32_MAX,
        };

        let mut state: Option<ID3D11SamplerState> = None;
        unsafe { device.CreateSamplerState(&sampler_desc, Some(&mut state))? };
        let state = state.context("failed to create linear sampler")?;

        Ok(Self { state })
    }

    pub fn pixelate(device: &ID3D11Device) -> anyhow::Result<SamplerState> {
        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_POINT,
            AddressU: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressV: D3D11_TEXTURE_ADDRESS_WRAP,
            AddressW: D3D11_TEXTURE_ADDRESS_WRAP,
            MipLODBias: 0.0,
            MaxAnisotropy: 0,
            ComparisonFunc: D3D11_COMPARISON_NEVER,
            BorderColor: [0.0, 0.0, 0.0, 0.0],
            MinLOD: 0.0,
            MaxLOD: D3D11_FLOAT32_MAX,
        };

        let mut state: Option<ID3D11SamplerState> = None;
        unsafe { device.CreateSamplerState(&sampler_desc, Some(&mut state))? };
        let state = state.context("failed to create pixelate sampler")?;

        Ok(Self { state })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        let sampler = self.state.clone();

        unsafe {
            ctx.PSSetSamplers(0, Some(&[Some(sampler)]));
        }
    }

    pub fn unbind(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.PSSetSamplers(0, None);
        }
    }
}
