
use vk;
use core_next::command;
use Resources;

pub struct Buffer {
    inner: vk::CommandBuffer,
    share: SharePointer,
}

impl command::Buffer<Resources> for Buffer {
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
        let (_, vk) = self.share.get_device();
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

    fn draw_indexed(&mut self, index_start: VertexCount, index_count: VertexCount, vertex_base: VertexCount, Option<InstanceParams>) {
        let (_, vk) = self.share.get_device();
        let (instance_count, instance_start) = instance.unwrap_or((1, 0));
        unsafe {
            vk.CmdDrawIndexed(
                self.inner,
                index_count,
                instance_count,
                index_start,
                vertex_base,
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

    fn clear_depth_stencil(&mut self, R::DepthStencilView,
                           Option<target::Depth>, Option<target::Stencil>);
    fn begin_renderpass(&mut self);
    fn blit_image(&mut self) -> ();
    fn resolve_image(&mut self) -> ();
    fn bind_index_buffer(&mut self, buffer: R::Buffer, index_type: IndexType) {
        let (_, vk) = self.share.get_device();
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
    fn bind_vertex_buffers(&mut self, pso::VertexBufferSet<R>) {
        let (_, vk) = self.share.get_device();
        let mut buffers = [native::Buffer(ptr::null_mut()); MAX_VERTEX_ATTRIBUTES];
        let mut offsets = [0; MAX_VERTEX_ATTRIBUTES];
        for i in 0 .. MAX_VERTEX_ATTRIBUTES {
            if let Some((buffer, offset)) = vbs.0[i] {
                buffers[i] = buffer.0;
                offsets[i] = offset as UINT;
            }
            // TODO: error if sth is missing?
        }

        unsafe {
            vk.CmdBindVertexBuffers(
                self.inner,
                0,
                buffers.len(),
                &buffers,
                &offsets,
            )
        }
    }

    fn set_viewports(&mut self, viewports: &[target::Rect]) {
        let (_, vk) = self.share.get_device();
        let viewports = viewports.iter().map(|viewport| {
            vk::Viewport {
                x: viewport.x,
                y: viewport.y,
                width: viewport.w,
                height: viewport.h,
                minDepth: 0.0,
                maxDepth: 1.0,
            }
        }).collect::<Vec<_>>();

        unsafe {
            vk.CmdSetViewport(
                self.inner,
                0,
                viewports.len(),
                &viewports,
            );
        }
    }

    fn set_scissors(&mut self, scissors: &[target::Rect]) {
        let (_, vk) = self.share.get_device();
        let scissors = scissors.iter().map(|scissor| {
            vk::Rect2D {
                offset: vk::Offset2D {
                    x: scissor.x,
                    y: scissor.y,
                },
                extent: vk::Extent2D {
                    width: scissor.w,
                    height: scissor.h,
                },
            }
        }).collect::<Vec<_>>();

        unsafe {
            vk.CmdSetScissor(
                self.inner,
                0,
                scissors.len(),
                &scissors,
            );
        }
    }
    fn set_ref_values(&mut self, state::RefValues) {

    }

    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        let (_, vk) = self.share.get_device();
        unsafe {
            vk.CmdDispatch(self.inner, x, y, z);
        }
    }
    fn dispatch_indirect(&mut self) -> () {
        unimplemented!()
    }

    fn clear_color(&mut self, rtv: R::RenderTargetView, color: ClearColor) -> () {
        let (_, vk) = self.share.get_device();
        let value = match color {
            command::ClearColor::Float(v) => vk::ClearColorValue::float32(v),
            command::ClearColor::Int(v)   => vk::ClearColorValue::int32(v),
            command::ClearColor::Uint(v)  => vk::ClearColorValue::uint32(v),
        };
        unsafe {
            vk.CmdClearColorImage(self.inner, rtv.image, rtv.layout, &value, 1, &rtv.sub_range);
        }
    }

    fn fill_buffer(&mut self) -> () {

    }

    fn bind_pipeline(&mut self, R::PipelineStateObject) {
        let (_, vk) = self.share.get_device();
        unsafe {
            vk.CmdBindPipeline(self.inner, vk::PIPELINE_BIND_POINT_GRAPHICS, pso.pipeline); // TODO: differ between graphics/compute
        }
    }
    fn bind_descriptor_sets(&mut self) -> ();
    fn push_constants(&mut self) -> ();
    fn update_buffer(&mut self, buffer, R::Buffer, data: &[u8], offset: usize) -> () {
        let (_, vk) = self.share.get_device();
        unsafe {
            vk.CmdUpdateBuffer(self.inner, buffer.buffer, offset, data.len(), data);
        }
    }

    fn copy_buffer(&mut self, src: R::Buffer, dest: R::Buffer, &[BufferCopy]) -> ();
    fn copy_image(&mut self, src: R::Image, dest: R::Image) -> ();
    fn copy_buffer_to_image(&mut self) -> ();
    fn copy_image_to_buffer(&mut self) -> ();

    fn pipeline_barrier(&mut self) -> ();
    fn execute_commands(&mut self) -> ();
}
