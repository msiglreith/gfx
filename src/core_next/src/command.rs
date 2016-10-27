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

use core::target;
use Resources;

pub struct BufferCopy {
    src: usize,
    dest: usize,
    size: usize,
}

pub trait CommandBuffer<R: Resources> {
    // vk: primary | inside
    fn next_subpass(&mut self) -> (); // vk: Graphics // d3d12: needs to be emulated
    // vk: primary | inside
    fn end_renderpass(&mut self) -> (); // vk: Graphics // d3d12: needs to be emulated

    // vk: primary/seconday | inside
    fn clear_attachment(&mut self) -> (); // vk: Graphics
    // vk: primary/seconday | inside
    fn draw(&mut self, start: VertexCount, count: VertexCount, Option<InstanceParams>) -> (); // vk: Graphics // d3d12: DrawInstanced
    // vk: primary/seconday | inside
    fn draw_indexed(&mut self, start: VertexCount, count: VertexCount, base: VertexCount, Option<InstanceParams>) -> (); // vk: Graphics // d3d12: DrawIndexedInstanced
    // vk: primary/seconday | inside
    fn draw_indirect(&mut self) -> (); // vk: Graphics // d3d12: ExecuteIndirect !
    // vk: primary/seconday | inside
    fn draw_indexed_indirect(&mut self) -> (); // vk: Graphics // d3d12: ExecuteIndirect !

    // vk: primary/seconday | outside // d3d12: primary
    fn clear_depth_stencil(&mut self) -> (); // vk: Graphics // d3d12: ClearDepthStencilView

    // vk: primary | outside
    fn begin_renderpass(&mut self) -> (); // vk: Graphics // d3d12: needs to be emulated
    
    // vk: primary/seconday | outside
    fn blit_image(&mut self) -> (); // vk: Graphics
    // vk: primary/seconday | outside // d3d12: primary
    fn resolve_image(&mut self) -> (); // vk: Graphics // d3d12: ResolveSubresource?

    // vk: primary/seconday | inside/outside
    fn bind_index_buffer(&mut self, R::Buffer, IndexType) -> (); // vk: Graphics // d3d12: IASetIndexBuffer
    // vk: primary/seconday | inside/outside
    fn bind_vertex_buffers(&mut self) -> (); // vk: Graphics // d3d12: IASetVertexBuffers

    // vk: primary/seconday | inside/outside // d3d12: primary
    fn set_viewports(&mut self, &[target::Rect]) -> (); // vk: Graphics // d3d12: RSSetViewports
    // vk: primary/seconday | inside/outside // d3d12: primary
    fn set_scissors(&mut self, &[target::Rect]) -> (); // vk: Graphics // d3d12: RSSetScissorRects
    // vk: primary/seconday | inside/outside
    // fn set_line_width(&mut self) -> (); // vk: Graphics // d3d12:! unsupported?
    // vk: primary/seconday | inside/outside
    // fn set_depth_bias(&mut self) -> (); // vk: Graphics // d3d12:! part of the PSO

    // vk: primary/seconday | inside/outside
    // fn set_depth_bounds(&mut self) -> (); // vk: Graphics
    // vk: primary/seconday | inside/outside
    // fn set_stencil_compare_mask(&mut self) -> (); // vk: Graphics // d3d12:! part of the PSO
    // vk: primary/seconday | inside/outside
    // fn set_stencil_write_mask(&mut self) -> (); // vk: Graphics // d3d12:! part of the PSO
    // vk: primary/seconday | inside/outside

    // Merged:
    // vk: primary/seconday | inside/outside
    // fn set_blend_constants(&mut self) -> (); // vk: Graphics // d3d12: OMSetBlendFactor
    // fn set_stencil_reference(&mut self) -> (); // vk: Graphics // d3d12: OMSetStencilRef
    fn set_ref_values(&mut self, state::RefValues);

    // vk: primary/seconday | outside
    fn dispatch(&mut self) -> (); // vk: Compute // d3d12: Dispatch
    // vk: primary/seconday | outside
    fn dispatch_indirect(&mut self) -> (); // vk: Compute // d3d12: ExecuteIndirect !

    // vk: primary/seconday | outside
    fn clear_color(&mut self, R::RenderTargetView, ClearColor) -> (); // vk: Graphics/Compute // d3d12: ClearRenderTargetView

    // vk: primary/seconday | outside
    fn fill_buffer(&mut self) -> (); // vk: Graphics/Compute

    // vk: primary/seconday | inside/outside
    fn bind_pipeline(&mut self, R::PipelineStateObject) -> (); // vk: Graphics/Compute // d3d12: SetPipelineState
    // vk: primary/seconday | inside/outside
    fn bind_descriptor_sets(&mut self) -> (); // vk: Graphics/Compute
    // vk: primary/seconday | inside/outside
    fn push_constants(&mut self) -> (); // vk: Graphics/Compute // d3d12: set root constants

    // Ignore for the moment (:
    /*
    // vk: primary/seconday | outside
    fn set_event(&mut self) -> (); // vk: Graphics/Compute // d3d12:! emulation needed
    // vk: primary/seconday | outside
    fn reset_event(&mut self) -> (); // vk: Graphics/Compute
    // vk: primary/seconday | inside/outside
    fn wait_event(&mut self) -> (); // vk: Graphics/Compute

    // vk: primary/seconday | inside/outside // d3d12: primary
    fn begin_query(&mut self) -> (); // vk: Graphics/Compute // d3d12: BeginQuery
    // vk: primary/seconday | inside/outside // d3d12: primary
    fn end_query(&mut self) -> (); // vk: Graphics/Compute // d3d12: EndQuery
    // vk: primary/seconday | outside
    fn reset_query_pool(&mut self) -> (); // vk: Graphics/Compute
    // vk: primary/seconday | inside/outside
    fn write_timestamp(&mut self) -> (); // vk: Graphics/Compute
    // vk: primary/seconday | outside
    fn copy_query_pool_results(&mut self) -> (); // vk: Graphics/Compute
    */

    // vk: primary/seconday | outside
    fn update_buffer(&mut self, R::Buffer, data: &[u8], offset: usize) -> (); // vk: Graphics/Compute/Transfer
    // vk: primary/seconday | outside // d3d12: primary
    fn copy_buffer(&mut self, src: R::Buffer, dest: R::Buffer, &[BufferCopy]) -> (); // vk: Graphics/Compute/Transfer // d3d12: CopyBufferRegion
    // vk: primary/seconday | outside // d3d12: primary
    fn copy_image(&mut self, src: R::Image, dest: R::Image) -> (); // vk: Graphics/Compute/Transfer // d3d12: CopyTextureRegion
    // vk: primary/seconday | outside // d3d12: primary
    fn copy_buffer_to_image(&mut self) -> (); // vk: Graphics/Compute/Transfer
    // vk: primary/seconday | outside // d3d12: primary
    fn copy_image_to_buffer(&mut self) -> (); // vk: Graphics/Compute/Transfer

    // vk: primary/seconday | inside/outside // d3d12: primary
    fn pipeline_barrier(&mut self) -> (); // vk: Graphics/Compute/Transfer // d3d12: ResourceBarrier
    // vk: primary | inside/outside // d3d12: primary
    fn execute_commands(&mut self) -> (); // vk: Graphics/Compute/Transfer // d3d12:! Allowed to call bundles?
}

// Semaphores
//  PresentInfoKHR - wait
//  BindSparseInfo - wait/signal
//  SubmitInfo - wait/signal
//  AcquireNextImageKHR()