//! Queue submission.
//!
//! // TODO

use {pso, Backend};
use command::{Submit, OneShot, MultiShot, Primary};
use super::capability::{Transfer, Supports, Upper};
use std::borrow::Borrow;
use std::marker::PhantomData;
use smallvec::SmallVec;

/// Raw submission information for a command queue.
pub struct RawSubmission<'a, B: Backend + 'a, IC>
where
    IC: IntoIterator,
    IC::Item: Borrow<B::CommandBuffer>,
{
    /// Command buffers to submit.
    pub cmd_buffers: IC,
    /// Semaphores to wait being signalled before submission.
    pub wait_semaphores: &'a [(&'a B::Semaphore, pso::PipelineStage)],
    /// Semaphores which get signalled after submission.
    pub signal_semaphores: &'a [&'a B::Semaphore],
}

enum Submits<'a, B: Backend> {
    Owned(B::CommandBuffer),
    Ref(&'a B::CommandBuffer),
}

/// Submission information for a command queue.
pub struct Submission<'a, B: Backend, C> {
    cmd_buffers: SmallVec<[Submits<'a, B>; 16]>,
    wait_semaphores: SmallVec<[(&'a B::Semaphore, pso::PipelineStage); 16]>,
    signal_semaphores: SmallVec<[&'a B::Semaphore; 16]>,
    marker: PhantomData<C>,
}

impl<'a, B: Backend> Submission<'a, B, Transfer> {
    /// Create a new empty (transfer) submission.
    ///
    /// Transfer is the minimum supported capability by all queues.
    pub fn new() -> Self {
        Submission {
            cmd_buffers: SmallVec::new(),
            wait_semaphores: SmallVec::new(),
            signal_semaphores: SmallVec::new(),
            marker: PhantomData,
        }
    }
}

impl<'a, B, C> Submission<'a, B, C>
where
    B: Backend
{
    /// Set semaphores which will waited on to be signalled before the submission will be executed.
    pub fn wait_on(mut self, semaphores: &[(&'a B::Semaphore, pso::PipelineStage)]) -> Self {
        self.wait_semaphores.extend_from_slice(semaphores);
        self
    }

    /// Set semaphores which will be signalled once this submission has finished executing.
    pub fn signal(mut self, semaphores: &[&'a B::Semaphore]) -> Self {
        self.signal_semaphores.extend_from_slice(semaphores);
        self
    }

    /// Convert strong-typed submission object into untyped equivalent.
    pub(super) fn as_raw(&self) -> RawSubmission<B, Vec<&B::CommandBuffer>> {
        let cmd_buffers = self.cmd_buffers.iter().map(|buffer| {
            match *buffer {
                Submits::Owned(ref buffer) => buffer,
                Submits::Ref(buffer) => buffer,
            }
        }).collect::<Vec<_>>();
        RawSubmission {
            cmd_buffers: cmd_buffers,
            wait_semaphores: &self.wait_semaphores,
            signal_semaphores: &self.signal_semaphores,
        }
    }

    /// Append a new list of finished command buffers to this submission.
    ///
    /// All submits for this call must be of the same type.
    /// Submission will be automatically promoted to to the minimum required capability
    /// to hold all passed submits.
    pub fn submit<I, K>(mut self, submits: I) -> Submission<'a, B, <(C, K) as Upper>::Result>
    where
        I: IntoIterator<Item=Submit<B, K, OneShot, Primary>>,
        (C, K): Upper
    {
        self.cmd_buffers.extend(submits.into_iter().map(|submit| Submits::Owned(submit.0)));
        Submission {
            cmd_buffers: self.cmd_buffers,
            wait_semaphores: self.wait_semaphores,
            signal_semaphores: self.signal_semaphores,
            marker: PhantomData,
        }
    }

    /// Append a new list of finished command buffers to this submission.
    ///
    /// All submits for this call must be of the same type.
    /// Submission will be automatically promoted to to the minimum required capability
    /// to hold all passed submits.
    pub fn submit_reusables<I, K>(mut self, submits: I) -> Submission<'a, B, <(C, K) as Upper>::Result>
    where
        I: IntoIterator<Item = &'a Submit<B, K, MultiShot, Primary>>,
        K: 'a,
        (C, K): Upper
    {
        self.cmd_buffers.extend(submits.into_iter().map(|submit| Submits::Ref(&submit.0)));
        Submission {
            cmd_buffers: self.cmd_buffers,
            wait_semaphores: self.wait_semaphores,
            signal_semaphores: self.signal_semaphores,
            marker: PhantomData,
        }
    }

    /// Promote a submission to a higher capability type.
    ///
    /// Submission promotion is only necessary for shoving multiple submissions
    /// of different capabilities into one submit call.
    pub fn promote<P>(self) -> Submission<'a, B, P>
    where
        P: Supports<C>
    {
        Submission {
            cmd_buffers: self.cmd_buffers,
            wait_semaphores: self.wait_semaphores,
            signal_semaphores: self.signal_semaphores,
            marker: PhantomData,
        }
    }
}
