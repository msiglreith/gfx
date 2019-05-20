use egl_sys;

use std::ptr;
use std::sync::Arc;
use std::os::raw::c_void;
use std::ffi::CString;

use hal;
use hal::image::{NumSamples, Size};
use hal::format::Format;
use crate::{native, QueueFamily, PhysicalDevice, Backend};

#[cfg(feature = "winit")]
use winit;

struct Entry {
    lib: libloading::Library,
    egl: egl_sys::egl::Egl,
}

impl Entry {
    pub fn new() -> Self {
        let lib = libloading::Library::new("libEGL.dll").unwrap();
        let egl = egl_sys::egl::Egl::load_with(|sym| unsafe {
            lib.get(CString::new(sym.as_bytes()).unwrap().as_bytes_with_nul())
                .map(|sym| *sym)
                .unwrap_or(std::ptr::null_mut())
        });

        Entry { lib, egl }
    }
}

lazy_static! {
    // Entry function pointers
    static ref EGL_ENTRY: Entry = Entry::new();
}

unsafe impl Sync for Entry { }

#[derive(Debug)]
pub struct Surface {
    surface: egl_sys::egl::types::EGLSurface,
}

// TODO: high -msiglreith
unsafe impl Send for Surface { }
unsafe impl Sync for Surface { }

pub struct Instance {
    display: egl_sys::egl::types::EGLDisplay,
}
// TODO: high -msiglreith
unsafe impl Send for Instance { }
unsafe impl Sync for Instance { }

impl Instance {
    pub fn create(name: &str, version: u32) -> Self {
        unsafe {
            let egl = &EGL_ENTRY.egl;
            let display = egl.GetDisplay(egl_sys::egl::DEFAULT_DISPLAY);

            let mut major = 0;
            let mut minor = 0;

            dbg!(EGL_ENTRY.egl.Initialize(display, &mut major, &mut minor));
            dbg!((major, minor));

            let mut config = ptr::null();
            let mut num_configs = 0;
            let attribs = [
                egl_sys::egl::SURFACE_TYPE, egl_sys::egl::PBUFFER_BIT,
                egl_sys::egl::NONE
            ];
            EGL_ENTRY.egl.ChooseConfig(
                display,
                attribs.as_ptr() as *const _,
                &mut config as *mut _ as *mut _,
                1,
                &mut num_configs,
            );

            let attribs = [
                egl_sys::egl::CONTEXT_CLIENT_VERSION, 3,
                egl_sys::egl::NONE
            ];
            let context = dbg!(EGL_ENTRY.egl.CreateContext(display, config, egl_sys::egl::NO_CONTEXT, attribs.as_ptr() as *const _));

            let attribs = [
                egl_sys::egl::WIDTH, 1,
                egl_sys::egl::HEIGHT, 1,
                egl_sys::egl::NONE
            ];
            let pbuffer = dbg!(EGL_ENTRY.egl.CreatePbufferSurface(display, config, attribs.as_ptr() as *const _));

            dbg!(EGL_ENTRY.egl.MakeCurrent(display, pbuffer, pbuffer, context));

            Instance { display }
        }
    }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn create_surface_from_xlib(
        &self
    ) -> Surface {
        unimplemented!()
    }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn create_surface_from_xcb(
        &self
    ) -> Surface {
        unimplemented!()
    }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn create_surface_from_wayland(
        &self, display: *mut c_void, surface: *mut c_void, width: Size, height: Size
    ) -> Surface {
        unimplemented!()
    }

    #[cfg(target_os = "android")]
    pub fn create_surface_android(
        &self
    ) -> Surface {
        unimplemented!()
    }

    #[cfg(windows)]
    pub fn create_surface_from_hwnd(
        &self, hwnd: *mut c_void
    ) -> Surface {
        let mut config = ptr::null();
        let mut num_configs = 0;

        let attribs = [
            egl_sys::egl::SURFACE_TYPE, egl_sys::egl::WINDOW_BIT,
            egl_sys::egl::NONE
        ];
        unsafe {
            if EGL_ENTRY.egl.ChooseConfig(
            self.display,
            attribs.as_ptr() as *const _,
            &mut config as *mut _ as *mut _,
            1,
            &mut num_configs,
        ) != 0
        {
            let surface = dbg!(EGL_ENTRY.egl.CreateWindowSurface(self.display, config, hwnd as _, ptr::null()));
            Surface { surface }
        } else {
            unimplemented!()
        }
        }
    }

    #[cfg(feature = "winit")]
    pub fn create_surface(&self, window: &winit::Window) -> Surface {
    //     #[cfg(all(unix, not(target_os = "android")))]
    //     {
    //         use winit::os::unix::WindowExt;

    //         if self.extensions.contains(&vk::VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME) {
    //             if let Some(display) = window.get_wayland_display() {
    //                 let display: *mut c_void = display as *mut _;
    //                 let surface: *mut c_void = window.get_wayland_surface().unwrap() as *mut _;
    //                 let px = window.get_inner_size().unwrap();
    //                 return self.create_surface_from_wayland(display, surface, px.width as _, px.height as _);
    //             }
    //         }
    //         if self.extensions.contains(&vk::VK_KHR_XLIB_SURFACE_EXTENSION_NAME) {
    //             if let Some(display) = window.get_xlib_display() {
    //                 let window = window.get_xlib_window().unwrap();
    //                 return self.create_surface_from_xlib(display as _, window);
    //             }
    //         }
    //         panic!("The OpenGL driver does not support surface creation!");
    //     }
    //     #[cfg(target_os = "android")]
    //     {
    //         use winit::os::android::WindowExt;
    //         let logical_size = window.get_inner_size().unwrap();
    //         let width = logical_size.width * window.get_hidpi_factor();
    //         let height = logical_size.height * window.get_hidpi_factor();
    //         self.create_surface_android(window.get_native_window(), width as _, height as _)
    //     }
    // }

        #[cfg(windows)]
        {
            use winapi::um::libloaderapi::GetModuleHandleW;
            use winit::os::windows::WindowExt;

            let hwnd = window.get_hwnd();
            self.create_surface_from_hwnd(hwnd as *mut _)
        }
    }
}

impl hal::Instance for Instance {
    type Backend = Backend;

    fn enumerate_adapters(&self) -> Vec<hal::Adapter<Backend>> {
        let adapter = PhysicalDevice::new_adapter(|s| unsafe {
            let symbol = CString::new(s.as_bytes()).unwrap();
            EGL_ENTRY.egl.GetProcAddress(symbol.as_ptr()) as *const _
        });
        vec![adapter]
    }
}

impl hal::Surface<Backend> for Surface {
    fn kind(&self) -> hal::image::Kind {
        unimplemented!()
    }

    fn compatibility(
        &self, physical_device: &PhysicalDevice
    ) -> (hal::SurfaceCapabilities, Option<Vec<Format>>, Vec<hal::PresentMode>) {
        unimplemented!()
    }

    fn supports_queue_family(&self, queue_family: &QueueFamily) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Swapchain {
}


impl hal::Swapchain<Backend> for Swapchain {
    unsafe fn acquire_image(
        &mut self, _timeout_ns: u64,
        _semaphore: Option<&native::Semaphore>,
        _fence: Option<&native::Fence>,
    ) -> Result<(hal::SwapImageIndex, Option<hal::window::Suboptimal>), hal::AcquireError> {
        unimplemented!()
    }
}
