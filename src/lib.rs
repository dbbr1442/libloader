use std::{ffi::{c_double, c_float, c_int, c_long, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void, CStr, CString}, marker::PhantomData};
use libc::{RTLD_LAZY, RTLD_NOW, dlclose, dlerror, dlopen, dlsym};

//pub unsafe trait FromDL 
//where 
//    Self: Sized
//{
//    unsafe fn convert(ptr: *mut c_void) -> Self;
//}
//
//macro_rules! multi_impl_unsafe {
//    (for $($t:ty), +) => {
//        $(unsafe impl FromDL for $t {
//            unsafe fn convert(ptr: *mut c_void) -> Self {
//                let new = ptr as Self;
//                new
//            } 
//        })*
//    };
//}
//
//multi_impl_unsafe!{for 
//    *const c_double, *const c_float, *const c_int, *const c_long, *const isize, *const c_schar, *const c_short, *const usize, *const c_uchar, *const c_uint, *const c_ulong, *const c_ushort,
//    *mut c_double, *mut c_float, *mut c_int, *mut c_long, *mut isize, *mut c_schar, *mut c_short, *mut usize, *mut c_uchar, *mut c_uint, *mut c_ulong, *mut c_ushort
//}

pub struct CFunction<'lib, Args, Ret> {
    ptr: extern "C" fn(Args) -> Ret,
    _lib: &'lib DynamicLibrary,
}

impl<'lib, Args, Ret> CFunction<'lib, Args, Ret> {
    fn new(ptr: *mut c_void, lib: &'lib DynamicLibrary) -> Self {
        let new = unsafe { std::mem::transmute_copy::<*mut c_void, extern "C" fn(Args) -> Ret>(&ptr) };

        Self {
            _lib: lib, 
            ptr: new,
        }
    }

    pub unsafe fn call(&self, args: Args) -> Ret {
        (self.ptr)(args)
    }
}

pub struct DynamicLibrary {
    handle: *mut c_void,
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        unsafe { dlclose(self.handle) };
    }
}

impl DynamicLibrary {
    pub fn load(path: &str) -> Result<Self, String> {
        let path = CString::new(path).unwrap();
        unsafe { dlerror(); }
        let handle = unsafe { dlopen(path.as_ptr(), RTLD_LAZY) }; 
        if handle.is_null() {
            let reason = unsafe { dlerror() };
            if !reason.is_null() {
                let reason = unsafe { CStr::from_ptr(reason) }; 
                return Err(reason.to_string_lossy().to_string());
            }
        }

        let result = Self { handle, };
        Ok(result)
    }

    pub fn get_symbol<T>(&self, symbol: &str) -> Option<Symbol<'_, T>> {
        let ptr = unsafe { dlsym(self.handle, CString::new(symbol).unwrap().as_ptr()) };

        if ptr.is_null() {
            return None;
        }

        let ptr = ptr as *mut T;

        let result = Symbol::<'_, T> {
            _library: self,
            ptr,
        };

        Some(result)
    } 

    pub fn get_symbol_fn<Args, Ret>(&self, symbol: &str) -> Option<CFunction<'_, Args, Ret>> {
        let ptr = unsafe { dlsym(self.handle, CString::new(symbol).unwrap().as_ptr()) };

        if ptr.is_null() {
            return None;
        }

        let c_fn: CFunction<'_, Args, Ret> = CFunction::new(ptr, &self);

        Some(c_fn)
    }

    pub unsafe fn set_symbol_fn<Args, Ret>(&self, symbol: &str, new: extern "C" fn(Args) -> Ret) -> Result<(), String> {
        let ptr = unsafe { dlsym(self.handle, CString::new(symbol).unwrap().as_ptr()) };
        let ptr = ptr as *mut extern "C" fn(Args) -> Ret;

        if ptr.is_null() {
            return Err("Null pointer".to_string());
        }

        unsafe { *ptr = new };
        
        Ok(())

    }
}

pub struct Symbol<'lib, T> {
    _library: &'lib DynamicLibrary,
    ptr: *mut T,
}

impl<T> Symbol<'_, T> {
    pub fn get(&mut self) -> *mut T {
        self.ptr
    }
}
