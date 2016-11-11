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

use core::{self, buffer, factory, handle, mapping, shade, texture, pso};
use core::ShaderSet;
use std::slice;
use {Resources as R};

#[derive(Copy, Clone, Debug)]
pub struct MappingGate {
    pointer: *mut (),
}

impl mapping::Gate<R> for MappingGate {
    unsafe fn set<T>(&self, index: usize, val: T) {
        *(self.pointer as *mut T).offset(index as isize) = val;
    }

    unsafe fn slice<'a, 'b, T>(&'a self, len: usize) -> &'b [T] {
        slice::from_raw_parts(self.pointer as *const T, len)
    }

    unsafe fn mut_slice<'a, 'b, T>(&'a self, len: usize) -> &'b mut [T] {
        slice::from_raw_parts_mut(self.pointer as *mut T, len)
    }
}

pub struct Factory {

}


impl factory::Factory<R> for Factory {
    fn allocate_memory(&mut self) {
        unimplemented!()
    }

    fn create_renderpass(&mut self) -> handle::RenderPass<R> {
        unimplemented!()
    }

    fn create_pipeline_layout(&mut self) -> handle::PipelineLayout<R> {
        unimplemented!()
    }

    fn create_fence(&mut self, signaled: bool) -> handle::Fence<R> {
        unimplemented!()
    }

    fn create_shader(&mut self, code: &[u8]) -> Result<handle::Shader<R>, shade::CreateShaderError> {
        unimplemented!()
    }

    fn create_compute_pipelines(&mut self) -> Vec<Result<handle::RawPipelineState<R>, pso::CreationError>> {
        unimplemented!()
    }

    fn create_graphics_pipelines<'a>(&mut self, infos: &[(&ShaderSet<R>, &handle::PipelineLayout<R>, handle::SubPass<'a, R>, &pso::PipelineDesc)]) -> Vec<Result<handle::RawPipelineState<R>, pso::CreationError>> {
        unimplemented!()
    }

    fn create_pipeline_cache(&mut self) -> () {
        unimplemented!()
    }

    fn create_buffer_raw(&mut self, info: buffer::Info) -> Result<handle::RawBuffer<R>, buffer::CreationError> {
        unimplemented!()
    }

    fn create_buffer_view(&mut self) -> () {
        unimplemented!()
    }

    fn create_image(&mut self, desc: texture::Info, hint: Option<core::format::ChannelType>) -> Result<handle::RawTexture<R>, texture::CreationError> {
        unimplemented!()
    }

    fn create_image_view(&mut self) -> () {
        unimplemented!()
    }

    fn create_sampler(&mut self, info: texture::SamplerInfo) -> () {
       unimplemented!()
    }

}