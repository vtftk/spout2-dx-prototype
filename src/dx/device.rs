use nalgebra::Vector2;
use winapi::um::{
    d3d11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_SDK_VERSION, D3D11_VIEWPORT,
    },
    d3dcommon::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
};

use crate::{com::ComPtr, hr_bail};

pub fn create_device_and_context(
) -> anyhow::Result<(ComPtr<ID3D11Device>, ComPtr<ID3D11DeviceContext>)> {
    let feature_level = D3D_FEATURE_LEVEL_11_0;

    let mut device: *mut ID3D11Device = std::ptr::null_mut();
    let mut context: *mut ID3D11DeviceContext = std::ptr::null_mut();

    let hr = unsafe {
        D3D11CreateDevice(
            std::ptr::null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            std::ptr::null_mut(),
            0,
            &feature_level,
            1,
            D3D11_SDK_VERSION,
            &mut device,
            std::ptr::null_mut(),
            &mut context,
        )
    };

    hr_bail!(hr, "failed to create D3D11 device and context");

    Ok((device.into(), context.into()))
}

pub struct Viewport {
    inner: D3D11_VIEWPORT,
}

impl Viewport {
    pub fn new(size: Vector2<f32>, depth: Vector2<f32>) -> Viewport {
        Viewport {
            inner: D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: size.x,
                Height: size.y,
                MinDepth: depth.x,
                MaxDepth: depth.y,
            },
        }
    }

    pub fn bind(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.RSSetViewports(1, &self.inner);
        }
    }
}
