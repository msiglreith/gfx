// Copyright 2016 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate log;
extern crate gfx_core_next as core;
extern crate d3d12;
extern crate dxgi;
extern crate dxguid;
extern crate winapi;
extern crate winit;
extern crate comptr;

use std::ptr;
use std::sync::Arc;
use std::os::windows::ffi::{OsStringExt};
use std::ffi::{OsString};
use comptr::ComPtr;
use winapi::BOOL;

#[cfg(target_os = "windows")]
use winit::os::windows::WindowExt;

pub use factory::Factory;

mod command;
mod data;
mod factory;
mod native;

pub struct Instance {
    inner: ComPtr<winapi::IDXGIFactory4>,
    physical_devices: Vec<PhysicalDevice>
}

impl Instance {
    pub fn new() -> Instance {
        let mut debug_controller: ComPtr<winapi::ID3D12Debug> = ComPtr::new(ptr::null_mut());
        let hr = unsafe { d3d12::D3D12GetDebugInterface(&dxguid::IID_ID3D12Debug, debug_controller.as_mut() as *mut *mut _ as *mut *mut std::os::raw::c_void) };
        if winapi::SUCCEEDED(hr) {
            unsafe { debug_controller.EnableDebugLayer() };
        }

        let mut dxgi_factory: ComPtr<winapi::IDXGIFactory4> = ComPtr::new(ptr::null_mut());
        let hr = unsafe {
            dxgi::CreateDXGIFactory2(winapi::DXGI_CREATE_FACTORY_DEBUG, &dxguid::IID_IDXGIFactory4, dxgi_factory.as_mut() as *mut *mut _ as *mut *mut std::os::raw::c_void)
        };

        if !winapi::SUCCEEDED(hr) {
            println!("error on dxgi factory {:?}", hr);
        }

        // enumerate adapters
        let mut cur_index = 0;
        let mut devices = Vec::new();
        loop {
            let mut adapter: ComPtr<winapi::IDXGIAdapter2> = ComPtr::new(ptr::null_mut());
            let hr = unsafe { dxgi_factory.EnumAdapters1(cur_index, adapter.as_mut() as *mut *mut _ as *mut *mut winapi::IDXGIAdapter1) };
            if hr == winapi::DXGI_ERROR_NOT_FOUND {
                break;
            }

            // check if the adapter supports dx12
            let hr = unsafe {
                d3d12::D3D12CreateDevice(
                    adapter.as_mut_ptr() as *mut _ as *mut winapi::IUnknown,
                    winapi::D3D_FEATURE_LEVEL_11_0, // TODO: correct feature level?
                    &dxguid::IID_ID3D12Device,
                    ptr::null_mut(),
                )
            };

            if winapi::SUCCEEDED(hr) {
                // we have a possible adapter!
                let mut desc: winapi::DXGI_ADAPTER_DESC2 = unsafe { std::mem::uninitialized() };
                unsafe { adapter.GetDesc2(&mut desc); }
                let device_name: OsString = OsStringExt::from_wide(&desc.Description);
                let device_name = device_name.into_string().unwrap(); // TODO: do this nicer and trim the \0 at the end
                println!("{:?}", device_name);

                let info = core::PhysicalDeviceInfo {
                    device_name: device_name,
                    vendor_id: desc.VendorId as usize,
                    device_id: desc.DeviceId as usize,
                    software: false, // TODO
                };

                devices.push(PhysicalDevice {
                    adapter: adapter,
                    info: info,
                });
            }

            cur_index += 1;
        }

        Instance {
            inner: dxgi_factory,
            physical_devices: devices,
        }
    }

    pub fn get(&self) -> &ComPtr<winapi::IDXGIFactory4> {
        &self.inner
    }
}

impl core::Instance for Instance {
    type PhysicalDevice = PhysicalDevice;

    fn enumerate_physical_devices(&self) -> &Vec<Self::PhysicalDevice> {
        &self.physical_devices
    }
}

pub struct PhysicalDevice {
    adapter: ComPtr<winapi::IDXGIAdapter2>,
    info: core::PhysicalDeviceInfo,
}

impl core::PhysicalDevice for PhysicalDevice {
    type Device = Device;
    type Queue = Queue;

    fn open_device(&self) -> (Arc<Self::Device>, Vec<Arc<Self::Queue>>) {
        let mut device: ComPtr<winapi::ID3D12Device> = ComPtr::new(ptr::null_mut());
        let hr = unsafe {
            d3d12::D3D12CreateDevice(
                self.adapter.as_mut_ptr() as *mut _ as *mut winapi::IUnknown,
                winapi::D3D_FEATURE_LEVEL_11_0, // TODO: correct feature level?
                &dxguid::IID_ID3D12Device,
                device.as_mut() as *mut *mut _ as *mut *mut std::os::raw::c_void,
            )
        };
        if !winapi::SUCCEEDED(hr) {
            println!("error on device creation: {:?}", hr);
        }
        let mut queue: ComPtr<winapi::ID3D12CommandQueue> = ComPtr::new(ptr::null_mut());
        let queue_desc = winapi::D3D12_COMMAND_QUEUE_DESC {
            Type: winapi::D3D12_COMMAND_LIST_TYPE_DIRECT,
            Priority: 0,
            Flags: winapi::D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 1,
        };

        let hr = unsafe {
            device.CreateCommandQueue(
                &queue_desc,
                &dxguid::IID_ID3D12CommandQueue,
                queue.as_mut() as *mut *mut _ as *mut *mut std::os::raw::c_void,
            )
        };

        if !winapi::SUCCEEDED(hr) {
            println!("error on queue creation: {:?}", hr);
        }

        (Arc::new(Device { inner: device }), vec![Arc::new(Queue { inner: queue })])
    }

    fn get_info(&self) -> &core::PhysicalDeviceInfo {
        &self.info
    }
}

pub struct Surface {
    wnd_handle: winapi::HWND,
    instance: Arc<Instance>,
}

impl Surface {
    pub fn get_hwnd(&self) -> winapi::HWND {
        self.wnd_handle
    }
}

impl core::Surface for Surface {
    type Instance = Instance;
    type Device = Device;
    type Queue = Queue;
    type Window = winit::Window;

    fn from_window(instance: &Arc<Self::Instance>, window: &Self::Window) -> Self {
        Surface {
            wnd_handle: window.get_hwnd() as *mut _,
            instance: instance.clone(),
        }
    }


    fn supports_presentation(&self, present_queue: &Arc<Self::Queue>) -> bool {
        unimplemented!()
    }

    fn get_capabilities(&self, device: &Arc<Self::Device>) -> core::SurfaceCapabilities {
        unimplemented!()
    }
}

pub struct SwapChain {
    inner: ComPtr<winapi::IDXGISwapChain1>,
}

impl core::SwapChain for SwapChain {
    type Resources = Resources;
    type Factory = Factory;
    type Surface = Surface;
    type Queue = Queue;

    fn new<T: core::format::RenderFormat>(
        _factory: &mut Self::Factory,
        present_queue: &Arc<Self::Queue>,
        surface: &Self::Surface,
        width: u32,
        height: u32
    ) -> Self {
        // TODO: re-check values
        let desc = winapi::DXGI_SWAP_CHAIN_DESC1 {
            AlphaMode: winapi::DXGI_ALPHA_MODE(0),
            BufferCount: 2,
            Width: width,
            Height: height,
            Format: data::map_format(T::get_format(), true).unwrap(), // TODO: error handling
            Flags: 0,
            BufferUsage: winapi::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SampleDesc: winapi::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: winapi::DXGI_SCALING(0),
            Stereo: false as BOOL,
            SwapEffect: winapi::DXGI_SWAP_EFFECT(4), // TODO: FLIP_DISCARD
        };

        let mut swap_chain: ComPtr<winapi::IDXGISwapChain1> = ComPtr::new(ptr::null_mut());
        let dxgi_factory = surface.instance.get();
        let hr = unsafe {
            (**dxgi_factory.as_ref()).CreateSwapChainForHwnd(
                present_queue.get().as_mut_ptr() as *mut _ as *mut winapi::IUnknown,
                surface.get_hwnd(),
                &desc,
                ptr::null(),
                ptr::null_mut(),
                swap_chain.as_mut() as *mut *mut _,
            )
        };

        if !winapi::SUCCEEDED(hr) {
            println!("error on swapchain creation {:x}", hr);
        }
        
        SwapChain {
            inner: swap_chain,
        }
    }

    fn present(&mut self) {
        unsafe {
            self.inner.Present(1, 0); // TODO: check values
        }
    }
}

pub struct Device {
	inner: ComPtr<winapi::ID3D12Device>,
}

impl Device {
    pub fn get(&self) -> &ComPtr<winapi::ID3D12Device> {
        &self.inner
    }
}

impl core::Device for Device {

}

pub struct Queue {
	inner: ComPtr<winapi::ID3D12CommandQueue>,
}

impl Queue {
    pub fn get(&self) -> &ComPtr<winapi::ID3D12CommandQueue> {
        &self.inner
    }
}

impl core::Queue for Queue {
    type Resources = Resources;
    type CommandBuffer = command::Buffer;

    fn submit(&mut self, command_buffer: &mut Self::CommandBuffer, access: &core::pso::AccessInfo<Self::Resources>) {
        unimplemented!()
    }
}

pub struct CommandPool {
	inner: ComPtr<winapi::ID3D12CommandAllocator>,
}

impl core::CommandPool for CommandPool {
    fn reset(&mut self) {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resources {}

impl core::Resources for Resources {
    type Buffer               = native::Buffer;
    type Shader               = native::Shader;
    type PipelineLayout       = ();
    type RenderPass           = ();
    type PipelineStateObject  = native::Pipeline;
    type Image                = native::Image;
    type ShaderResourceView   = native::ImageView; //TODO: buffer view
    type UnorderedAccessView  = ();
    type RenderTargetView     = native::ImageView;
    type DepthStencilView     = native::ImageView;
    type Sampler              = ();
    type Fence                = Fence;
    type Mapping              = factory::MappingGate;
}

pub enum Backend { }
impl core::Backend for Backend {
    type Instance = Instance;
    type Device = Device;
    type Queue = Queue;
    type PhysicalDevice = PhysicalDevice;
    type Surface = Surface;
    type SwapChain = SwapChain;
    type CommandPool = CommandPool;
    type Resources = Resources;
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Fence;

impl core::Fence for Fence {
    fn wait(&self) {
        unimplemented!()
    }
}
