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

use core::{command, pso, state, target};
use core::command::{BufferCopy, BufferBarrier, ImageBarrier};
use core::{ClearColor, VertexCount, IndexType, InstanceParams};
use core::MAX_VERTEX_ATTRIBUTES;
use {Resources, Device};
use native;
use std::sync::Arc;
use winapi::{self, INT, UINT, FLOAT};
use comptr::ComPtr;

pub struct Buffer {
    inner: ComPtr<winapi::ID3D12GraphicsCommandList>,
    device: Arc<Device>,
}

impl command::CommandBuffer<Resources> for Buffer {
	fn next_subpass(&mut self) -> () {
        unimplemented!()
    }
    
    fn end_renderpass(&mut self) -> () {
        unimplemented!()
    }
    
    fn clear_attachment(&mut self) -> () {
        unimplemented!()
    }

    fn draw(&mut self, vertex_start: VertexCount, vertex_count: VertexCount, instance: Option<InstanceParams>) {
        let (instance_count, instance_start) = instance.unwrap_or((1, 0));
        unsafe {
            self.inner.DrawInstanced(
                vertex_count as UINT,
                instance_count as UINT,
                vertex_start as UINT,
                instance_start as UINT,
            );
        }
    }

    fn draw_indexed(&mut self, index_start: VertexCount, index_count: VertexCount, vertex_base: VertexCount, instance: Option<InstanceParams>) {
        let (instance_count, instance_start) = instance.unwrap_or((1, 0));
        unsafe {
            self.inner.DrawIndexedInstanced(
                index_count as UINT,
                instance_count as UINT,
                index_start as UINT,
                vertex_base as INT,
                instance_start as UINT,
            );
        }
    }

    fn draw_indirect(&mut self) -> () {
        unimplemented!()
    }

    fn draw_indexed_indirect(&mut self) -> () {
        unimplemented!()
    }

    fn clear_depth_stencil(&mut self, dsv: native::ImageView,
                           depth: Option<target::Depth>, stencil: Option<target::Stencil>) {
        unimplemented!()
    }
    
    fn begin_renderpass(&mut self) {
        unimplemented!()
    }
    
    fn blit_image(&mut self) -> () {
        unimplemented!()
    }
    
    fn resolve_image(&mut self) -> () {
        unimplemented!()
    }
    
    fn bind_index_buffer(&mut self, buffer: native::Buffer, index_type: IndexType) {
        unimplemented!()
    }
    
    fn bind_vertex_buffers(&mut self, vbs: pso::VertexBufferSet<Resources>) {
        unimplemented!()
    }

    fn set_viewports(&mut self, viewports: &[target::Rect]) {
        let viewports = viewports.iter().map(|viewport| {
            winapi::D3D12_VIEWPORT {
                TopLeftX: viewport.x as FLOAT,
                TopLeftY: viewport.y as FLOAT,
                Width: viewport.w as FLOAT,
                Height: viewport.h as FLOAT,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            }
        }).collect::<Vec<_>>();
        unsafe {
            self.inner.RSSetViewports(viewports.len() as u32, viewports.as_ptr());
        }
    }

    fn set_scissors(&mut self, scissors: &[target::Rect]) {
        let scissors = scissors.iter().map(|rect| {
            winapi::D3D12_RECT {
                left: rect.x as INT,
                top: rect.y as INT,
                right: (rect.x + rect.w) as INT,
                bottom: (rect.y + rect.h) as INT
            }
        }).collect::<Vec<_>>();
        unsafe {
            self.inner.RSSetScissorRects(scissors.len() as u32, scissors.as_ptr());
        }
    }
    
    fn set_ref_values(&mut self, rv: state::RefValues) {
        if rv.stencil.0 != rv.stencil.1 {
            error!("Unable to set different stencil ref values for front ({}) and back ({})",
                rv.stencil.0, rv.stencil.1);
        }
        unsafe {
            self.inner.OMSetStencilRef(rv.stencil.0 as UINT);
            self.inner.OMSetBlendFactor(&rv.blend);
        }
    }

    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        unsafe {
            self.inner.Dispatch(x, y, z);
        }
    }
    
    fn dispatch_indirect(&mut self) -> () {
        unimplemented!()
    }

    fn clear_color(&mut self, rtv: native::ImageView, color: ClearColor) -> () {
        unimplemented!()
    }

    fn fill_buffer(&mut self) -> () {
        unimplemented!()
    }

    fn bind_pipeline(&mut self, pso: native::Pipeline) {
        unimplemented!()
    }
    
    fn bind_descriptor_sets(&mut self) -> () {
        unimplemented!()
    }
    
    fn push_constants(&mut self) -> () {
        unimplemented!()
    }
    
    fn update_buffer(&mut self, buffer: native::Buffer, data: &[u8], offset: usize) -> () {
        unimplemented!()
    }

    fn copy_buffer(&mut self, src: native::Buffer, dest: native::Buffer, _: &[BufferCopy]) -> () {
        unimplemented!()
    }
    
    fn copy_image(&mut self, src: native::Image, dest: native::Image) -> () {
        unimplemented!()
    }
    
    fn copy_buffer_to_image(&mut self) -> () {
        unimplemented!()
    }
    
    fn copy_image_to_buffer(&mut self) -> () {
        unimplemented!()
    }

    fn pipeline_barrier(&mut self, buffer_barriers: &[BufferBarrier], image_barriers: &[ImageBarrier]) -> () {
        unimplemented!()
    }
    
    fn execute_commands(&mut self) -> () {
        unimplemented!()
    }
}