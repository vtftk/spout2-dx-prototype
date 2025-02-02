use anyhow::Context;
use nalgebra::Vector2;
use windows::Win32::{
    Foundation::HMODULE,
    Graphics::{
        Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
        Direct3D11::{
            D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_SDK_VERSION, D3D11_VIEWPORT,
        },
    },
};

pub fn create_device_and_context() -> anyhow::Result<(ID3D11Device, ID3D11DeviceContext)> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            Default::default(),
            Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )?
    };

    let device = device.context("failed to create device")?;
    let context = context.context("failed to create context")?;

    Ok((device, context))
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
            ctx.RSSetViewports(Some(&[self.inner.clone()]));
        }
    }
}
