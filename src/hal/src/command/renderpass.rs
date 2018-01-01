use std::borrow::Borrow;
use std::ops::Range;
use {pso, Backend, IndexCount, InstanceCount, VertexCount, VertexOffset};
use buffer::IndexBufferView;
use queue::{Supports, Graphics};
use super::{
    ColorValue, StencilValue, Rect, Viewport,
    AttachmentClear, ClearValue, CommandBuffer, RawCommandBuffer, Shot
};

/// Specifies how commands for the following renderpasses will be recorded.
pub enum SubpassContents {
    ///
    Inline,
    ///
    SecondaryBuffers,
}

// This macro contains all methods for all commands submittable during a subpass.
// It is used to implement the identical portions of RenderPassInlineEncoder and SubpassCommandBuffer's API.
macro_rules! add_subpass_commands { () => {
    pub fn clear_attachments<T, U>(&mut self, clears: T, rects: U)
    where
        T: IntoIterator,
        T::Item: Borrow<AttachmentClear>,
        U: IntoIterator,
        U::Item: Borrow<Rect>,
    {
        self.0.clear_attachments(clears, rects)
    }

    ///
    pub fn draw(&mut self, vertices: Range<VertexCount>, instances: Range<InstanceCount>) {
        self.0.draw(vertices, instances)
    }
    ///
    pub fn draw_indexed(&mut self, indices: Range<IndexCount>, base_vertex: VertexOffset, instances: Range<InstanceCount>) {
        self.0.draw_indexed(indices, base_vertex, instances)
    }
    ///
    pub fn draw_indirect(&mut self, buffer: &B::Buffer, offset: u64, draw_count: u32, stride: u32) {
        self.0.draw_indirect(buffer, offset, draw_count, stride)
    }
    ///
    pub fn draw_indexed_indirect(&mut self, buffer: &B::Buffer, offset: u64, draw_count: u32, stride: u32) {
        self.0.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }

    /// Bind index buffer view.
    pub fn bind_index_buffer(&mut self, ibv: IndexBufferView<B>) {
        self.0.bind_index_buffer(ibv)
    }

    /// Bind vertex buffers.
    pub fn bind_vertex_buffers(&mut self, vbs: pso::VertexBufferSet<B>) {
        self.0.bind_vertex_buffers(vbs);
    }

    /// Bind a graphics pipeline.
    pub fn bind_graphics_pipeline(&mut self, pipeline: &B::GraphicsPipeline) {
        self.0.bind_graphics_pipeline(pipeline)
    }

    ///
    pub fn bind_graphics_descriptor_sets<'i, T>(
        &mut self,
        layout: &B::PipelineLayout,
        first_set: usize,
        sets: T,
    ) where
        T: IntoIterator,
        T::Item: Borrow<B::DescriptorSet>,
    {
        self.0.bind_graphics_descriptor_sets(layout, first_set, sets)
    }

    ///
    pub fn set_viewports<T>(&mut self, viewports: T)
    where
        T: IntoIterator,
        T::Item: Borrow<Viewport>,
    {
        self.0.set_viewports(viewports)
    }

    ///
    pub fn set_scissors<T>(&mut self, scissors: T)
    where
        T: IntoIterator,
        T::Item: Borrow<Rect>,
    {
        self.0.set_scissors(scissors)
    }

    ///
    pub fn set_stencil_reference(&mut self, front: StencilValue, back: StencilValue) {
        self.0.set_stencil_reference(front, back)
    }

    ///
    pub fn set_blend_constants(&mut self, cv: ColorValue) {
        self.0.set_blend_constants(cv)
    }

    ///
    pub fn push_graphics_constants(&mut self, layout: &B::PipelineLayout, stages: pso::ShaderStageFlags, offset: u32, constants: &[u32]) {
        self.0.push_graphics_constants(layout, stages, offset, constants);
    }

    // TODO: set_line_width
    // TODO: set_depth_bounds
    // TODO: set_depth_bias
    // TODO: set_stencil_compare_mask
    // TODO: set_stencil_write_mask
    // TODO: pipeline barrier (postponed)
    // TODO: begin/end query
}}

///
pub struct RenderPassInlineEncoder<'a, B: Backend, L: Level>(pub(crate) &'a mut B::CommandBuffer, PhantomData<L>)
where B::CommandBuffer: 'a;

impl<'a, B: Backend, L: Level> RenderPassInlineEncoder<'a, B, L> {
    ///
    fn new<C, T, S: Shot>(
        cmd_buffer: &'a mut CommandBuffer<B, C, S, L>,
        render_pass: &B::RenderPass,
        frame_buffer: &B::Framebuffer,
        render_area: Rect,
        clear_values: T,
    ) -> Self
    where
        C: Supports<Graphics>,
        T: IntoIterator,
        T::Item: Borrow<ClearValue>,
    {
        cmd_buffer.raw.begin_renderpass(
            render_pass,
            frame_buffer,
            render_area,
            clear_values,
            SubpassContents::Inline);
        RenderPassInlineEncoder(cmd_buffer.raw)
    }

    ///
    pub fn next_subpass_inline(self) -> Self {
        self.0.next_subpass(SubpassContents::Inline);
        self
    }

    add_subpass_commands!();
}

impl<'a, B: Backend> RenderPassInlineEncoder<'a, B, Primary> {
    pub fn next_subpass_secondary(self) -> RenderPassSecondaryEncoder<B> {
        self.0.next_subpass(SubpassContents::SecondaryBuffers);
        RenderPassSecondaryEncoder(self.0)
    }
}

impl<'a, B: Backend, L: Level> Drop for RenderPassInlineEncoder<'a, B, L> {
    fn drop(&mut self) {
        self.0.end_renderpass();
    }
}

///
pub struct RenderPassSecondaryEncoder<'a, B: Backend>(pub(crate) &'a mut B::CommandBuffer)
where B::CommandBuffer: 'a;

impl<'a, B: Backend> RenderPassSecondaryEncoder<'a, B> {
    ///
    pub fn new<C, T, S: Shot>(
        cmd_buffer: &'a mut CommandBuffer<B, C, S, Primary>,
        render_pass: &B::RenderPass,
        frame_buffer: &B::Framebuffer,
        render_area: Rect,
        clear_values: T,
    ) -> Self
    where
        C: Supports<Graphics>,
        T: IntoIterator,
        T::Item: Borrow<ClearValue>,
    {
        cmd_buffer.raw.begin_renderpass(
            render_pass,
            frame_buffer,
            render_area,
            clear_values,
            SubpassContents::SecondaryBuffers);
        RenderPassSecondaryEncoder(cmd_buffer.raw)
    }

    ///
    pub fn next_subpass_inline(self) -> RenderPassInlineEncoder<B, Primary> {
        self.0.next_subpass(SubpassContents::Inline);
        RenderPassInlineEncoder(self.0)
    }

    pub fn next_subpass_secondary(self) -> Self {
        self.0.next_subpass(SubpassContents::SecondaryBuffers);
        self
    }

    pub fn execute_commands<I, S>(&mut self, submits: I) 
    where
        I: Iterator<Item=S>,
        S: Submittable<B, Subpass, Secondary>,
    {
        self.0.execute_commands(&submits
            .map(|submit| unsafe { submit.into_buffer() })
            .collect::<Vec<_>>()
        );
    }
}

/// Capability used only for subpass command buffers' Submits.
pub enum Subpass { }

pub struct SubpassCommandBuffer<B: Backend, S: Shot>(pub(crate) B::CommandBuffer);
impl<B: Backend, S: Shot> SubpassCommandBuffer<B, S> {
    add_subpass_commands!();

    pub fn finish(self) -> Submit<B, Subpass, S, Secondary> {
        Submit(self.0.clone(), PhantomData, PhantomData, PhantomData)
    }
}

impl<B: Backend, S: Shot> Drop for SubpassCommandBuffer<B, S> {
    fn drop(&mut self) {
        self.0.finish();
    }
}

