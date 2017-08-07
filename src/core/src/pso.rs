//! Raw Pipeline State Objects
//!
//! This module contains items used to create and manage a raw pipeline state object. Most users
//! will want to use the typed and safe `PipelineState`. See the `pso` module inside the `gfx`
//! crate.

use {MAX_COLOR_TARGETS, MAX_VERTEX_ATTRIBUTES, MAX_CONSTANT_BUFFERS,
     MAX_RESOURCE_VIEWS, MAX_UNORDERED_VIEWS, MAX_SAMPLERS};
use {ConstantBufferSlot, ColorSlot, ResourceViewSlot,
     UnorderedViewSlot, SamplerSlot,
     Primitive, Backend};
use {format, state as s, texture};
use shade::Usage;
use std::error::Error;
use std::fmt;

/// Maximum number of vertex buffers used in a PSO definition.
pub const MAX_VERTEX_BUFFERS: usize = 4;

/// An offset inside a vertex buffer, in bytes.
pub type BufferOffset = usize;

/// Error types happening upon PSO creation on the device side.
#[derive(Clone, Debug, PartialEq)]
pub struct CreationError;

impl fmt::Display for CreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        "Could not create PSO on device."
    }
}

/// Color output configuration of the PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ColorInfo {
    /// Color channel mask
    pub mask: s::ColorMask,
    /// Optional color blending
    pub color: Option<s::BlendChannel>,
    /// Optional alpha blending
    pub alpha: Option<s::BlendChannel>,
}
impl From<s::ColorMask> for ColorInfo {
    fn from(mask: s::ColorMask) -> ColorInfo {
        ColorInfo {
            mask: mask,
            color: None,
            alpha: None,
        }
    }
}
impl From<s::Blend> for ColorInfo {
    fn from(blend: s::Blend) -> ColorInfo {
        ColorInfo {
            mask: s::MASK_ALL,
            color: Some(blend.color),
            alpha: Some(blend.alpha),
        }
    }
}

/// Depth and stencil state of the PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct DepthStencilInfo {
    /// Optional depth test configuration
    pub depth: Option<s::Depth>,
    /// Optional stencil test on the front faces
    pub front: Option<s::StencilSide>,
    /// Optional stencil test on the back faces
    pub back: Option<s::StencilSide>,
}
impl From<s::Depth> for DepthStencilInfo {
    fn from(depth: s::Depth) -> DepthStencilInfo {
        DepthStencilInfo {
            depth: Some(depth),
            front: None,
            back: None,
        }
    }
}
impl From<s::Stencil> for DepthStencilInfo {
    fn from(stencil: s::Stencil) -> DepthStencilInfo {
        DepthStencilInfo {
            depth: None,
            front: Some(stencil.front),
            back: Some(stencil.back),
        }
    }
}
impl From<(s::Depth, s::Stencil)> for DepthStencilInfo {
    fn from(ds: (s::Depth, s::Stencil)) -> DepthStencilInfo {
        DepthStencilInfo {
            depth: Some(ds.0),
            front: Some(ds.1.front),
            back: Some(ds.1.back),
        }
    }
}

/// Index of a vertex buffer.
pub type BufferIndex = u8;
/// Offset of an attribute from the start of the buffer, in bytes
pub type ElemOffset = u32;
/// Offset between attribute values, in bytes
pub type ElemStride = u8;
/// The number of instances between each subsequent attribute value
pub type InstanceRate = u8;

/// A struct element descriptor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Element<F> {
    /// Element format
    pub format: F,
    /// Offset from the beginning of the container, in bytes
    pub offset: ElemOffset,
}

/// Vertex buffer descriptor
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct VertexBufferDesc {
    /// Total container size, in bytes
    pub stride: ElemStride,
    /// Rate of the input for the given buffer
    pub rate: InstanceRate,
}

/// PSO vertex attribute descriptor
pub type AttributeDesc = (BufferIndex, Element<format::Format>);
/// PSO constant buffer descriptor
pub type ConstantBufferDesc = Usage;
/// PSO shader resource view descriptor
pub type ResourceViewDesc = Usage;
/// PSO unordered access view descriptor
pub type UnorderedViewDesc = Usage;
/// PSO sampler descriptor
pub type SamplerDesc = Usage;
/// PSO color target descriptor
pub type ColorTargetDesc = (format::Format, ColorInfo);
/// PSO depth-stencil target descriptor
pub type DepthStencilDesc = (format::Format, DepthStencilInfo);

/// A complete set of vertex buffers to be used for vertex import in PSO.
#[derive(Clone, Debug)]
pub struct VertexBufferSet<'a, B: Backend>(
    /// Array of buffer handles with offsets in them
    pub Vec<(&'a B::Buffer, BufferOffset)>,
);

impl<'a, B: Backend> VertexBufferSet<'a, B> {
    /// Create an empty set
    pub fn new() -> VertexBufferSet<'a, B> {
        VertexBufferSet(Vec::new())
    }
}

/// A constant buffer run-time parameter for PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConstantBufferParam<B: Backend>(pub B::Buffer, pub Usage, pub ConstantBufferSlot);

/// A shader resource view (SRV) run-time parameter for PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourceViewParam<B: Backend>(pub B::ShaderResourceView, pub Usage, pub ResourceViewSlot);

/// An unordered access view (UAV) run-time parameter for PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnorderedViewParam<B: Backend>(pub B::UnorderedAccessView, pub Usage, pub UnorderedViewSlot);

/// A sampler run-time parameter for PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SamplerParam<B: Backend>(pub B::Sampler, pub Usage, pub SamplerSlot);

/// Shader entry point.
pub type EntryPoint = &'static str;

/// A complete set of shaders to build a graphics pipeline.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GraphicsShaderSet {
    ///
    pub vertex_shader: EntryPoint,
    ///
    pub hull_shader: Option<EntryPoint>,
    ///
    pub domain_shader: Option<EntryPoint>,
    ///
    pub geometry_shader: Option<EntryPoint>,
    ///
    pub pixel_shader: Option<EntryPoint>,
}

///
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct GraphicsPipelineDesc {
    /// Type of the primitive
    pub primitive: Primitive,
    /// Rasterizer setup
    pub rasterizer: s::Rasterizer,
    /// Depth stencil
    pub depth_stencil: Option<DepthStencilDesc>,
    /// Shader entry points
    pub shader_entries: GraphicsShaderSet,
    /// Render target views (RTV)
    /// The entries are supposed to be contiguous, starting from 0
    pub color_targets: [Option<ColorTargetDesc>; MAX_COLOR_TARGETS],
    /// Vertex buffers
    pub vertex_buffers: Vec<VertexBufferDesc>,
    /// Vertex attributes
    pub attributes: Vec<AttributeDesc>,
}

impl GraphicsPipelineDesc {
    /// Create a new empty PSO descriptor.
    pub fn new(primitive: Primitive, rasterizer: s::Rasterizer, shader_entries: GraphicsShaderSet) -> GraphicsPipelineDesc {
        GraphicsPipelineDesc {
            primitive: primitive,
            rasterizer: rasterizer,
            depth_stencil: None,
            shader_entries: shader_entries,
            color_targets: [None; MAX_COLOR_TARGETS],
            vertex_buffers: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

bitflags!(
    /// Stages of the logical pipeline.
    ///
    /// The pipeline is structured as given the by the ordering of the flags.
    /// Some stages are queue type dependent.
    pub flags PipelineStage: u32 {
        /// Beginning of the command queue.
        const TOP_OF_PIPE = 0x1,
        /// Indirect data consumption.
        const DRAW_INDIRECT = 0x2,
        /// Vertex data consumption.
        const VERTEX_INPUT = 0x4,
        /// Vertex shader execution.
        const VERTEX_SHADER = 0x8,
        /// Hull shader execution.
        const HULL_SHADER = 0x10,
        /// Domain shader execution.
        const DOMAIN_SHADER = 0x20,
        /// Geometry shader execution.
        const GEOMETRY_SHADER = 0x40,
        /// Pixel shader execution.
        const PIXEL_SHADER = 0x80,
        /// Stage of early depth and stencil test.
        const EARLY_FRAGMENT_TESTS = 0x100,
        /// Stage of late depth and stencil test.
        const LATE_FRAGMENT_TESTS = 0x200,
        /// Stage of final color value calculation.
        const COLOR_ATTACHMENT_OUTPUT = 0x400,
        /// Compute shader execution,
        const COMPUTE_SHADER = 0x800,
        /// Copy/Transfer command execution.
        const TRANSFER = 0x1000,
        /// End of the command queue.
        const BOTTOM_OF_PIPE = 0x2000,
        /// Read/Write access from host.
        /// (Not a real pipeline stage)
        const HOST = 0x4000,
    }
);
