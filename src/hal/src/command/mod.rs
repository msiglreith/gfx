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

/// Trait used to represent a single submission that can be added to a `Submission`.
pub unsafe trait Submit<B: Backend, C>: Send {
    ///
    unsafe fn new(buffer: B::CommandBuffer) -> Self;
    ///
    unsafe fn buffer(self) -> B::CommandBuffer;
}

/// Thread-safe finished command buffer for single submission.
pub struct SubmitOnce<B: Backend, C>(pub(crate) B::CommandBuffer, pub(crate) PhantomData<C>);
unsafe impl<B: Backend, C> Send for SubmitOnce<B, C> {}

unsafe impl<B: Backend, C> Submit<B, C> for SubmitOnce<B, C> {
    unsafe fn new(buffer: B::CommandBuffer) -> Self { 
        SubmitOnce(buffer.clone(), PhantomData)
    }

    unsafe fn buffer(self) -> B::CommandBuffer { self.0 }
}

/// Thread-safe finished command buffer for multiple submission.
#[derive(Copy, Clone)]
pub struct Resubmit<B: Backend, C>(pub(crate) B::CommandBuffer, pub(crate) PhantomData<C>);
unsafe impl<B: Backend, C> Send for Resubmit<B, C> {}

unsafe impl<B: Backend, C> Submit<B, C> for Resubmit<B, C> {
    unsafe fn new(buffer: B::CommandBuffer) -> Self { 
        Resubmit(buffer.clone(), PhantomData)
    }

    unsafe fn buffer(self) -> B::CommandBuffer { self.0 }
}

/// A dummy implementor of `Submit` used as a hack for downgraded command buffers.
/// Since these cannot be submitted, an instance of this struct should never actually exist.
pub struct SubmitDummy<B: Backend, C>(pub(crate) PhantomData<B>, pub(crate) PhantomData<C>);
unsafe impl<B: Backend, C> Send for SubmitDummy<B, C> {}

unsafe impl<B: Backend, C> Submit<B, C> for SubmitDummy<B, C> {
    unsafe fn new(_buffer: B::CommandBuffer) -> Self { 
        unreachable!()
    }

    unsafe fn buffer(self) -> B::CommandBuffer { unreachable!() }
}

/// A convenience to avoid having to type out `S = Resubmit<B, C>` every time.
pub type ReusableCommandBuffer<'a, B: Backend, C> = CommandBuffer<'a, B, C, Resubmit<B, C>>;

/// Command buffer with compute, graphics and transfer functionality.
pub struct CommandBuffer<'a, B: Backend, C, S = SubmitOnce<B, C>> 
    where S: Submit<B, C>
{
    pub(crate) raw: &'a mut B::CommandBuffer,
    _capability: PhantomData<C>,
    _submit_type: PhantomData<S>,
}

impl<'a, B: Backend, C, S: Submit<B, C>> CommandBuffer<'a, B, C, S> {
    /// Create a new typed command buffer from a raw command pool.
    pub unsafe fn new(raw: &'a mut B::CommandBuffer) -> Self {
        CommandBuffer {
            raw,
            _capability: PhantomData,
            _submit_type: PhantomData,
        }
    }

    /// Finish recording commands to the command buffers.
    ///
    /// The command buffer will be consumed and can't be modified further.
    /// The command pool must be reset to able to re-record commands.
    pub fn finish(self) -> S {
        unsafe { S::new(self.raw.clone()) }
    }

    /// Downgrade a command buffer to a lesser capability type.
    /// 
    /// This is safe as you can't `submit` downgraded version since `submit`
    /// requires `self` by move.
    pub fn downgrade<D>(&mut self) -> &mut CommandBuffer<'a, B, D, SubmitDummy<B, D>>
    where
        C: Supports<D>
    {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl<'a, B: Backend, C, S: Submit<B, C>> Drop for CommandBuffer<'a, B, C, S> {
    fn drop(&mut self) {
        self.raw.finish();
    }
}
