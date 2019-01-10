use openvr_sys as sys;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

pub mod enums;

pub use self::enums::*;

#[inline]
fn phantom_data<T>(_: T) -> PhantomData<T> {
    PhantomData
}

pub struct RawInitError(u32);

pub enum InitError {
    None,
}

static INITIALIZED: AtomicBool = ATOMIC_BOOL_INIT;

pub struct Context {}

impl Context {
    pub fn new(ty: sys::EVRApplicationType) -> Result<Self, u32> {
        if INITIALIZED.compare_and_swap(false, true, Ordering::Acquire) {
            panic!("OpenVR can only be initialized once.");
        }

        let mut error = sys::EVRInitError_VRInitError_None;
        unsafe {
            sys::VR_InitInternal(&mut error, ty as sys::EVRApplicationType);
        }

        if error == sys::EVRInitError_VRInitError_None {
            Ok(Context {})
        } else {
            INITIALIZED.store(false, Ordering::Release);
            Err(error)
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::VR_ShutdownInternal();
        }
        INITIALIZED.store(false, Ordering::Release);
    }
}

pub struct System<'context> {
    pub fn_table: sys::VR_IVRSystem_FnTable,
    _context: PhantomData<&'context Context>,
}

impl<'context> System<'context> {
    pub fn new(context: &'context Context) -> Result<Self, sys::EVRInitError> {
        let mut magic = Vec::from(b"FnTable:".as_ref());
        magic.extend(sys::IVRSystem_Version.as_ref());

        unsafe {
            let mut err = sys::EVRInitError_VRInitError_None;
            let fn_table = sys::VR_GetGenericInterface(magic.as_ptr() as *const c_char, &mut err)
                as *const sys::VR_IVRSystem_FnTable;
            if err == sys::EVRInitError_VRInitError_None {
                Ok(System {
                    fn_table: {
                        if fn_table.is_null() {
                            panic!("Unexpected null pointer.");
                        }
                        *fn_table
                    },
                    _context: phantom_data(context),
                })
            } else {
                Err(err)
            }
        }
    }
}

pub struct Compositor<'context> {
    pub fn_table: sys::VR_IVRCompositor_FnTable,
    _context: PhantomData<&'context Context>,
}

impl<'context> Compositor<'context> {
    pub fn new(context: &'context Context) -> Result<Self, sys::EVRInitError> {
        let mut magic = Vec::from(b"FnTable:".as_ref());
        magic.extend(sys::IVRCompositor_Version.as_ref());

        unsafe {
            let mut err = sys::EVRInitError_VRInitError_None;
            let fn_table = sys::VR_GetGenericInterface(magic.as_ptr() as *const c_char, &mut err)
                as *const sys::VR_IVRCompositor_FnTable;
            if err == sys::EVRInitError_VRInitError_None {
                Ok(Compositor {
                    fn_table: {
                        if fn_table.is_null() {
                            panic!("Unexpected null pointer.");
                        }
                        *fn_table
                    },
                    _context: phantom_data(context),
                })
            } else {
                Err(err)
            }
        }
    }
}
