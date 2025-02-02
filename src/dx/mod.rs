use std::{
    ffi::c_void,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

use windows::{
    core::{IUnknown, Interface, Ref},
    Win32::Graphics::Direct3D11::ID3D11RenderTargetView,
};

pub mod buffer;
pub mod device;
pub mod sampler;
pub mod shader;
pub mod texture;

/// Clone a value without incrementing the reference count
/// (Used by functions that required a owned value but don't actually decrease the ref count see: https://github.com/microsoft/windows-rs/issues/1339)
pub unsafe fn leak_copy_com<T: Interface + Sized>(ptr_ref: &T) -> T {
    ptr_ref.to_ref().cast().unwrap()
}
