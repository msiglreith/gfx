//!

use Backend;
use queue::capability::Supports;
use std::marker::PhantomData;

mod compute;
mod graphics;
mod raw;
mod renderpass;
mod transfer;

pub use self::graphics::*;
pub use self::raw::RawCommandBuffer;
pub use self::renderpass::*;
pub use self::transfer::*;

/// Trait indicating how many times a Submit can be submitted.
pub trait Shot { }
/// Indicates a Submit that can only be submitted once.
pub enum OneShot { }
impl Shot for OneShot { }
/// Indicates a Submit that can be submitted multiple times.
pub enum MultiShot { }
impl Shot for MultiShot { }

/// A trait representing a command buffer that can be added to a `Submission`.
pub unsafe trait Submittable<B: Backend, C> {
    ///
    unsafe fn into_buffer(self) -> B::CommandBuffer;
}

/// Thread-safe finished command buffer for submission.
pub struct Submit<B: Backend, C, S: Shot>(pub(crate) B::CommandBuffer, pub(crate) PhantomData<C>, pub(crate) PhantomData<S>);
unsafe impl<B: Backend, C> Submittable<B, C> for Submit<B, C, OneShot> {
    unsafe fn into_buffer(self) -> B::CommandBuffer { self.0 }
}
unsafe impl<'a, B: Backend, C> Submittable<B, C> for &'a Submit<B, C, MultiShot> {
    unsafe fn into_buffer(self) -> B::CommandBuffer { self.0.clone() }
}
unsafe impl<B: Backend, C, S: Shot> Send for Submit<B, C, S> {}

/// A convenience to avoid having to type out `MultiShot` every time.
pub type ReusableCommandBuffer<'a, B: Backend, C> = CommandBuffer<'a, B, C, MultiShot>;

/// Command buffer with compute, graphics and transfer functionality.
pub struct CommandBuffer<'a, B: Backend, C, S: Shot = OneShot> {
    pub(crate) raw: &'a mut B::CommandBuffer,
    _capability: PhantomData<C>,
    _shot_type: PhantomData<S>,
}

impl<'a, B: Backend, C, S: Shot> CommandBuffer<'a, B, C, S> {
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
    pub fn finish(self) -> Submit<B, C, S> {
        Submit(self.raw.clone(), PhantomData, PhantomData)
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

impl<'a, B: Backend, C, S: Shot> Drop for CommandBuffer<'a, B, C, S> {
    fn drop(&mut self) {
        self.raw.finish();
    }
}
