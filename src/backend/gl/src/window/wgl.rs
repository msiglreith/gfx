
use std::ptr;
use std::sync::Arc;
use std::os::raw::c_void;
use std::ffi::{CStr, CString, OsStr};
use std::os::windows::ffi::OsStrExt;
use crate::hal::window::Extent2D;
use crate::hal::{self, format as f, image, memory, CompositeAlpha};

use hal::image::{NumSamples, Size};
use hal::format::Format;
use crate::{native, QueueFamily, PhysicalDevice, Backend};

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;
use std::mem;

pub mod wgl_sys {
    include!(concat!(env!("OUT_DIR"), "/wgl_bindings.rs"));
}

/// Functions that are not necessarily always available
pub mod wgl_extra {
    include!(concat!(env!("OUT_DIR"), "/wgl_extra_bindings.rs"));
}

#[link(name = "opengl32")]
extern "C" {}

#[cfg(feature = "winit")]
use winit;

pub(crate) struct Entry {
    hwnd: HWND,
    pub(crate) hdc: HDC,
    pub(crate) wgl: wgl_extra::Wgl,
    lib: HMODULE,
}

impl Entry {
    pub fn new() -> Self {
        unsafe {
            let mut class: WNDCLASSEXW = std::mem::zeroed();
        let instance = GetModuleHandleW(std::ptr::null());
        let class_name = OsStr::new("regl")
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect::<Vec<_>>();

        class.cbSize = std::mem::size_of::<WNDCLASSEXW>() as UINT;
        class.lpszClassName = class_name.as_ptr();
        class.lpfnWndProc = Some(DefWindowProcW);

        RegisterClassExW(&class);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null_mut(),
        );

        let hdc = GetDC(hwnd);

        let desc = PIXELFORMATDESCRIPTOR {
            nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 32,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 8,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 0,
            cStencilBits: 0,
            cAuxBuffers: 0,
            iLayerType: PFD_MAIN_PLANE,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0,
        };

        let format_id = unsafe { ChoosePixelFormat(hdc, &desc) };
        SetPixelFormat(hdc, format_id, &desc);

        let hglrc = wglCreateContext(hdc);

        println!("{:?}", (hwnd, hdc, format_id, hglrc));

        dbg!(wglMakeCurrent(hdc, hglrc));

        let name = OsStr::new("opengl32.dll")
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect::<Vec<_>>();

        let lib = dbg!(LoadLibraryW(name.as_ptr()));

        let wgl = wgl_extra::Wgl::load_with(|sym| {
            let sym = CString::new(sym.as_bytes()).unwrap();
            let addr = wgl_sys::GetProcAddress(sym.as_ptr()) as *const ();
            if !addr.is_null() {
                addr as *const _
            } else {
                GetProcAddress(lib, sym.as_ptr()) as *const _
            }
        });

        Entry { hwnd, hdc, wgl, lib }
        }
    }
}

lazy_static! {
    // Entry function pointers
    pub(crate) static ref WGL_ENTRY: Entry = Entry::new();
}

unsafe impl Sync for Entry { }

#[derive(Debug)]
pub struct Surface {
    pub(crate) hwnd: HWND,
}

// TODO: high -msiglreith
unsafe impl Send for Surface { }
unsafe impl Sync for Surface { }

pub struct Instance {
    pub(crate) ctxt: HGLRC,
}
// TODO: high -msiglreith
unsafe impl Send for Instance { }
unsafe impl Sync for Instance { }

impl Instance {
    pub fn create(name: &str, version: u32) -> Self {
        // unsafe {
        //     let egl = &EGL_ENTRY.egl;
        //     let display = egl.GetDisplay(egl_sys::egl::DEFAULT_DISPLAY);

        //     let mut major = 0;
        //     let mut minor = 0;

        //     dbg!(EGL_ENTRY.egl.Initialize(display, &mut major, &mut minor));
        //     dbg!((major, minor));

        //     let mut config = ptr::null();
        //     let mut num_configs = 0;
        //     let attribs = [
        //         egl_sys::egl::SURFACE_TYPE, egl_sys::egl::PBUFFER_BIT,
        //         egl_sys::egl::NONE
        //     ];
        //     EGL_ENTRY.egl.ChooseConfig(
        //         display,
        //         attribs.as_ptr() as *const _,
        //         &mut config as *mut _ as *mut _,
        //         1,
        //         &mut num_configs,
        //     );

        //     let attribs = [
        //         egl_sys::egl::CONTEXT_CLIENT_VERSION, 3,
        //         egl_sys::egl::NONE
        //     ];
        //     let context = dbg!(EGL_ENTRY.egl.CreateContext(display, config, egl_sys::egl::NO_CONTEXT, attribs.as_ptr() as *const _));

        //     let attribs = [
        //         egl_sys::egl::WIDTH, 1,
        //         egl_sys::egl::HEIGHT, 1,
        //         egl_sys::egl::NONE
        //     ];
        //     let pbuffer = dbg!(EGL_ENTRY.egl.CreatePbufferSurface(display, config, attribs.as_ptr() as *const _));

        //     dbg!(EGL_ENTRY.egl.MakeCurrent(display, pbuffer, pbuffer, context));

        //     Instance { display }
        // }

        unsafe {

        //     let mut wgl_configs = vec![0; 1];
        //     let mut num_pixel_formats = 0;

        //     let mut attribs_int = Vec::<i32>::new();
        // let mut attribs_float = Vec::new();

        // attribs_int.push(0);
        //             attribs_float.push(0.0);

        // WGL_ENTRY.wgl.ChoosePixelFormatARB(
        //     WGL_ENTRY.hdc as *const _,
        //     attribs_int.as_ptr(),
        //     attribs_float.as_ptr(),
        //     wgl_configs.len() as _,
        //     wgl_configs.as_mut_ptr(),
        //     &mut num_pixel_formats,
        // );

        // let pbuffer =
        //     dbg!(WGL_ENTRY
        //         .wgl
        //         .CreatePbufferARB(WGL_ENTRY.hdc as *const _, wgl_configs[0] as _, 1, 1, ptr::null()));


        // let pbuffer_hdc = WGL_ENTRY.wgl.GetPbufferDCARB(pbuffer);
            let ctxt = dbg!(WGL_ENTRY.wgl.CreateContextAttribsARB(
            WGL_ENTRY.hdc as *const _,
            ptr::null(),
            ptr::null()
        ));

        dbg!(wglMakeCurrent(WGL_ENTRY.hdc as *mut _, ctxt as *mut _));

        Instance { ctxt: ctxt as *mut _ }
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
        // let mut config = ptr::null();
        // let mut num_configs = 0;

        // let attribs = [
        //     egl_sys::egl::SURFACE_TYPE, egl_sys::egl::WINDOW_BIT,
        //     egl_sys::egl::NONE
        // ];
        // unsafe {
        //     if EGL_ENTRY.egl.ChooseConfig(
        //     self.display,
        //     attribs.as_ptr() as *const _,
        //     &mut config as *mut _ as *mut _,
        //     1,
        //     &mut num_configs,
        // ) != 0
        // {
        //     let surface = dbg!(EGL_ENTRY.egl.CreateWindowSurface(self.display, config, hwnd as _, ptr::null()));
        //     Surface { surface }
        // } else {
        //     unimplemented!()
        // }
        // }

        Surface { hwnd: hwnd as *mut _}
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
        let adapter = PhysicalDevice::new_adapter(self.ctxt as *const _, |s| unsafe {
            let sym = CString::new(s.as_bytes()).unwrap();
            let addr = wgl_sys::GetProcAddress(sym.as_ptr()) as *const ();
            if !addr.is_null() {
                addr as *const _
            } else {
                GetProcAddress(WGL_ENTRY.lib, sym.as_ptr()) as *const _
            }
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
        let mut rect: RECT = unsafe { mem::uninitialized() };
        unsafe { GetClientRect(self.hwnd, &mut rect); }
        let extent = hal::window::Extent2D {
            width: (rect.right - rect.left) as _,
            height: (rect.bottom - rect.top) as _,
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
            composite_alpha: CompositeAlpha::OPAQUE, //TODO
        };
        let present_modes = vec![
            hal::PresentMode::Fifo, //TODO
        ];

        (caps, Some(vec![f::Format::Rgba8Srgb, f::Format::Bgra8Srgb]), present_modes)
    }

    fn supports_queue_family(&self, queue_family: &QueueFamily) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Swapchain {
    pub(crate) fbos: Vec<gl::types::GLuint>,
    pub(crate) ctxt: HGLRC,
    pub(crate) hdc: HDC,
    pub(crate) extent: Extent2D,
}

// TODO
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
