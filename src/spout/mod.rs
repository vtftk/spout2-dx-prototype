use autocxx::prelude::*;
use ffi::{spoutDX, spoutDirectX, spoutSenderNames};
use std::{ffi::CString, pin::Pin};
use winapi::{
    shared::dxgiformat::DXGI_FORMAT,
    um::d3d11::{ID3D11Device, ID3D11Texture2D},
}; // use all the main autocxx functions

include_cpp! {
    #include "Spout.h"
    safety!(unsafe)
    generate!("spoutDX")
    generate!("spoutSenderNames")
    generate!("spoutDirectX")
    generate!("spoutFrameCount")
}

pub struct SpoutSender {
    handle: UniquePtr<ffi::spoutDX>,
}

impl SpoutSender {
    pub fn create() -> anyhow::Result<Self> {
        let handle: UniquePtr<spoutDX> = spoutDX::new().within_unique_ptr();
        if handle.is_null() {
            return Err(anyhow::anyhow!("Failed to get spout sender names handle"));
        }

        Ok(Self { handle })
    }

    pub fn set_sender_name<N: AsRef<str>>(&mut self, name: N) -> anyhow::Result<()> {
        let library = self.handle.as_mut().unwrap();
        let sender_name = CString::new(name.as_ref())?;

        unsafe {
            spoutDX::SetSenderName(library, sender_name.as_ptr());
        }

        Ok(())
    }

    pub fn set_sender_format(&mut self) -> anyhow::Result<()> {
        let library = self.handle.as_mut().unwrap();

        unsafe {
            spoutDX::SetSenderFormat(library, ffi::DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM);
        }

        Ok(())
    }
    pub fn open_directx11(&mut self, device: *mut ID3D11Device) -> anyhow::Result<()> {
        let library = self.handle.as_mut().unwrap();

        unsafe {
            spoutDX::OpenDirectX11(library, device.cast());
        }

        Ok(())
    }
    pub unsafe fn send_texture(&mut self, texture: *mut ID3D11Texture2D) -> anyhow::Result<()> {
        let library = self.handle.as_mut().unwrap();

        let value = spoutDX::SendTexture(library, texture.cast());

        Ok(())
    }
    pub fn hold_fps(&mut self, fps: c_int) -> anyhow::Result<()> {
        let library = self.handle.as_mut().unwrap();

        unsafe {
            spoutDX::HoldFps(library, fps);
        }

        Ok(())
    }
}
