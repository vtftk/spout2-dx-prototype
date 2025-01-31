use winapi::{
    ctypes::c_void,
    shared::{
        dxgiformat::{DXGI_FORMAT, DXGI_FORMAT_UNKNOWN},
        winerror::FAILED,
    },
    um::d3d11::{
        ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, D3D11_BIND_INDEX_BUFFER,
        D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT,
    },
};

use crate::com::ComPtr;

pub struct IndexBuffer {
    pub buffer: ComPtr<ID3D11Buffer>,
    pub format: DXGI_FORMAT,
    pub offset: u32,
}

impl IndexBuffer {
    pub fn create_from_array<T>(
        device: &ID3D11Device,
        data: &[T],
        format: DXGI_FORMAT,
    ) -> anyhow::Result<Self>
    where
        T: Sized,
    {
        unsafe {
            Self::create(
                device,
                data.as_ptr().cast(),
                std::mem::size_of_val(data) as u32,
                format,
                0,
            )
        }
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.IASetIndexBuffer(self.buffer.as_ptr(), self.format, self.offset);
        }
    }

    pub fn unbind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.IASetIndexBuffer(std::ptr::null_mut(), DXGI_FORMAT_UNKNOWN, 0);
        }
    }

    pub unsafe fn create(
        device: &ID3D11Device,
        data: *const c_void,
        size: u32,
        format: DXGI_FORMAT,
        offset: u32,
    ) -> anyhow::Result<Self> {
        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: size,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: data,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let mut buffer = std::ptr::null_mut();
        let hr = device.CreateBuffer(&buffer_desc, &init_data, &mut buffer);
        if FAILED(hr) {
            return Err(anyhow::anyhow!("Failed to create vertex buffer"));
        }

        Ok(Self {
            buffer: buffer.into(),
            format,
            offset,
        })
    }
}

pub struct VertexBuffer {
    pub buffer: ComPtr<ID3D11Buffer>,
    pub stride: u32,
    pub offset: u32,
}

impl VertexBuffer {
    pub fn create_from_array<T>(device: &ID3D11Device, data: &[T]) -> anyhow::Result<Self>
    where
        T: Sized,
    {
        unsafe {
            Self::create(
                device,
                data.as_ptr().cast(),
                std::mem::size_of_val(data) as u32,
                std::mem::size_of::<T>() as u32,
                0,
            )
        }
    }

    pub fn bind(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            let buffer = self.buffer.clone();
            ctx.IASetVertexBuffers(0, 1, &buffer.into(), &self.stride, &self.offset);
        }
    }

    pub fn unbind(&self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.IASetVertexBuffers(0, 0, std::ptr::null(), &0, &0);
        }
    }

    pub unsafe fn create(
        device: &ID3D11Device,
        data: *const c_void,
        size: u32,
        stride: u32,
        offset: u32,
    ) -> anyhow::Result<Self> {
        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: size,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: data,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let mut buffer = std::ptr::null_mut();
        let hr = device.CreateBuffer(&buffer_desc, &init_data, &mut buffer);
        if FAILED(hr) {
            return Err(anyhow::anyhow!("Failed to create vertex buffer"));
        }

        Ok(VertexBuffer {
            buffer: buffer.into(),
            stride,
            offset,
        })
    }
}
