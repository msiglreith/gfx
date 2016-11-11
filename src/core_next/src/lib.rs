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
extern crate bitflags;

extern crate gfx_core as core;
extern crate draw_state;

use std::fmt::Debug;
use std::hash::Hash;
use std::any::Any;
use std::sync::Arc;

pub use draw_state::{state, target};
pub use self::factory::Factory;

pub use core::{format, memory};
pub use core::{MAX_COLOR_TARGETS, MAX_VERTEX_ATTRIBUTES, MAX_CONSTANT_BUFFERS,
     MAX_RESOURCE_VIEWS, MAX_UNORDERED_VIEWS, MAX_SAMPLERS};
pub use core::{AttributeSlot, ColorSlot, ConstantBufferSlot, ResourceViewSlot, SamplerSlot, UnorderedViewSlot};
pub use core::Primitive;
pub use core::{IndexType, VertexCount};
pub use core::command::{ClearColor, InstanceParams};

pub mod buffer;
pub mod command;
pub mod factory;
pub mod handle;
pub mod mapping;
pub mod pso;
pub mod shade;
pub mod texture;

pub trait Queue {
    /// Associated `Resources` type.
    type Resources: Resources;
    /// Associated `CommandBuffer` type. Every `Queue` type can only work with one `CommandBuffer`
    /// type.
    type CommandBuffer: command::CommandBuffer<Self::Resources>;

    /// Submits a `CommandBuffer` to the GPU for execution.
    fn submit(&mut self, &mut Self::CommandBuffer, access: &pso::AccessInfo<Self::Resources>);
}

pub trait CommandPool {
    fn reset(&mut self);
}

#[derive(Clone, Debug)]
pub struct PhysicalDeviceInfo {
    pub device_name: String, // TODO: fixed size?
    pub vendor_id: usize,
    pub device_id: usize,
    pub software: bool,
}

pub trait PhysicalDevice {
    type Device: Device;
    type Queue: Queue;

    fn open_device(&self) -> (Arc<Self::Device>, Vec<Arc<Self::Queue>>);
    fn get_info(&self) -> &PhysicalDeviceInfo;
}

pub trait Device {

}

pub trait Instance {
    type PhysicalDevice: PhysicalDevice;
    fn enumerate_physical_devices(&self) -> &Vec<Self::PhysicalDevice>; // TODO: return an actual iterator
}

pub struct SurfaceCapabilities {

}

pub trait Surface {
    type Instance: Instance;
    type Device: Device;
    type Queue: Queue;
    type Window;

    fn from_window(&Arc<Self::Instance>, &Self::Window) -> Self;
    fn supports_presentation(&self, present_queue: &Arc<Self::Queue>) -> bool;
    fn get_capabilities(&self, device: &Arc<Self::Device>) -> SurfaceCapabilities;
}

pub trait SwapChain {
    type Resources: Resources;
    type Factory: Factory<Self::Resources>;
    type Surface: Surface;
    type Queue: Queue;
    fn new<T: core::format::RenderFormat>(
        factory: &mut Self::Factory,
        present_queue: &Arc<Self::Queue>,
        surface: &Self::Surface,
        width: u32,
        height: u32
    ) -> Self;
    fn present(&mut self);
}

/// Operations that must be provided by a fence.
pub trait Fence {
    /// Stalls the current thread until the fence is satisfied
    fn wait(&self);
}

macro_rules! define_shader_entries {
    ($($entry:ident $shader:ident),+) => {$(
        #[allow(missing_docs)]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $entry<R: Resources>($shader<R>, String);
        impl<R: Resources> $entry<R> {
            pub fn get_shader(&self, man: &mut handle::Manager<R>) -> &R::Shader {
                self.0.reference(man)
            }

            pub fn get_entry_point(&self) -> &str {
                &self.1
            }
        }

        #[allow(missing_docs)]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $shader<R: Resources>(handle::Shader<R>);
        impl<R: Resources> $shader<R> {
            #[allow(missing_docs)]
            pub fn reference(&self, man: &mut handle::Manager<R>) -> &R::Shader {
                man.ref_shader(&self.0)
            }
        }
    )+}
}

define_shader_entries!(
    VertexEntry VertexShader,
    HullEntry HullShader,
    DomainEntry DomainShader,
    GeometryEntry GeometryShader,
    PixelEntry PixelShader
);

/// A complete set of shaders to link a program.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ShaderSet<R: Resources> {
    /// Simple program: Vs-Ps
    Simple(VertexEntry<R>, PixelEntry<R>),
    /// Geometry shader programs: Vs-Gs-Ps
    Geometry(VertexEntry<R>, GeometryEntry<R>, PixelEntry<R>),
    //TODO: Tessellated, TessellatedGeometry, TransformFeedback
}

impl<R: Resources> ShaderSet<R> {
    /// Return the aggregated stage usage for the set.
    pub fn get_usage(&self) -> shade::Usage {
        match *self {
            ShaderSet::Simple(..) => shade::VERTEX | shade::PIXEL,
            ShaderSet::Geometry(..) => shade::VERTEX | shade::GEOMETRY | shade::PIXEL,
        }
    }

    pub fn get_vertex_entry(&self) -> Option<&VertexEntry<R>> {
        match *self {
            ShaderSet::Simple(ref vertex, _) |
            ShaderSet::Geometry(ref vertex, _, _) => Some(vertex),
        }
    }

    pub fn get_geometry_entry(&self) -> Option<&GeometryEntry<R>> {
        match *self {
            ShaderSet::Geometry(_, ref geometry, _) => Some(geometry),
            _ => None,
        }
    }

    pub fn get_pixel_entry(&self) -> Option<&PixelEntry<R>> {
        match *self {
            ShaderSet::Simple(_, ref pixel) |
            ShaderSet::Geometry(_, _, ref pixel) => Some(pixel),
        }
    }
}

/// Different resource types of a specific API. 
pub trait Resources:          Clone + Hash + Debug + Eq + PartialEq + Any {
    type Buffer:              Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync + Copy;
    type Shader:              Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync;
    type RenderPass:          Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync;
    type PipelineLayout:      Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync;
    type PipelineStateObject: Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync;
    type Image:               Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync;
    type ShaderResourceView:  Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync + Copy;
    type UnorderedAccessView: Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync + Copy;
    type RenderTargetView:    Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync + Copy;
    type DepthStencilView:    Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync;
    type Sampler:             Clone + Hash + Debug + Eq + PartialEq + Any + Send + Sync + Copy;
    type Fence:               Clone + Hash + Debug + Eq + PartialEq + Any + Fence;
    type Mapping:             Debug + Any + mapping::Gate<Self>;
}

/// Different types of a specific API.
pub trait Backend {
    type Instance: Instance;
    type Device: Device;
    type Queue: Queue;
    type PhysicalDevice: PhysicalDevice;
    type Surface: Surface;
    type SwapChain: SwapChain;
    type CommandPool: CommandPool;
    type Resources: Resources;
}
