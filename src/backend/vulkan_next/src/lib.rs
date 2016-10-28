
#[macro_use]
extern crate log;
extern crate shared_library;
extern crate gfx_core_next as core;
extern crate vk_sys as vk;
extern crate spirv_utils;

mod command;
mod factory;
mod mirror;
mod native;

use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::sync::Arc;
use shared_library::dynamic_library::DynamicLibrary;

struct PhysicalDeviceInfo {
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

pub struct Share {
    _dynamic_lib: DynamicLibrary,
    _library: vk::Static,
    instance: vk::Instance,
    inst_pointers: vk::InstancePointers,
    device: vk::Device,
    dev_pointers: vk::DevicePointers,
    physical_device: vk::PhysicalDevice,
    handles: RefCell<core::handle::Manager<Resources>>,
}

pub type SharePointer = Arc<Share>;

impl Share {
    pub fn get_instance(&self) -> (vk::Instance, &vk::InstancePointers) {
        (self.instance, &self.inst_pointers)
    }
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
