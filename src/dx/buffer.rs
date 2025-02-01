use std::marker::PhantomData;

use winapi::{
    ctypes::c_void,
    shared::{
        dxgiformat::{DXGI_FORMAT, DXGI_FORMAT_UNKNOWN},
        winerror::FAILED,
    },
    um::d3d11::{
        ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, D3D11_BIND_CONSTANT_BUFFER,
        D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC,
        D3D11_CPU_ACCESS_WRITE, D3D11_MAP_WRITE_DISCARD, D3D11_SUBRESOURCE_DATA,
        D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC,
    },
};

use crate::{com::ComPtr, hr_bail};

/// Constant buffer containing a specific type
pub struct ConstantBuffer<T> {
    pub buffer: ComPtr<ID3D11Buffer>,
    pub _type: PhantomData<T>,
}

impl<T> ConstantBuffer<T>
where
    T: Sized,
{
    pub fn create_default(device: &ID3D11Device) -> anyhow::Result<ConstantBuffer<T>>
    where
        T: Default,
    {
        Self::create(device, T::default())
    }

    pub fn create(device: &ID3D11Device, initial_data: T) -> anyhow::Result<ConstantBuffer<T>> {
        // Const buffers must be aligned to 16 byte boundary
        debug_assert!(
            std::mem::size_of::<T>() % 16 == 0,
            "constant buffer not aligned to 16 byte boundaries"
        );

        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: std::mem::size_of::<T>() as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: 0,
            StructureByteStride: 0,
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: (&initial_data) as *const _ as *const c_void,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let mut buffer = std::ptr::null_mut();
        let hr = unsafe { device.CreateBuffer(&buffer_desc, &init_data, &mut buffer) };
        hr_bail!(hr, "failed to create constant buffer");

        Ok(ConstantBuffer {
            buffer: buffer.into(),
            _type: PhantomData,
        })
    }

    /// Replaces the constant buffer data with the new data
    pub fn replace(&mut self, ctx: &ID3D11DeviceContext, new_data: &T) -> anyhow::Result<()> {
        unsafe {
            self.map(ctx, |data| {
                // Copy the new data into the mapped buffer
                std::ptr::copy_nonoverlapping(new_data, data.cast(), 1);
            })
        }
    }

    pub fn bind(&mut self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.VSSetConstantBuffers(0, 1, &self.buffer.as_ptr());
        }
    }

    /// Map the resource into memory and apply an action against it, un-maps the
    /// resource after the action returns
    pub unsafe fn map<F>(&mut self, ctx: &ID3D11DeviceContext, action: F) -> anyhow::Result<()>
    where
        F: FnOnce(*mut c_void),
    {
        // Inside the loop where you update the constant buffer:
        let mut mapped_resource = std::mem::zeroed();

        let resource = self.buffer.cast_as_mut();

        let hr = ctx.Map(
            resource,
            0,
            D3D11_MAP_WRITE_DISCARD,
            0,
            &mut mapped_resource,
        );
        hr_bail!(hr, "failed to map constant buffer");

        // Execute the action on the mapped data
        action(mapped_resource.pData);

        ctx.Unmap(resource, 0);

        Ok(())
    }
}

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
