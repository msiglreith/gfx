
use vk;
use core::{command, pso, state, target};
use core::{ClearColor, VertexCount, IndexType, InstanceParams};
use core::MAX_VERTEX_ATTRIBUTES;
use core::command::{BufferCopy, BufferBarrier, ImageBarrier};
use {Resources, Device, SharePointer};
use native;
use std::sync::Arc;

pub struct Buffer {
    inner: vk::CommandBuffer,
    device: Arc<Device>,
}

impl Buffer {
    #[doc(hidden)]
    pub fn new(buffer: vk::CommandBuffer, device: Arc<Device>) -> Buffer {
        unimplemented!()
    }

    #[doc(hidden)]
    pub fn get(&self) -> vk::CommandBuffer {
        self.inner
    }
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
        let (_, vk) = self.device.get();
        let (instance_count, instance_start) = instance.unwrap_or((1, 0));
        unsafe {
            vk.CmdDraw(
                self.inner,
                vertex_count,
                instance_count,
                vertex_start,
                instance_start
            );
        }
    }

    fn draw_indexed(&mut self, index_start: VertexCount, index_count: VertexCount, vertex_base: VertexCount, instance: Option<InstanceParams>) {
        let (_, vk) = self.device.get();
        let (instance_count, instance_start) = instance.unwrap_or((1, 0));
        unsafe {
            vk.CmdDrawIndexed(
                self.inner,
                index_count,
                instance_count,
                index_start,
                vertex_base as i32,
                instance_start
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
        let (_, vk) = self.device.get();
        let index_type = match index_type {
            IndexType::U16 => vk::INDEX_TYPE_UINT16,
            IndexType::U32 => vk::INDEX_TYPE_UINT32,
        };
        unsafe {
            vk.CmdBindIndexBuffer(
                self.inner,
                buffer.buffer,
                0,
                index_type,
            );
        }
    }
    
    fn bind_vertex_buffers(&mut self, vbs: pso::VertexBufferSet<Resources>) {
        let (_, vk) = self.device.get();
        let mut buffers = [0; MAX_VERTEX_ATTRIBUTES];
        let mut offsets = [0u64; MAX_VERTEX_ATTRIBUTES];
        for i in 0 .. MAX_VERTEX_ATTRIBUTES {
            if let Some((buffer, offset)) = vbs.0[i] {
                buffers[i] = buffer.buffer;
                offsets[i] = offset as u64;
            }
            // TODO: error if sth is missing?
        }

        unsafe {
            vk.CmdBindVertexBuffers(
                self.inner,
                0,
                buffers.len() as u32,
                buffers.as_ptr(),
                offsets.as_ptr(),
            )
        }
    }

    fn set_viewports(&mut self, viewports: &[target::Rect]) {
        let (_, vk) = self.device.get();
        let viewports = viewports.iter().map(|viewport| {
            vk::Viewport {
                x: viewport.x as f32,
                y: viewport.y as f32,
                width: viewport.w as f32,
                height: viewport.h as f32,
                minDepth: 0.0,
                maxDepth: 1.0,
            }
        }).collect::<Vec<_>>();

        unsafe {
            vk.CmdSetViewport(
                self.inner,
                0,
                viewports.len() as u32,
                viewports.as_ptr(),
            );
        }
    }

    fn set_scissors(&mut self, scissors: &[target::Rect]) {
        let (_, vk) = self.device.get();
        let scissors = scissors.iter().map(|scissor| {
            vk::Rect2D {
                offset: vk::Offset2D {
                    x: scissor.x as i32,
                    y: scissor.y as i32,
                },
                extent: vk::Extent2D {
                    width: scissor.w as u32,
                    height: scissor.h as u32,
                },
            }
        }).collect::<Vec<_>>();

        unsafe {
            vk.CmdSetScissor(
                self.inner,
                0,
                scissors.len() as u32,
                scissors.as_ptr(),
            );
        }
    }
    
    fn set_ref_values(&mut self, _: state::RefValues) {
        unimplemented!()
    }

    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        let (_, vk) = self.device.get();
        unsafe {
            vk.CmdDispatch(self.inner, x, y, z);
        }
    }
    
    fn dispatch_indirect(&mut self) -> () {
        unimplemented!()
    }

    fn clear_color(&mut self, rtv: native::ImageView, color: ClearColor) -> () {
        let (_, vk) = self.device.get();
        let value = match color {
            ClearColor::Float(v) => vk::ClearColorValue::float32(v),
            ClearColor::Int(v)   => vk::ClearColorValue::int32(v),
            ClearColor::Uint(v)  => vk::ClearColorValue::uint32(v),
        };
        unsafe {
            vk.CmdClearColorImage(self.inner, rtv.image, rtv.layout, &value, 1, &rtv.sub_range);
        }
    }

    fn fill_buffer(&mut self) -> () {
        unimplemented!()
    }

    fn bind_pipeline(&mut self, pso: native::Pipeline) {
        let (_, vk) = self.device.get();
        unsafe {
            vk.CmdBindPipeline(self.inner, vk::PIPELINE_BIND_POINT_GRAPHICS, pso.pipeline); // TODO: differ between graphics/compute
        }
    }
    
    fn bind_descriptor_sets(&mut self) -> () {
        unimplemented!()
    }
    
    fn push_constants(&mut self) -> () {
        unimplemented!()
    }
    
    fn update_buffer(&mut self, buffer: native::Buffer, data: &[u8], offset: usize) -> () {
        let (_, vk) = self.device.get();
        unsafe {
            vk.CmdUpdateBuffer(self.inner, buffer.buffer, offset as u64, data.len() as u64, data.as_ptr() as *const u32);
        }
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
