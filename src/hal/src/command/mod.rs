//!

use Backend;
use queue::capability::Supports;
use std::marker::PhantomData;

mod compute;
mod graphics;
mod raw;
mod renderpass;
mod transfer;

use self::raw::CommandBufferFlags;

pub use self::graphics::*;
pub use self::raw::RawCommandBuffer;
pub use self::renderpass::*;
pub use self::transfer::*;

/// Trait indicating how many times a Submit can be submitted.
pub trait Shot { 
    fn once() -> raw::CommandBufferFlags;
}
/// Indicates a Submit that can only be submitted once.
pub enum OneShot { }
impl Shot for OneShot { fn flags() -> CommandBufferFlags { CommandBufferFlags::ONE_TIME_SUBMIT } }
/// Indicates a Submit that can be submitted multiple times.
pub enum MultiShot { }
impl Shot for MultiShot { fn flags() -> CommandBufferFlags { Default::default() } }

/// A trait indicating the level of a command buffer.
pub trait Level { }
pub enum Primary { }
impl Level for Primary { }
pub enum Secondary { }
impl Level for Secondary { }

/// A trait representing a command buffer that can be added to a `Submission`.
pub unsafe trait Submittable<B: Backend, C, L: Level> {
    ///
    unsafe fn get_buffer(self) -> B::CommandBuffer;
}

/// Thread-safe finished command buffer for submission.
pub struct Submit<B: Backend, C, S: Shot, L: Level> {
    pub(crate) buffer:  B::CommandBuffer,
    pub(crate) _capability: PhantomData<C>,
    pub(crate) _shot: PhantomData<S>,
    pub(crate) _level: PhantomData<L>
}
unsafe impl<B: Backend, C, L: Level> Submittable<B, C, L> for Submit<B, C, OneShot, L> {
    unsafe fn into_buffer(self) -> B::CommandBuffer { self.buffer }
}
unsafe impl<'a, B: Backend, C, L: Level> Submittable<B, C, L> for &'a Submit<B, C, MultiShot, L> {
    unsafe fn into_buffer(self) -> B::CommandBuffer { self.buffer.clone() }
}
unsafe impl<B: Backend, C, S: Shot, L: Level> Send for Submit<B, C, S, L> {}

/// A convenience for not typing out the full signature of a secondary command buffer.
pub type SecondaryCommandBuffer<'a, B: Backend, C, S: Shot = OneShot> = CommandBuffer<'a, B, C, S, Secondary>;

/// Command buffer with compute, graphics and transfer functionality.
pub struct CommandBuffer<'a, B: Backend, C, S: Shot = OneShot, L: Level = Primary> {
    pub(crate) raw: &'a mut B::CommandBuffer,
    _capability: PhantomData<C>,
    _shot_type: PhantomData<S>,
    _level: PhantomData<L>,
}

impl<'a, B: Backend, C, S: Shot, L: Level> CommandBuffer<'a, B, C, S, L> {
    /// Create a new typed command buffer from a raw command pool.
    pub unsafe fn new(raw: &'a mut B::CommandBuffer) -> Self {
        CommandBuffer {
            raw,
            _capability: PhantomData,
            _shot_type: PhantomData,
        }
    }

    /// Finish recording commands to the command buffers.
    ///
    /// The command buffer will be consumed and can't be modified further.
    /// The command pool must be reset to able to re-record commands.
    pub fn finish(self) -> Submit<B, C, S, L> {
        Submit(self.raw, PhantomData, PhantomData, PhantomData)
    }

    /// Downgrade a command buffer to a lesser capability type.
    /// 
    /// This is safe as you can't `submit` downgraded version since `submit`
    /// requires `self` by move.
    pub fn downgrade<D>(&mut self) -> &mut CommandBuffer<'a, B, D, S>
    where
        C: Supports<D>
    {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl<'a, B: Backend, C, S: Shot> CommandBuffer<'a, B, C, S, Primary> {
    ///
    pub fn execute_commands<I, S, K>(&mut self, submits: I) 
    where
        I: Iterator<Item=S>,
        S: Submittable<B, K, Secondary>,
        C: Supports<K>,
    {
        self.0.execute_commands(&submits
            .map(|submit| unsafe { submit.into_buffer() })
            .collect::<Vec<_>>()
        );
    }
}

impl<'a, B: Backend, C, S: Shot, L: Level> Drop for CommandBuffer<'a, B, C, S, L> {
    fn drop(&mut self) {
        self.raw.finish();
    }
}
