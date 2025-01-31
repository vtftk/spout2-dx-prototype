use std::{
    convert, fmt,
    ops::{Deref, DerefMut},
    ptr,
};
use winapi::{um::unknwnbase::IUnknown, Interface};

#[repr(transparent)]
pub struct ComPtr<T>(ptr::NonNull<T>);

impl<T> From<*mut T> for ComPtr<T> {
    fn from(value: *mut T) -> Self {
        Self::new(value)
    }
}

impl<T> ComPtr<T> {
    pub fn new(raw_pointer: *mut T) -> Self {
        let ptr =
            ptr::NonNull::new(raw_pointer).expect("Tried to create `ComPtr` from a null pointer");

        ComPtr(ptr)
    }

    pub unsafe fn new_unchecked(raw_pointer: *mut T) -> Self {
        let ptr = ptr::NonNull::new_unchecked(raw_pointer);

        ComPtr(ptr)
    }

    pub fn dangling() -> Self {
        ComPtr(ptr::NonNull::dangling())
    }

    pub fn query_interface<U>(&self) -> Option<ComPtr<U>>
    where
        U: Interface,
    {
        let mut ptr = ptr::null_mut();

        unsafe {
            self.as_unknown().QueryInterface(&U::uuidof(), &mut ptr);
        }

        ptr::NonNull::new(ptr as *mut U).map(|ptr| ComPtr(ptr))
    }

    pub fn upcast<U: Interface>(&self) -> &ComPtr<U>
    where
        T: Deref<Target = U>,
    {
        unsafe { &*(self as *const _ as *const _) }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }

    pub unsafe fn as_ptr(&mut self) -> *mut T {
        self.0.as_ptr()
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.as_ptr() }
    }

    pub fn cast_as_mut<C>(&mut self) -> &mut C {
        unsafe { &mut *self.0.as_ptr().cast::<C>() }
    }

    fn as_unknown(&self) -> &IUnknown {
        unsafe { &*(self.0.as_ptr().cast()) }
    }
}

impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            self.as_unknown().Release();
        }
    }
}

impl<T> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.as_unknown().AddRef();
        }

        ComPtr(self.0)
    }
}

impl<T> fmt::Pointer for ComPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.0)
    }
}

impl<T> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<T> From<ComPtr<T>> for *mut T {
    fn from(val: ComPtr<T>) -> Self {
        let ptr = val.0.as_ptr();
        std::mem::forget(val);
        ptr
    }
}
