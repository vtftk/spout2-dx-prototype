use anyhow::Context;
use windows::{
    core::PCSTR,
    Win32::Graphics::{
        Direct3D::{
            Fxc::{D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS},
            ID3DBlob,
        },
        Direct3D11::{
            ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader,
            ID3D11Resource, ID3D11ShaderResourceView, ID3D11VertexShader, D3D11_INPUT_ELEMENT_DESC,
        },
    },
};

/// Compiled shader blob
#[derive(Clone)]
#[repr(transparent)]
pub struct ShaderBlob(ID3DBlob);

impl ShaderBlob {
    pub fn compile(src: &[u8], target: PCSTR, entrypoint: PCSTR) -> anyhow::Result<ShaderBlob> {
        let mut blob: Option<ID3DBlob> = None;

        unsafe {
            D3DCompile(
                src.as_ptr().cast(),
                src.len(),
                None,
                None,
                None,
                entrypoint,
                target,
                D3DCOMPILE_ENABLE_STRICTNESS,
                0,
                &mut blob,
                None,
            )
            .context("failed to compile shader")?;
        };

        let blob = blob.context("failed to get compiled")?;

        Ok(ShaderBlob(blob))
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.0.GetBufferPointer().cast(), self.0.GetBufferSize())
        }
    }
}

pub struct PixelShader {
    pub shader: ID3D11PixelShader,
}

impl PixelShader {
    pub fn create(device: &ID3D11Device, shader_bytecode: &[u8]) -> anyhow::Result<PixelShader> {
        let mut shader = None;
        unsafe {
            device.CreatePixelShader(shader_bytecode, None, Some(&mut shader))?;
        };

        let shader = shader.context("failed to create vertex shader")?;

        Ok(PixelShader { shader })
    }

    pub fn set_shader(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.PSSetShader(Some(&self.shader), None);
        }
    }
}

pub struct VertexShader {
    pub shader: ID3D11VertexShader,
}

impl VertexShader {
    pub fn create(device: &ID3D11Device, bytecode: &[u8]) -> anyhow::Result<VertexShader> {
        let mut shader: Option<ID3D11VertexShader> = None;
        unsafe { device.CreateVertexShader(bytecode, None, Some(&mut shader))? };
        let shader = shader.context("failed to create vertex shader")?;

        Ok(VertexShader { shader })
    }

    pub fn set_shader(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.VSSetShader(Some(&self.shader), None);
        }
    }
}

pub struct ShaderResourceView {
    view: ID3D11ShaderResourceView,
}

impl ShaderResourceView {
    pub fn create_from_texture(
        device: &ID3D11Device,
        texture: &ID3D11Resource,
    ) -> anyhow::Result<ShaderResourceView> {
        let mut view: Option<ID3D11ShaderResourceView> = None;
        unsafe { device.CreateShaderResourceView(texture, None, Some(&mut view))? };
        let view = view.context("failed to create shader resource view")?;

        Ok(Self { view })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            let view = self.view.clone();
            ctx.PSSetShaderResources(0, Some(&[Some(view)]));
        }
    }

    pub fn unbind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.PSSetShaderResources(0, None);
        }
    }
}

pub struct ShaderInputLayout {
    layout: ID3D11InputLayout,
}

impl ShaderInputLayout {
    pub fn create(
        device: &ID3D11Device,
        layout_desc: &[D3D11_INPUT_ELEMENT_DESC],
        bytecode: &[u8],
    ) -> anyhow::Result<ShaderInputLayout> {
        // Create input layout
        let mut layout: Option<ID3D11InputLayout> = None;

        unsafe { device.CreateInputLayout(layout_desc, bytecode, Some(&mut layout))? };
        let layout = layout.context("failed to create shader input layout")?;

        Ok(Self { layout })
    }

    pub fn bind(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.IASetInputLayout(&self.layout);
        }
    }
}
