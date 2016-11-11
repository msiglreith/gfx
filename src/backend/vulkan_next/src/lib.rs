
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
mod data;
mod factory;
mod mirror;
mod native;

pub use factory::Factory;

use core::format;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;
use std::sync::Arc;
use shared_library::dynamic_library::DynamicLibrary;

lazy_static! {
    static ref VK_DYNAMIC_LIBRARY: DynamicLibrary = {
        use std::path::Path;
        DynamicLibrary::open(Some(
            if cfg!(target_os = "windows") {
                Path::new("vulkan-1.dll")
            } else {
                Path::new("libvulkan.so.1")
            }
        )).expect("Unable to open vulkan shared library")
    };

    static ref VK_LIBRARY: vk::Static = {
        vk::Static::load(|name| unsafe {
            let name = name.to_str().unwrap();
            VK_DYNAMIC_LIBRARY.symbol(name).unwrap()
        })
    };
}

pub struct PhysicalDevice {
    device: vk::PhysicalDevice,
    _properties: vk::PhysicalDeviceProperties,
    queue_families: Vec<vk::QueueFamilyProperties>,
    memory: vk::PhysicalDeviceMemoryProperties,
    _features: vk::PhysicalDeviceFeatures,
    info: core::PhysicalDeviceInfo,
}

impl PhysicalDevice {
    pub fn new(dev: vk::PhysicalDevice, vk: &vk::InstancePointers) -> PhysicalDevice {
        let properties = unsafe {
            let mut out = mem::zeroed();
            vk.GetPhysicalDeviceProperties(dev, &mut out);
            out
        };
        let queue_families = unsafe {
            let mut num = 0;
            vk.GetPhysicalDeviceQueueFamilyProperties(dev, &mut num, ptr::null_mut());
            let mut families = Vec::with_capacity(num as usize);
            vk.GetPhysicalDeviceQueueFamilyProperties(dev, &mut num, families.as_mut_ptr());
            families.set_len(num as usize);
            families
        };
        let memory = unsafe {
            let mut out = mem::zeroed();
            vk.GetPhysicalDeviceMemoryProperties(dev, &mut out);
            out
        };
        let features = unsafe {
            let mut out = mem::zeroed();
            vk.GetPhysicalDeviceFeatures(dev, &mut out);
            out
        };
        let device_info = core::PhysicalDeviceInfo {
            device_name: String::new(),
            device_id: properties.deviceID as usize,
            vendor_id: properties.vendorID as usize,
            software: properties.deviceType == vk::PHYSICAL_DEVICE_TYPE_CPU,
        };

        PhysicalDevice {
            device: dev,
            _properties: properties,
            queue_families: queue_families,
            memory: memory,
            _features: features,
            info: device_info,
        }
    }

    pub fn open_device<F>(&self, instance: &InstancePointer, dev_extensions: &[&str], mut queue_filter: F) -> (Arc<Device>, Vec<Arc<Queue>>)
        where F: FnMut(&vk::QueueFamilyProperties) -> bool
    {
        let (_, vk) = instance.get();

        let queue_infos = self.queue_families.iter()
                                .enumerate()
                                .filter(|&(_, queue_family)| queue_filter(queue_family))
                                .map(|(i, queue_family)| {
                                    vk::DeviceQueueCreateInfo {
                                        sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                                        pNext: ptr::null(),
                                        flags: 0,
                                        queueFamilyIndex: i as u32,
                                        queueCount: queue_family.queueCount,
                                        pQueuePriorities: &1.0,
                                    }
                                }).collect::<Vec<_>>();

        let device = {
            let cstrings = dev_extensions.iter()
                                         .map(|&s| CString::new(s).unwrap())
                                         .collect::<Vec<_>>();
            let str_pointers = cstrings.iter().map(|s| s.as_ptr())
                                       .collect::<Vec<_>>();

            let features = unsafe{ mem::zeroed() };

            let dev_info = vk::DeviceCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                queueCreateInfoCount: queue_infos.len() as u32,
                pQueueCreateInfos: queue_infos.as_ptr(),
                enabledLayerCount: 0,
                ppEnabledLayerNames: ptr::null(),
                enabledExtensionCount: str_pointers.len() as u32,
                ppEnabledExtensionNames: str_pointers.as_ptr(),
                pEnabledFeatures: &features,
            };
            let mut out = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.CreateDevice(self.device, &dev_info, ptr::null(), &mut out)
            });
            out
        };

        let dev_pointers = vk::DevicePointers::load(|name| unsafe {
            vk.GetDeviceProcAddr(device, name.as_ptr()) as *const _
        });

        let device = Arc::new(Device {
            inner: device,
            physical: self.device,
            pointers: dev_pointers,
        });

        let queues = self.queue_families.iter()
                                .enumerate()
                                .filter(|&(_, queue_family)| queue_filter(queue_family))
                                .flat_map(|(i, queue_family)| {
                                            (0..queue_family.queueCount)
                                                .map(|j| PhysicalDevice::open_queue(device.clone(), i as u32, j))
                                                .collect::<Vec<_>>()
                                    }).collect::<Vec<_>>();

        (device, queues)
    }

    fn open_queue(device: Arc<Device>, family: u32, index: u32) -> Arc<Queue> {
        let queue = unsafe {
            let (dev, vk) = device.get();
            let mut out = mem::zeroed();
            vk.GetDeviceQueue(dev, family, index, &mut out);
            out
        };

        Arc::new(Queue {
            device: device,
            inner: queue,
            family_index: family,

        })
    }
}

impl core::PhysicalDevice for PhysicalDevice {
    type Device = Device;
    type Queue = Queue;

    fn open_device(&self) -> (Arc<Self::Device>, Vec<Arc<Self::Queue>>) {
        unimplemented!()
    }

    fn get_info(&self) -> &core::PhysicalDeviceInfo {
        &self.info
    }
}


pub struct Queue {
    device: Arc<Device>,
    inner: vk::Queue,
    family_index: u32,
}

impl Queue {
    pub fn get(&self) -> vk::Queue {
        self.inner
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn family_index(&self) -> u32 {
        self.family_index
    }

    pub fn create_command_pool(&self) -> CommandPool {
        let com_info = vk::CommandPoolCreateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: self.family_index,
        };
        let mut com_pool = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            let (dev, vk) = self.device.get();
            vk.CreateCommandPool(dev, &com_info, ptr::null(), &mut com_pool)
        });

        CommandPool {
            inner: com_pool,
            device: self.device.clone(),
        }
    }
}

impl core::Queue for Queue {
    type Resources = Resources;
    type CommandBuffer = command::Buffer;

    fn submit(&mut self, command_buffer: &mut Self::CommandBuffer, access: &core::pso::AccessInfo<Self::Resources>) {
        // self.ensure_mappings_flushed(access.mapped_reads());

        let (_, vk) = self.device.get();

        let submit_info = vk::SubmitInfo {
            sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
            commandBufferCount: 1,
            pCommandBuffers: &command_buffer.get(),
            .. unsafe { mem::zeroed() }
        };
        assert_eq!(vk::SUCCESS, unsafe {
            vk.QueueSubmit(self.inner, 1, &submit_info, 0)
        });
    }
}

pub struct CommandPool {
    inner: vk::CommandPool,
    device: Arc<Device>,
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let (dev, vk) = self.device.get();
        unsafe {
            vk.DestroyCommandPool(dev, self.inner, ptr::null())
        };
    }
}

impl CommandPool {
    pub fn create_command_buffers(&self, num: usize) -> Vec<command::Buffer> {
        let alloc_info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ptr::null(),
            commandPool: self.inner,
            level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: num as u32,
        };

        let (dev, vk) = self.device.get();
        let mut buf = Vec::with_capacity(num);
        assert_eq!(vk::SUCCESS, unsafe {
            vk.AllocateCommandBuffers(dev, &alloc_info, buf.as_mut_ptr())
        });

        buf.iter().map(|&buffer| {
            command::Buffer::new(buffer, self.device.clone())
        }).collect::<Vec<_>>()
    }
}

// TODO: move this to the window creation
pub struct Surface {
    inner: vk::SurfaceKHR,
    instance: Arc<Instance>,
}

impl Surface {
    #[cfg(target_os = "windows")]
    pub fn new(instance: &Arc<Instance>, window: &winit::Window) -> Surface {
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
        Surface {
            inner: out,
            instance: instance.clone(),
        }
    }

    #[cfg(unix)]
    pub fn new(instance: &Arc<Instance>, window: &winit::Window) -> Surface {
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
        Surface {
            inner: out,
            instance: instance.clone(),
        }
    }
}

impl core::Surface for Surface {
    type Instance = Instance;
    type Queue = Queue;
    type Device = Device;
    type Window = winit::Window;

    fn from_window(instance: &Arc<Self::Instance>, window: &Self::Window) -> Self {
        Self::new(instance, window)
    }

    fn supports_presentation(&self, present_queue: &Arc<Self::Queue>) -> bool {
        let (_, vk) = self.instance.get();
        let dev = present_queue.get_device().get_physical_device();
        let mut supported = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.GetPhysicalDeviceSurfaceSupportKHR(dev, present_queue.family_index(), self.inner, &mut supported)
        });
        supported != 0
    }

    fn get_capabilities(&self, device: &Arc<Self::Device>) -> core::SurfaceCapabilities {
        let (_, vk) = self.instance.get();
        let dev = device.get_physical_device();
        let surface_capabilities = {
            
            
            let mut capabilities: vk::SurfaceCapabilitiesKHR = unsafe { std::mem::uninitialized() };
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceCapabilitiesKHR(dev, self.inner, &mut capabilities)
            });
            capabilities
        };

        let surface_formats = {
            let mut num = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceFormatsKHR(dev, self.inner, &mut num, ptr::null_mut())
            });
            let mut formats = Vec::with_capacity(num as usize);
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfaceFormatsKHR(dev, self.inner, &mut num, formats.as_mut_ptr())
            });
            unsafe { formats.set_len(num as usize); }
            formats
        };

        let present_modes = {
            let mut num = 0;
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfacePresentModesKHR(dev, self.inner, &mut num, ptr::null_mut())
            });
            let mut modes = Vec::with_capacity(num as usize);
            assert_eq!(vk::SUCCESS, unsafe {
                vk.GetPhysicalDeviceSurfacePresentModesKHR(dev, self.inner, &mut num, modes.as_mut_ptr())
            });
            unsafe { modes.set_len(num as usize); }
            modes
        };

        unimplemented!()
    }
}

pub struct SwapChain {
    inner: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
    present_queue: Arc<Queue>,
}

impl core::SwapChain for SwapChain {
    type Resources = Resources;
    type Factory = Factory;
    type Surface = Surface;
    type Queue = Queue;
    fn new<T: core::format::RenderFormat>(
        factory: &mut Self::Factory,
        present_queue: &Arc<Self::Queue>,
        surface: &Self::Surface,
        width: u32,
        height: u32
    ) -> Self {
        let (dev, vk) = factory.get_device().get();
        let mut images: [vk::Image; 2] = [0; 2];
        let mut num = images.len() as u32;
        let format = <T as format::Formatted>::get_format();

        let mut presentation_mode = vk::PRESENT_MODE_FIFO_KHR; // required to be supported
        /*
        for mode in present_modes  {
            // lowest-latency non-tearing mode according to vulkan specs
            if mode == vk::PRESENT_MODE_MAILBOX_KHR {
                presentation_mode = vk::PRESENT_MODE_MAILBOX_KHR;
                break;
            }
        }
        */

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
            presentMode: presentation_mode,
            clipped: vk::TRUE,
            oldSwapchain: 0,
        };

        let mut swapchain = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateSwapchainKHR(dev, &swapchain_info, ptr::null(), &mut swapchain)
        });

        assert_eq!(vk::SUCCESS, unsafe {
            vk.GetSwapchainImagesKHR(dev, swapchain, &mut num, images.as_mut_ptr())
        });

        SwapChain {
            inner: swapchain,
            surface: surface.inner,
            present_queue: present_queue.clone(),
        }
    }

    fn present(&mut self) {
        let mut result = vk::SUCCESS;
        let info = vk::PresentInfoKHR {
            sType: vk::STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            swapchainCount: 1,
            pSwapchains: &self.inner,
            pImageIndices: &0, // &self.target_id,
            pResults: &mut result,
        };
        let (_, vk) = self.present_queue.device.get();
        unsafe {
            vk.QueuePresentKHR(self.present_queue.get(), &info);
        }
        assert_eq!(vk::SUCCESS, result);
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
    physical_devices: Vec<PhysicalDevice>,
}

pub type InstancePointer = Arc<Instance>;

impl Instance {
    pub fn new(app_name: &str, app_version: u32, layers: &[&str], extensions: &[&str]) -> (InstancePointer, Arc<Share>) {
        let entry_points = vk::EntryPoints::load(|name| unsafe {
            mem::transmute(VK_LIBRARY.GetInstanceProcAddr(0, name.as_ptr()))
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
            mem::transmute(VK_LIBRARY.GetInstanceProcAddr(instance, name.as_ptr()))
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
            .map(|dev| PhysicalDevice::new(*dev, &inst_pointers))
            .collect::<Vec<_>>();

        let instance = Arc::new(Instance {
                inner: instance,
                pointers: inst_pointers,
                physical_devices: devices,
            });

        let share = Arc::new(Share {
                handles: RefCell::new(core::handle::Manager::new()),
            });

        (instance, share)
    }

    pub fn get(&self) -> (vk::Instance, &vk::InstancePointers) {
        (self.inner, &self.pointers)
    }
}

impl core::Instance for Instance {
    type PhysicalDevice = PhysicalDevice;
    fn enumerate_physical_devices(&self) -> &Vec<Self::PhysicalDevice> {
        &self.physical_devices
    }
}

pub struct Device {
    inner: vk::Device,
    physical: vk::PhysicalDevice,
    pointers: vk::DevicePointers,
}

impl Device {
    pub fn get(&self) -> (vk::Device, &vk::DevicePointers) {
        (self.inner, &self.pointers)
    }

    pub fn get_physical_device(&self) -> vk::PhysicalDevice {
        self.physical
    }
}

impl core::Device for Device {

}

pub struct Share {
    handles: RefCell<core::handle::Manager<Resources>>,
}

type SharePointer = Arc<Share>;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resources {}

impl core::Resources for Resources {
    type Buffer               = native::Buffer;
    type Shader               = native::Shader;
    type RenderPass           = native::RenderPass;
    type PipelineLayout       = native::PipelineLayout;
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
