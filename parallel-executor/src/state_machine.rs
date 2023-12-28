use hash_db::Hasher;
use sp_core::traits::RuntimeCode;
use sp_state_machine::backend::Backend;
use sp_state_machine::{Error, StateMachineStats};

use super::*;

/// The substrate state machine.
pub struct StateMachine<'a, B, H, Exec>
where
    H: Hasher,
    B: Backend<H>,
{
    backend: &'a B,
    exec: &'a Exec,
    method: &'a str,
    call_data: &'a [u8],
    overlay: &'a mut OverlayedChanges<H>,
    extensions: &'a mut Extensions,
    runtime_code: &'a RuntimeCode<'a>,
    stats: StateMachineStats,
    /// The hash of the block the state machine will be executed on.
    ///
    /// Used for logging.
    parent_hash: Option<H::Out>,
    context: CallContext,
}

impl<'a, B, H, Exec> Drop for StateMachine<'a, B, H, Exec>
where
    H: Hasher,
    B: Backend<H>,
{
    fn drop(&mut self) {
        self.backend.register_overlay_stats(&self.stats);
    }
}

impl<'a, B, H, Exec> StateMachine<'a, B, H, Exec>
where
    H: Hasher,
    H::Out: Ord + 'static + codec::Codec,
    Exec: CodeExecutor + Clone + 'static,
    B: Backend<H>,
{
    /// Creates new substrate state machine.
    pub fn new(
        backend: &'a B,
        overlay: &'a mut OverlayedChanges<H>,
        exec: &'a Exec,
        method: &'a str,
        call_data: &'a [u8],
        extensions: &'a mut Extensions,
        runtime_code: &'a RuntimeCode,
        context: CallContext,
    ) -> Self {
        Self {
            backend,
            exec,
            method,
            call_data,
            extensions,
            overlay,
            runtime_code,
            stats: StateMachineStats::default(),
            parent_hash: None,
            context,
        }
    }

    /// Set the given `parent_hash` as the hash of the parent block.
    ///
    /// This will be used for improved logging.
    pub fn set_parent_hash(mut self, parent_hash: H::Out) -> Self {
        self.parent_hash = Some(parent_hash);
        self
    }

    /// Execute a call using the given state backend, overlayed changes, and call executor.
    ///
    /// On an error, no prospective changes are written to the overlay.
    ///
    /// Note: changes to code will be in place if this call is made again. For running partial
    /// blocks (e.g. a transaction at a time), ensure a different method is used.
    ///
    /// Returns the SCALE encoded result of the executed function.
    pub fn execute(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        unimplemented!()
    }
}
