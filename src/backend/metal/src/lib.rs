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
#[macro_use]
extern crate objc;
extern crate objc_foundation;
extern crate cocoa;
extern crate gfx_core as core;
extern crate metal_rs as metal;
extern crate bit_set;

// use cocoa::base::{selector, class};
// use cocoa::foundation::{NSUInteger};

use metal::*;

use core::{handle, texture as tex};
use core::SubmissionResult;
use core::memory::{self, Usage, Bind};
use core::command::{AccessInfo, AccessGuard};

use std::cell::RefCell;
use std::sync::Arc;
// use std::{mem, ptr};

mod factory;
mod encoder;
mod command;
mod mirror;
mod map;
pub mod native;

/*
pub use self::command::CommandBuffer;
pub use self::factory::Factory;
pub use self::map::*;

// Grabbed from https://developer.apple.com/metal/limits/
const MTL_MAX_TEXTURE_BINDINGS: usize = 128;
const MTL_MAX_BUFFER_BINDINGS: usize = 31;
const MTL_MAX_SAMPLER_BINDINGS: usize = 16;

/// Internal struct of shared data between the device and its factories.
#[doc(hidden)]
pub struct Share {
    capabilities: core::Capabilities,
    handles: RefCell<handle::Manager<Resources>>,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct InputLayout(pub MTLVertexDescriptor);
unsafe impl Send for InputLayout {}
unsafe impl Sync for InputLayout {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Shader {
    func: MTLFunction,
}
unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Program {
    vs: MTLFunction,
    ps: MTLFunction,
}
unsafe impl Send for Program {}
unsafe impl Sync for Program {}

pub struct ShaderLibrary {
    lib: MTLLibrary,
}
unsafe impl Send for ShaderLibrary {}
unsafe impl Sync for ShaderLibrary {}

// ShaderLibrary isn't handled via Device.cleanup(). Not really an issue since it will usually
// live for the entire application lifetime and be cloned rarely.
impl Drop for ShaderLibrary {
    fn drop(&mut self) {
        unsafe { self.lib.release() };
    }
}

impl Clone for ShaderLibrary {
    fn clone(&self) -> Self {
        unsafe { self.lib.retain() };
        ShaderLibrary { lib: self.lib }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pipeline {
    pipeline: MTLRenderPipelineState,
    depth_stencil: Option<MTLDepthStencilState>,
    winding: MTLWinding,
    cull: MTLCullMode,
    fill: MTLTriangleFillMode,
    alpha_to_one: bool,
    alpha_to_coverage: bool,
    depth_bias: i32,
    slope_scaled_depth_bias: i32,
    depth_clip: bool,
}
unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Buffer(native::Buffer, Usage, Bind);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Texture(native::Texture, Usage);

pub struct Device {
    pub device: MTLDevice,
    pub drawable: *mut CAMetalDrawable,
    pub backbuffer: *mut MTLTexture,
    feature_set: MTLFeatureSet,
    share: Arc<Share>,
    frame_handles: handle::Manager<Resources>,
    max_resource_count: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Fence;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resources {}

impl core::Resources for Resources {
    type Buffer = Buffer;
    type Shader = Shader;
    type Program = Program;
    type PipelineStateObject = Pipeline;
    type Texture = Texture;
    type RenderTargetView = native::Rtv;
    type DepthStencilView = native::Dsv;
    type ShaderResourceView = native::Srv;
    type UnorderedAccessView = ();
    type Sampler = native::Sampler;
    type Fence = Fence;
    type Semaphore = (); // TODO
    type Mapping = factory::RawMapping;
}

pub type ShaderModel = u16;

impl Device {
    pub fn get_shader_model(&self) -> ShaderModel {
        use metal::MTLFeatureSet::*;

        match self.feature_set {
            iOS_GPUFamily1_v1 |
            iOS_GPUFamily1_v2 => 10,
            iOS_GPUFamily2_v1 |
            iOS_GPUFamily2_v2 |
            iOS_GPUFamily3_v1 |
            OSX_GPUFamily1_v1 => 11,
        }
    }
}

impl core::Device for Device {
    type Resources = Resources;
    type CommandBuffer = command::CommandBuffer;

    fn get_capabilities(&self) -> &core::Capabilities {
        &self.share.capabilities
    }

    fn pin_submitted_resources(&mut self, man: &handle::Manager<Resources>) {
        self.frame_handles.extend(man);
        match self.max_resource_count {
            Some(c) if self.frame_handles.count() > c => {
                error!("Way too many resources in the current frame. Did you call \
                        Device::cleanup()?");
                self.max_resource_count = None;
            }
            _ => (),
        }
    }

    fn submit(&mut self,
              cb: &mut command::CommandBuffer,
              access: &AccessInfo<Resources>) -> SubmissionResult<()> {
        let _guard = try!(access.take_accesses());
        cb.commit(unsafe { *self.drawable });
        Ok(())
    }

    fn fenced_submit(&mut self,
                     _: &mut Self::CommandBuffer,
                     _: &AccessInfo<Resources>,
                     _after: Option<handle::Fence<Resources>>)
                     -> SubmissionResult<handle::Fence<Resources>> {
        unimplemented!()
    }

    fn wait_fence(&mut self, fence: &handle::Fence<Self::Resources>) {
        unimplemented!()
    }

    fn cleanup(&mut self) {
        use core::handle::Producer;
        self.frame_handles.clear();
        self.share.handles.borrow_mut().clean_with(&mut (),
                                                   |_, _v| {
                                                       // v.0.release();
                                                   }, // buffer
                                                   |_, _s| { //shader
                /*(*s.object).Release();
                (*s.reflection).Release();*/
            },
                                                   |_, _p| {
                                                       // if !p.vs.is_null() { p.vs.release(); }
                                                       // if !p.ps.is_null() { p.ps.release(); }
                                                   }, // program
                                                   |_, _v| { //PSO
                /*type Child = *mut winapi::ID3D11DeviceChild;
                (*v.layout).Release();
                (*(v.rasterizer as Child)).Release();
                (*(v.depth_stencil as Child)).Release();
                (*(v.blend as Child)).Release();*/
            },
                                                   |_, _v| {
                                                       // (*(v.0).0).release();
                                                   }, // texture
                                                   |_, _v| {
                                                       // (*v.0).Release();
                                                   }, // SRV
                                                   |_, _| {}, // UAV
                                                   |_, _v| {
                                                       // (*v.0).Release();
                                                   }, // RTV
                                                   |_, _v| {
                                                       // (*v.0).Release();
                                                   }, // DSV
                                                   |_, _v| {
                                                       // v.sampler.release();
                                                   }, // sampler
                                                   |_, _| {
                                                       // fence
                                                   });
    }
}

#[derive(Clone, Debug)]
pub enum InitError {
    FeatureSet,
}

pub fn create(format: core::format::Format,
              width: u32,
              height: u32)
              -> Result<(Device,
                         Factory,
                         handle::RawRenderTargetView<Resources>,
                         *mut CAMetalDrawable,
                         *mut MTLTexture),
                        InitError> {
    use core::handle::Producer;

    let share = Share {
        capabilities: core::Capabilities {
            max_texture_size: 0,
            max_patch_size: 0,
            instance_base_supported: false,
            instance_call_supported: false,
            instance_rate_supported: false,
            vertex_base_supported: false,
            srgb_color_supported: false,
            constant_buffer_supported: true,
            unordered_access_view_supported: false,
            separate_blending_slots_supported: false,
            copy_buffer_supported: true,
        },
        handles: RefCell::new(handle::Manager::new()),
    };

    let mtl_device = create_system_default_device();
    let feature_sets = {
        use metal::MTLFeatureSet::*;
        [OSX_GPUFamily1_v1,
         //OSX_GPUFamily1_v2,
         iOS_GPUFamily3_v1,
         iOS_GPUFamily2_v2,
         iOS_GPUFamily2_v1,
         iOS_GPUFamily1_v2,
         iOS_GPUFamily1_v1]
    };
    let selected_set = feature_sets.into_iter()
                                   .find(|&&f| mtl_device.supports_feature_set(f));

    let bb = Box::into_raw(Box::new(MTLTexture::nil()));
    let d = Box::into_raw(Box::new(CAMetalDrawable::nil()));

    let device = Device {
        device: mtl_device,
        feature_set: match selected_set {
            Some(&set) => set,
            None => return Err(InitError::FeatureSet),
        },
        share: Arc::new(share),
        frame_handles: handle::Manager::new(),
        max_resource_count: None,

        drawable: d,
        backbuffer: bb,
    };

    // let raw_addr: *mut MTLTexture = ptr::null_mut();//&mut MTLTexture::nil();//unsafe { mem::transmute(&(raw_tex.0).0) };
    let raw_tex = Texture(native::Texture(bb), Usage::Data);

    let color_info = tex::Info {
        kind: tex::Kind::D2(width as tex::Size,
                            height as tex::Size,
                            tex::AaMode::Single),
        levels: 1,
        format: format.0,
        bind: memory::RENDER_TARGET,
        usage: raw_tex.1,
    };
    let color_tex = device.share.handles.borrow_mut().make_texture(raw_tex, color_info);

    let mut factory = Factory::new(mtl_device, device.share.clone());

    let color_target = {
        use core::Factory;

        let desc = tex::RenderDesc {
            channel: format.1,
            level: 0,
            layer: None,
        };

        factory.view_texture_as_render_target_raw(&color_tex, desc).unwrap()
    };

    Ok((device, factory, color_target, d, bb))
}
*/

#[derive(Clone)]
pub struct QueueFamily;
impl core::QueueFamily for QueueFamily {
    fn num_queues(&self) -> u32 { 1 }
}

pub struct Adapter {
    device: MTLDevice,
    adapter_info: core::AdapterInfo,
    queue_families: [QueueFamily; 1],
}

impl core::Adapter for Adapter {
    fn open(&self, queue_descs: &[(&QueueFamily, QueueType, u32)])
        -> core::Device<Backend>
    {
        if queue_descs.len() != 1 {
            panic!("Metal only supports one queue family");
        }
        let (_, queue_type, queue_count) = queue_descs.next().unwrap();
        let factory = factory::create_factory(self.device);

        // TODO: queues
        core::Device {
            factory,
            general_queues: Vec::new(),
            graphics_queues: Vec::new(),
            compute_queues: Vec::new(),
            transfer_queues: Vec::new(),
            heap_types: Vec::new(),
            memory_heaps: Vec::new(),
            _marker: PhantomData,
        }
    }

    fn get_info(&self) -> &core::AdapterInfo {
        unimplemented!()
    }

    fn get_queue_families(&self) -> &[(QueueFamily, QueueType)] {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Backend {}
impl core::Backend for Backend {
    type Adapter = Adapter;
    type Resources = Resources;
    type CommandQueue = CommandQueue;
    type RawCommandBuffer = command::CommandBuffer;
    type SubpassCommandBuffer = command::SubpassCommandBuffer;
    type SubmitInfo = command::SubmitInfo;
    type Factory = Factory;
    type QueueFamily = QueueFamily;

    type RawCommandPool = pool::RawCommandPool;
    type SubpassCommandPool = pool::SubpassCommandPool;
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resources {}
impl core::Resources for Resources {
    type Buffer = ();
    type Shader = ();
    type Program = ();
    type PipelineStateObject = ();
    type Texture = ();
    type ShaderResourceView = ();
    type UnorderedAccessView = ();
    type RenderTargetView = ();
    type DepthStencilView = ();
    type Sampler = ();
    type Fence = ();
    type Semaphore = ();
    type Mapping = Mapping;
}
