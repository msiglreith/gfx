
use std::ptr;
use std::sync::Arc;
use std::os::raw::c_void;
use std::ffi::CString;

use hal;
use hal::image::{self, NumSamples, Size};
use hal::format::Format;
use crate::{native, QueueFamily, PhysicalDevice, Backend};
use hal::window::Extent2D;

#[cfg(feature = "winit")]
use winit;



pub mod egl_sys {
    use std::os::raw;

    pub type NativeDisplayType = EGLNativeDisplayType;
    pub type NativePixmapType = EGLNativePixmapType;
pub type NativeWindowType = EGLNativeWindowType;


    pub type EGLNativeDisplayType = *const raw::c_void;
    pub type EGLNativePixmapType = *const raw::c_void; // FIXME: egl_native_pixmap_t instead

    #[cfg(target_os = "windows")]
    pub type EGLNativeWindowType = winapi::shared::windef::HWND;
    #[cfg(target_os = "linux")]
    pub type EGLNativeWindowType = *const raw::c_void;
    #[cfg(target_os = "android")]
    pub type EGLNativeWindowType = *const raw::c_void;
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub type EGLNativeWindowType = *const raw::c_void;


    pub type khronos_utime_nanoseconds_t = khronos_uint64_t;
    pub type khronos_uint64_t = u64;
    pub type khronos_ssize_t = raw::c_long;
    pub type EGLint = i32;

    include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));
}

pub(crate) struct Entry {
    lib: libloading::Library,
    pub(crate) egl: egl_sys::Egl,
    pub(crate) display: egl_sys::types::EGLDisplay,
    pub(crate) context: egl_sys::types::EGLContext,
    pub(crate) pbuffer: egl_sys::types::EGLSurface,
}

impl Entry {
    pub fn new() -> Self {unsafe {
        #[cfg(windows)]
        let lib = libloading::Library::new("libEGL.dll").unwrap();
        #[cfg(unix)]
        let lib = libloading::Library::new("libEGL.so.1").unwrap();

        let egl = egl_sys::Egl::load_with(|sym| unsafe {
            lib.get(CString::new(sym.as_bytes()).unwrap().as_bytes_with_nul())
                .map(|sym| *sym)
                .unwrap_or(std::ptr::null_mut())
        });

        let display = dbg!(egl.GetDisplay(egl_sys::DEFAULT_DISPLAY));

        let mut major = 0;
        let mut minor = 0;

        dbg!(egl.Initialize(display, &mut major, &mut minor));
        dbg!((major, minor));

        dbg!(egl.BindAPI(egl_sys::OPENGL_API));

        let mut config = ptr::null();
        let mut num_configs = 0;
        let attribs = [
            egl_sys::SURFACE_TYPE, egl_sys::PBUFFER_BIT,
            egl_sys::NONE
        ];
        egl.ChooseConfig(
            display,
            attribs.as_ptr() as *const _,
            &mut config as *mut _ as *mut _,
            1,
            &mut num_configs,
        );

        let attribs = [
            egl_sys::CONTEXT_CLIENT_VERSION, 4,
            egl_sys::NONE
        ];
        let context = dbg!(egl.CreateContext(display, config, egl_sys::NO_CONTEXT, attribs.as_ptr() as *const _));

        let attribs = [
            egl_sys::WIDTH, 1,
            egl_sys::HEIGHT, 1,
            egl_sys::NONE
        ];
        let pbuffer = dbg!(egl.CreatePbufferSurface(display, config, attribs.as_ptr() as *const _));

        dbg!(egl.MakeCurrent(display, pbuffer, pbuffer, context));

        Entry { lib, egl, display, context, pbuffer }
    }}
}

lazy_static! {
    // Entry function pointers
    pub(crate) static ref EGL_ENTRY: Entry = Entry::new();
}

unsafe impl Sync for Entry { }

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Surface {
    pub(crate) display: egl_sys::types::EGLDisplay,
    pub(crate) surface: egl_sys::types::EGLSurface,
    #[derivative(Debug="ignore")]
    window: wayland_client::egl::WlEglSurface,
}

// TODO: high -msiglreith
unsafe impl Send for Surface { }
unsafe impl Sync for Surface { }

pub struct Instance {

}
// TODO: high -msiglreith
unsafe impl Send for Instance { }
unsafe impl Sync for Instance { }

impl Instance {
    pub fn create(name: &str, version: u32) -> Self {
        unsafe {
            Instance { }
        }
    }

    // #[cfg(all(unix, not(target_os = "android")))]
    // pub fn create_surface_from_xlib(
    //     &self,
    //     window: *const (),
    // ) -> Surface {
    //     let mut config = ptr::null();
    //     let mut num_configs = 0;

    //     let attribs = [
    //         egl_sys::SURFACE_TYPE, egl_sys::WINDOW_BIT,
    //         egl_sys::NONE
    //     ];
    //     unsafe {
    //         if EGL_ENTRY.egl.ChooseConfig(
    //             EGL_ENTRY.display,
    //             attribs.as_ptr() as *const _,
    //             &mut config as *mut _ as *mut _,
    //             1,
    //             &mut num_configs,
    //         ) != 0
    //         {
    //             let surface = dbg!(EGL_ENTRY.egl.CreateWindowSurface(EGL_ENTRY.display, config, window as _, ptr::null()));
    //             Surface { display: EGL_ENTRY.display, surface }
    //         } else {
    //             unimplemented!()
    //         }
    //     }
    // }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn create_surface_from_xcb(
        &self
    ) -> Surface {
        unimplemented!()
    }

    #[cfg(all(unix, not(target_os = "android")))]
    pub fn create_surface_from_wayland(
        &self, display: *mut c_void, surface: *mut c_void
    ) -> Surface {
        let mut config = ptr::null();
        let mut num_configs = 0;

        let egl_display = unsafe { EGL_ENTRY.egl.GetDisplay(display) };

        let mut major = 0;
        let mut minor = 0;

        unsafe {
        dbg!(EGL_ENTRY.egl.Initialize(egl_display, &mut major, &mut minor));
        dbg!((major, minor));
        }


        let window = unsafe { wayland_client::egl::WlEglSurface::new_from_raw(surface as _, 800, 600) };

        let attribs = [
            egl_sys::SURFACE_TYPE, egl_sys::WINDOW_BIT,
            egl_sys::NONE
        ];
        unsafe {
            if EGL_ENTRY.egl.ChooseConfig(
                egl_display,
                attribs.as_ptr() as *const _,
                &mut config as *mut _ as *mut _,
                1,
                &mut num_configs,
            ) != 0
            {
                
                let surface = dbg!(EGL_ENTRY.egl.CreateWindowSurface(egl_display, config, dbg!(window.ptr()), ptr::null()));
                dbg!(EGL_ENTRY.egl.GetError());
                Surface { display: egl_display, surface, window }
            } else {
                unimplemented!()
            }
        }
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
            egl_sys::SURFACE_TYPE, egl_sys::WINDOW_BIT,
            egl_sys::NONE
        ];
        unsafe {
            if EGL_ENTRY.egl.ChooseConfig(
                EGL_ENTRY.display,
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

        #[cfg(all(unix, not(target_os = "android")))]
        {
            use winit::os::unix::WindowExt;

            // if let Some(display) = window.get_xlib_display() {
            //     let window = window.get_xlib_window().unwrap();
            //     return self.create_surface_from_xlib(window as _);
            // }

            if let Some(display) = window.get_wayland_display() {
                let display: *mut c_void = display as *mut _;
                let surface: *mut c_void = window.get_wayland_surface().unwrap() as *mut _;
                return self.create_surface_from_wayland(display, surface);
            }

            panic!("The OpenGL driver does not support surface creation!");
        }
    }
}

impl hal::Instance for Instance {
    type Backend = Backend;

    fn enumerate_adapters(&self) -> Vec<hal::Adapter<Backend>> {
        let adapter = PhysicalDevice::new_adapter((), |s| unsafe {
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
        let extent = hal::window::Extent2D {
            width: 800, // TODo
            height: 600, // TODO
        };

        let caps = hal::SurfaceCapabilities {
            image_count: 2..3,
            current_extent: Some(extent),
            extents: extent..hal::window::Extent2D {
                width: extent.width + 1,
                height: extent.height + 1,
            },
            max_image_layers: 1,
            usage: image::Usage::COLOR_ATTACHMENT | image::Usage::TRANSFER_SRC,
            composite_alpha: hal::CompositeAlpha::OPAQUE, //TODO
        };
        let present_modes = vec![
            hal::PresentMode::Fifo, //TODO
        ];

        (caps, Some(vec![Format::Rgba8Srgb, Format::Bgra8Srgb]), present_modes)
    }

    fn supports_queue_family(&self, queue_family: &QueueFamily) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Swapchain {
    pub(crate) display: egl_sys::types::EGLDisplay,
    pub(crate) surface: egl_sys::types::EGLSurface,
    pub(crate) fbos: Vec<gl::types::GLuint>,
    pub(crate) extent: Extent2D,
    pub(crate) ctxt: egl_sys::types::EGLContext,
}

// TODO: high -msiglreith
unsafe impl Send for Swapchain { }
unsafe impl Sync for Swapchain { }

impl hal::Swapchain<Backend> for Swapchain {
    unsafe fn acquire_image(
        &mut self, _timeout_ns: u64,
        _semaphore: Option<&native::Semaphore>,
        _fence: Option<&native::Fence>,
    ) -> Result<(hal::SwapImageIndex, Option<hal::window::Suboptimal>), hal::AcquireError> {
        Ok((0, None)) // TODO
    }
}
