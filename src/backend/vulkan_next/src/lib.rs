
#[macro_use]
extern crate log;
extern crate shared_library;
extern crate gfx_core_next as core;
extern crate vk_sys as vk;
extern crate spirv_utils;
#[macro_use]
extern crate lazy_static;

extern crate winit;
extern crate gfx_device_vulkan as device_vulkan;

#[cfg(unix)]
extern crate xcb;
#[cfg(target_os = "windows")]
extern crate kernel32;

#[cfg(unix)]
use winit::os::unix::WindowExt;
#[cfg(target_os = "windows")]
use winit::os::windows::WindowExt;

mod command;
mod factory;
mod mirror;
mod native;

use core::format;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;
use std::sync::Arc;
use shared_library::dynamic_library::DynamicLibrary;

lazy_static! {
    static ref vk_dynamic_library: DynamicLibrary = {
        use std::path::Path;
        DynamicLibrary::open(Some(
            if cfg!(target_os = "windows") {
                Path::new("vulkan-1.dll")
            } else {
                Path::new("libvulkan.so.1")
            }
        )).expect("Unable to open vulkan shared library")
    };

    static ref vk_library: vk::Static = {
        vk::Static::load(|name| unsafe {
            let name = name.to_str().unwrap();
            vk_dynamic_library.symbol(name).unwrap()
        })
    };
}

pub struct PhysicalDeviceInfo {
    device: vk::PhysicalDevice,
    _properties: vk::PhysicalDeviceProperties,
    queue_families: Vec<vk::QueueFamilyProperties>,
    memory: vk::PhysicalDeviceMemoryProperties,
    _features: vk::PhysicalDeviceFeatures,
}

impl PhysicalDeviceInfo {
    pub fn new(dev: vk::PhysicalDevice, vk: &vk::InstancePointers) -> PhysicalDeviceInfo {
        PhysicalDeviceInfo {
            device: dev,
            _properties: unsafe {
                let mut out = mem::zeroed();
                vk.GetPhysicalDeviceProperties(dev, &mut out);
                out
            },
            queue_families: unsafe {
                let mut num = 0;
                vk.GetPhysicalDeviceQueueFamilyProperties(dev, &mut num, ptr::null_mut());
                let mut families = Vec::with_capacity(num as usize);
                vk.GetPhysicalDeviceQueueFamilyProperties(dev, &mut num, families.as_mut_ptr());
                families.set_len(num as usize);
                families
            },
            memory: unsafe {
                let mut out = mem::zeroed();
                vk.GetPhysicalDeviceMemoryProperties(dev, &mut out);
                out
            },
            _features: unsafe {
                let mut out = mem::zeroed();
                vk.GetPhysicalDeviceFeatures(dev, &mut out);
                out
            },
        }
    }
}

pub struct Queue {
    inner: vk::Queue,
}

// TODO: move this to the window creation
pub struct Surface {
    inner: vk::SurfaceKHR,
}

impl Surface {
    #[cfg(target_os = "windows")]
    pub fn new(instance: Instance, window: &winit::Window) -> vk::SurfaceKHR {
        let (inst, vk) = instance.get();
        let info = vk::Win32SurfaceCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            hinstance: unsafe { kernel32::GetModuleHandleW(ptr::null()) } as *mut _,
            hwnd: window.get_hwnd() as *mut _,
        };
        let mut out = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateWin32SurfaceKHR(inst, &info, ptr::null(), &mut out)
        });
        out
    }

    #[cfg(unix)]
    pub fn new(instance: Instance, window: &winit::Window) -> vk::SurfaceKHR {
        let (inst, vk) = instance.get();
        let info = vk::XcbSurfaceCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            connection: window.get_xcb_connection().unwrap() as *const _,
            window: window.get_xlib_window().unwrap() as *const _,
        };
        let mut out = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateXcbSurfaceKHR(inst, &info, ptr::null(), &mut out)
        });
        out
    }
}

pub struct SwapChain {
    inner: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
}

impl SwapChain {
    pub fn new<T: core::format::RenderFormat>(backend: SharePointer, instance: Instance, surface: Surface, width: u32, height: u32) -> SwapChain {
        let (dev, vk) = backend.get_device();
        let mut images: [vk::Image; 2] = [0; 2];
        let mut num = images.len() as u32;
        let format = <T as format::Formatted>::get_format();

        let surface_capabilities = {
            let (_, vk) = instance.get();
            let dev = backend.get_physical_device();
            let mut capabilities: vk::SurfaceCapabilitiesKHR = unsafe { std::mem::uninitialized() };
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceCapabilitiesKHR(dev, surface.inner, &mut capabilities)
            });
            capabilities
        };

        /*
        // Determine whether a queue family of a physical device supports presentation to a given surface 
        let supports_presentation = {
            let (_, vk) = backend.get_instance();
            let dev = backend.get_physical_device();
            let mut supported = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceSupportKHR(dev, dev.get_family(), surface.inner, &mut supported)
            });
            supported != 0
        };
        */

        let surface_formats = {
            let (_, vk) = instance.get();
            let dev = backend.get_physical_device();
            let mut num = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceFormatsKHR(dev, surface.inner, &mut num, ptr::null_mut())
            });
            let mut formats = Vec::with_capacity(num as usize);
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceFormatsKHR(dev, surface.inner, &mut num, formats.as_mut_ptr())
            });
            unsafe { formats.set_len(num as usize); }
            formats
        };

        let present_modes = {
            let (_, vk) = instance.get();
            let dev = backend.get_physical_device();
            let mut num = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfacePresentModesKHR(dev, surface.inner, &mut num, ptr::null_mut())
            });
            let mut modes = Vec::with_capacity(num as usize);
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfacePresentModesKHR(dev, surface.inner, &mut num, modes.as_mut_ptr())
            });
            unsafe { modes.set_len(num as usize); }
            modes
        };

        // TODO: Use the queried information to check if our values are supported before creating the swapchain
        let swapchain_info = vk::SwapchainCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            surface: surface.inner,
            minImageCount: num,
            imageFormat: device_vulkan::data::map_format(format.0, format.1).unwrap(),
            imageColorSpace: vk::COLOR_SPACE_SRGB_NONLINEAR_KHR,
            imageExtent: vk::Extent2D { width: width, height: height },
            imageArrayLayers: 1,
            imageUsage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            imageSharingMode: vk::SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 1,
            pQueueFamilyIndices: &0,
            preTransform: vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
            compositeAlpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode: vk::PRESENT_MODE_FIFO_KHR, // required to be supported
            clipped: vk::TRUE,
            oldSwapchain: 0,
        };

        let mut swapchain = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateSwapchainKHR(dev, &swapchain_info, ptr::null(), &mut swapchain)
        });

        SwapChain {
            inner: swapchain,
            surface: surface.inner,
        }
    }
}

const SURFACE_EXTENSIONS: &'static [&'static str] = &[
    // Platform-specific WSI extensions
    "VK_KHR_xlib_surface",
    "VK_KHR_xcb_surface",
    "VK_KHR_wayland_surface",
    "VK_KHR_mir_surface",
    "VK_KHR_android_surface",
    "VK_KHR_win32_surface",
];

pub struct Instance {
    inner: vk::Instance,
    pointers: vk::InstancePointers,
    physical_devices: Vec<PhysicalDeviceInfo>,
}

pub type InstancePointer = Arc<Instance>;

impl Instance {
    pub fn new(app_name: &str, app_version: u32, layers: &[&str], extensions: &[&str]) -> InstancePointer {
        let entry_points = vk::EntryPoints::load(|name| unsafe {
            mem::transmute(vk_library.GetInstanceProcAddr(0, name.as_ptr()))
        });

        let app_info = vk::ApplicationInfo {
            sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: ptr::null(),
            pApplicationName: app_name.as_ptr() as *const _,
            applicationVersion: app_version,
            pEngineName: "gfx-rs".as_ptr() as *const _,
            engineVersion: 0x1000, //TODO
            apiVersion: 0x400000, //TODO
        };

        let instance_extensions = {
            let mut num = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                entry_points.EnumerateInstanceExtensionProperties(ptr::null(), &mut num, ptr::null_mut())
            });
            let mut out = Vec::with_capacity(num as usize);
            assert_eq!(vk::SUCCESS, unsafe {
                entry_points.EnumerateInstanceExtensionProperties(ptr::null(), &mut num, out.as_mut_ptr())
            });
            unsafe { out.set_len(num as usize); }
            out
        };

        // Check our surface extensions against the available extensions
        let surface_extensions = SURFACE_EXTENSIONS.iter().filter_map(|ext| {
            instance_extensions.iter().find(|inst_ext| {
                unsafe { CStr::from_ptr(inst_ext.extensionName.as_ptr()) == CStr::from_ptr(ext.as_ptr() as *const i8) }
            }).and_then(|_| Some(*ext))
        }).collect::<Vec<&str>>();

        let instance = {
            let cstrings = layers.iter().chain(extensions.iter())
                                        .chain(surface_extensions.iter())
                             .map(|&s| CString::new(s).unwrap())
                             .collect::<Vec<_>>();
            let str_pointers = cstrings.iter()
                                       .map(|s| s.as_ptr())
                                       .collect::<Vec<_>>();

            let create_info = vk::InstanceCreateInfo {
                sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                pApplicationInfo: &app_info,
                enabledLayerCount: layers.len() as u32,
                ppEnabledLayerNames: str_pointers.as_ptr(),
                enabledExtensionCount: (extensions.len() + surface_extensions.len()) as u32,
                ppEnabledExtensionNames: str_pointers[layers.len()..].as_ptr(),
            };
            let mut out = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                entry_points.CreateInstance(&create_info, ptr::null(), &mut out)
            });
            out
        };

        let inst_pointers = vk::InstancePointers::load(|name| unsafe {
            mem::transmute(vk_library.GetInstanceProcAddr(instance, name.as_ptr()))
        });

        let physical_devices = {
            let mut num = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                inst_pointers.EnumeratePhysicalDevices(instance, &mut num, ptr::null_mut())
            });
            let mut devices = Vec::with_capacity(num as usize);
            assert_eq!(vk::SUCCESS, unsafe {
                inst_pointers.EnumeratePhysicalDevices(instance, &mut num, devices.as_mut_ptr())
            });
            unsafe { devices.set_len(num as usize); }
            devices
        };
        
        let devices = physical_devices.iter()
            .map(|dev| PhysicalDeviceInfo::new(*dev, &inst_pointers))
            .collect::<Vec<_>>();

        Arc::new(Instance {
            inner: instance,
            pointers: inst_pointers,
            physical_devices: devices,
        })
    }

    pub fn get(&self) -> (vk::Instance, &vk::InstancePointers) {
        (self.inner, &self.pointers)
    }

    pub fn physical_devices(&self) -> &Vec<PhysicalDeviceInfo> {
        &self.physical_devices
    }
}

pub struct Device {
    inner: vk::Device,
    pointers: vk::DevicePointers,
}

impl Device {
    pub fn new(instance: &InstancePointer, dev_extensions: &[&str]) -> Arc<Device> {
        let (dev, (qf_id, _))  = {
            let devices = instance.physical_devices();
            devices.iter()
                .flat_map(|d| std::iter::repeat(d).zip(d.queue_families.iter().enumerate()))
                .find(|&(_, (_, qf))| qf.queueFlags & vk::QUEUE_GRAPHICS_BIT != 0)
                .unwrap()
        };

        info!("Chosen physical device {:?} with queue family {}", dev.device, qf_id);

        let (_, vk) = instance.get();

        let device = {
            let cstrings = dev_extensions.iter()
                                         .map(|&s| CString::new(s).unwrap())
                                         .collect::<Vec<_>>();
            let str_pointers = cstrings.iter().map(|s| s.as_ptr())
                                       .collect::<Vec<_>>();

            let queue_info = vk::DeviceQueueCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                queueFamilyIndex: qf_id as u32,
                queueCount: 1,
                pQueuePriorities: &1.0,
            };
            let features = unsafe{ mem::zeroed() };

            let dev_info = vk::DeviceCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                queueCreateInfoCount: 1,
                pQueueCreateInfos: &queue_info,
                enabledLayerCount: 0,
                ppEnabledLayerNames: ptr::null(),
                enabledExtensionCount: str_pointers.len() as u32,
                ppEnabledExtensionNames: str_pointers.as_ptr(),
                pEnabledFeatures: &features,
            };
            let mut out = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.CreateDevice(dev.device, &dev_info, ptr::null(), &mut out)
            });
            out
        };

        let dev_pointers = vk::DevicePointers::load(|name| unsafe {
            vk.GetDeviceProcAddr(device, name.as_ptr()) as *const _
        });

        Arc::new(Device {
            inner: device,
            pointers: dev_pointers,
        })
    }
}

// TODO: outdated, split up
pub struct Share {
    device: vk::Device,
    dev_pointers: vk::DevicePointers,
    physical_device: vk::PhysicalDevice,
    handles: RefCell<core::handle::Manager<Resources>>,
}

pub type SharePointer = Arc<Share>;

impl Share {
    pub fn get_device(&self) -> (vk::Device, &vk::DevicePointers) {
        (self.device, &self.dev_pointers)
    }
    pub fn get_physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }
}


#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resources {}

impl core::Resources for Resources {
    type Buffer               = native::Buffer;
    type Shader               = native::Shader;
    type Program              = native::Program;
    type PipelineStateObject  = native::Pipeline;
    type Image                = native::Image;
    type ShaderResourceView   = native::ImageView; //TODO: buffer view
    type UnorderedAccessView  = ();
    type RenderTargetView     = native::ImageView;
    type DepthStencilView     = native::ImageView;
    type Sampler              = vk::Sampler;
    type Fence                = Fence;
    type Mapping              = factory::MappingGate;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Fence(vk::Fence);

impl core::Fence for Fence {
    fn wait(&self) {
        unimplemented!()
    }
}
