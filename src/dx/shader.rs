use std::ffi::CString;
use winapi::{
    shared::winerror::FAILED,
    um::{
        d3d11::{
            ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader,
            ID3D11Resource, ID3D11ShaderResourceView, ID3D11VertexShader, D3D11_INPUT_ELEMENT_DESC,
        },
        d3dcommon::ID3D10Blob,
        d3dcompiler::{D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS},
    },
};

use crate::{com::ComPtr, hr_bail};

/// Compiled shader blob
#[derive(Clone)]
#[repr(transparent)]
pub struct ShaderBlob(pub ComPtr<ID3D10Blob>);

impl ShaderBlob {
    pub fn compile(src: &[u8], target: &str, entrypoint: &str) -> anyhow::Result<ShaderBlob> {
        let mut blob = std::ptr::null_mut();

        let target_c = CString::new(target)?;
        let entrypoint_c = CString::new(entrypoint)?;

        let hr = unsafe {
            D3DCompile(
                src.as_ptr().cast(),
                src.len(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                entrypoint_c.as_ptr(),
                target_c.as_ptr(),
                D3DCOMPILE_ENABLE_STRICTNESS,
                0,
                &mut blob,
                std::ptr::null_mut(),
            )
        };
        if FAILED(hr) {
            return Err(anyhow::anyhow!(
                "failed to compile shader {target} {entrypoint}"
            ));
        }

        Ok(ShaderBlob(blob.into()))
    }
}

pub struct PixelShader {
    pub blob: ShaderBlob,
    pub shader: *mut ID3D11PixelShader,
}

impl PixelShader {
    pub fn create(device: &ID3D11Device, blob: ShaderBlob) -> anyhow::Result<PixelShader> {
        let mut shader = std::ptr::null_mut();
        let blob_ref = blob.0.as_ref();

        let hr = unsafe {
            device.CreatePixelShader(
                blob_ref.GetBufferPointer(),
                blob_ref.GetBufferSize(),
                std::ptr::null_mut(),
                &mut shader,
            )
        };

        if FAILED(hr) {
            return Err(anyhow::anyhow!("failed to create vertex shader"));
        }

        Ok(PixelShader { blob, shader })
    }

    pub fn set_shader(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.PSSetShader(self.shader, std::ptr::null_mut(), 0);
        }
    }
}

pub struct VertexShader {
    pub blob: ShaderBlob,
    pub shader: *mut ID3D11VertexShader,
}

impl VertexShader {
    pub fn create(device: &ID3D11Device, blob: ShaderBlob) -> anyhow::Result<VertexShader> {
        let mut shader = std::ptr::null_mut();
        let blob_ref = blob.0.as_ref();
        let hr = unsafe {
            device.CreateVertexShader(
                blob_ref.GetBufferPointer(),
                blob_ref.GetBufferSize(),
                std::ptr::null_mut(),
                &mut shader,
            )
        };

        hr_bail!(hr, "failed to create vertex shader");

        Ok(VertexShader { blob, shader })
    }

    pub fn set_shader(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.VSSetShader(self.shader, std::ptr::null_mut(), 0);
        }
    }
}

pub struct ShaderResourceView {
    view: ComPtr<ID3D11ShaderResourceView>,
}

impl ShaderResourceView {
    pub fn create_from_texture(
        device: &ID3D11Device,
        texture: &mut ID3D11Resource,
    ) -> anyhow::Result<ShaderResourceView> {
        let mut srv = std::ptr::null_mut();
        let hr = unsafe { device.CreateShaderResourceView(texture, std::ptr::null(), &mut srv) };

        hr_bail!(hr, "failed to create shader resource view");

        Ok(Self { view: srv.into() })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.PSSetShaderResources(0, 1, &self.view.as_ptr());
        }
    }

    pub fn unbind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.PSSetShaderResources(0, 1, std::ptr::null());
        }
    }
}

pub struct ShaderInputLayout {
    layout: ComPtr<ID3D11InputLayout>,
}

impl ShaderInputLayout {
    pub fn create(
        device: &ID3D11Device,
        layout_desc: &[D3D11_INPUT_ELEMENT_DESC],
        shader_blob: ShaderBlob,
    ) -> anyhow::Result<ShaderInputLayout> {
        // Create input layout
        let mut layout = std::ptr::null_mut();

        let blob = shader_blob.0.as_ref();

        let hr = unsafe {
            device.CreateInputLayout(
                layout_desc.as_ptr(),
                layout_desc.len() as _,
                blob.GetBufferPointer(),
                blob.GetBufferSize(),
                &mut layout,
            )
        };

        hr_bail!(hr, "failed to create shader input layout");

        Ok(Self {
            layout: layout.into(),
        })
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.IASetInputLayout(self.layout.as_mut());
        }
    }
}
