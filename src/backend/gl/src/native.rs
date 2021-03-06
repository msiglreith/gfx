
use conv;
use core::{self, pass, pso};
use core::target::{Layer, Level};
use core::image as i;
use gl;
use Backend;
use std::cell::Cell;
use std::collections::BTreeMap;

pub type Buffer      = gl::types::GLuint;
pub type Shader      = gl::types::GLuint;
pub type Program     = gl::types::GLuint;
pub type FrameBuffer = gl::types::GLuint;
pub type Surface     = gl::types::GLuint;
pub type Texture     = gl::types::GLuint;
pub type Sampler     = gl::types::GLuint;

#[derive(Debug)]
pub struct Fence(pub Cell<gl::types::GLsync>);
unsafe impl Send for Fence {}
unsafe impl Sync for Fence {}

impl Fence {
    pub fn new(sync: gl::types::GLsync) -> Self {
        Fence(Cell::new(sync))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResourceView {
    pub object: Texture,
    pub(crate) bind: gl::types::GLenum,
    pub(crate) owned: bool,
}

impl ResourceView {
    pub fn new_texture(t: Texture, kind: i::Kind) -> ResourceView {
        ResourceView {
            object: t,
            bind: conv::image_kind_to_gl(kind),
            owned: false,
        }
    }
    pub fn new_buffer(b: Texture) -> ResourceView {
        ResourceView {
            object: b,
            bind: gl::TEXTURE_BUFFER,
            owned: true,
        }
    }
}


#[derive(Clone, Debug, Copy)]
pub struct GraphicsPipeline {
    pub program: Program,
}

#[derive(Clone, Debug, Copy)]
pub struct ComputePipeline {
    pub program: Program,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Image {
    Surface(Surface),
    Texture(Texture),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// Additionally storing the `SamplerInfo` for older OpenGL versions, which
/// don't support separate sampler objects.
pub enum FatSampler {
    Sampler(Sampler),
    Info(i::SamplerInfo),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TargetView {
    Surface(Surface),
    Texture(Texture, Level),
    TextureLayer(Texture, Level, Layer),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DescriptorSetLayout;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DescriptorSet;

#[allow(missing_copy_implementations)]
pub struct DescriptorPool {}

impl core::DescriptorPool<Backend> for DescriptorPool {
    fn allocate_sets(&mut self, layouts: &[&DescriptorSetLayout]) -> Vec<DescriptorSet> {
        layouts.iter().map(|_| DescriptorSet).collect()
    }

    fn reset(&mut self) {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Hash)]
pub struct ShaderLib {
    pub shaders: BTreeMap<pso::EntryPoint, Shader>,
}

impl ShaderLib {
    pub fn new() -> Self {
        ShaderLib {
            shaders: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct Heap;

#[derive(Debug)]
pub struct RenderPass {
    pub attachments: Vec<pass::Attachment>,
    pub subpasses: Vec<SubpassDesc>,
}

#[derive(Debug)]
pub struct SubpassDesc {
    pub color_attachments: Vec<usize>,
}

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct ConstantBufferView;
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct ShaderResourceView;
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct UnorderedAccessView;
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct RenderTargetView {
    pub view: TargetView,
}
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct DepthStencilView;
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PipelineLayout;

#[derive(Debug)]
#[allow(missing_copy_implementations)]
// No inter-queue synchronization required for GL.
pub struct Semaphore;
