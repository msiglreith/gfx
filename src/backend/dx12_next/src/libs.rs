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
extern crate winapi;

mod command;
mod native;

pub struct Instance {

}

pub struct SwapChain {

}

pub struct Device {
	inner: *mut winapi::ID3D12Device,
}

pub struct Queue {
	inner: *mut ID3D12GraphicsCommandList,
}

pub struct CommandPool {
	inner: *mut winapi::ID3D12CommandAllocator,
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
    type Sampler              = ();
    type Fence                = ();
    type Mapping              = factory::MappingGate;
}
