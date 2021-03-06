//! Input Assembler(IA) stage description.

use format;
use {Backend, Primitive};

/// Shader binding location.
pub type Location = u32;
/// Index of a vertex buffer.
pub type BufferIndex = u32;
/// Offset of an attribute from the start of the buffer, in bytes
pub type ElemOffset = u32;
/// Offset between attribute values, in bytes
pub type ElemStride = u32;
/// The number of instances between each subsequent attribute value
pub type InstanceRate = u8;
/// An offset inside a vertex buffer, in bytes.
pub type BufferOffset = usize;

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
    /// Total container size, in bytes.
    /// Specifies the byte distance between two consecutive elements.
    pub stride: ElemStride,
    /// Rate of the input for the given buffer
    pub rate: InstanceRate,
}

/// PSO vertex attribute descriptor
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct AttributeDesc {
    /// Attribute binding location in the shader.
    pub location: Location,
    /// Index of the associated vertex buffer descriptor.
    pub binding: BufferIndex,
    /// Attribute element description.
    pub element: Element<format::Format>,
}

///
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum PrimitiveRestart {
    ///
    Disabled,
    ///
    U16,
    ///
    U32,
}

///
pub struct InputAssemblerDesc {
    /// Type of the primitive
    pub primitive: Primitive,
    ///
    pub primitive_restart: PrimitiveRestart,
}

impl InputAssemblerDesc {
    /// Create a new IA descriptor without primitive restart
    pub fn new(primitive: Primitive) -> Self {
        InputAssemblerDesc {
            primitive,
            primitive_restart: PrimitiveRestart::Disabled,
        }
    }
}

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
