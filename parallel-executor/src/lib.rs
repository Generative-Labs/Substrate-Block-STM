use std::cell::RefCell;

use sc_client_api::execution_extensions::ExecutionExtensions;
use sc_client_api::{backend, CallExecutor};
use sc_executor::{RuntimeVersion, RuntimeVersionOf};
use sc_service::LocalCallExecutor;
use sp_api::ProofRecorder;
use sp_core::traits::{CallContext, CodeExecutor};
use sp_externalities::Extensions;
use sp_runtime::traits::{Block as BlockT, HashingFor};
use sp_state_machine::OverlayedChanges;
use sp_trie::StorageProof;

/// ParallelExecutor enables parallel execution of batched Substrate transactions.
/// It can be used as a replacement for the substrate `LocalCallExecutor`.
pub struct ParallelLocalCallExecutor<Block: BlockT, B, E> {
    pub executor: LocalCallExecutor<Block, B, E>,

    // Number of active concurrent tasks, corresponding to the maximum number of rayon
    // threads that may be concurrently participating in parallel execution.
    concurrency_level: usize,
}

impl<Block: BlockT, B, E> Clone for ParallelLocalCallExecutor<Block, B, E>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        ParallelLocalCallExecutor { executor: self.executor.clone(), concurrency_level: self.concurrency_level }
    }
}

impl<B, E, Block> CallExecutor<Block> for ParallelLocalCallExecutor<Block, B, E>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
    Block: BlockT,
{
    type Error = E::Error;

    type Backend = B;

    fn execution_extensions(&self) -> &ExecutionExtensions<Block> {
        &self.executor.execution_extensions()
    }

    fn call(
        &self,
        at_hash: Block::Hash,
        method: &str,
        call_data: &[u8],
        context: CallContext,
    ) -> sp_blockchain::Result<Vec<u8>> {
        self.executor.call(at_hash, method, call_data, context)
    }

    fn contextual_call(
        &self,
        at_hash: Block::Hash,
        method: &str,
        call_data: &[u8],
        changes: &RefCell<OverlayedChanges<HashingFor<Block>>>,
        recorder: &Option<ProofRecorder<Block>>,
        call_context: CallContext,
        extensions: &RefCell<Extensions>,
    ) -> Result<Vec<u8>, sp_blockchain::Error> {
        self.executor.contextual_call(at_hash, method, call_data, changes, recorder, call_context, extensions)
    }

    fn runtime_version(&self, at_hash: Block::Hash) -> sp_blockchain::Result<RuntimeVersion> {
        CallExecutor::runtime_version(&self.executor, at_hash)
    }

    fn prove_execution(
        &self,
        at_hash: Block::Hash,
        method: &str,
        call_data: &[u8],
    ) -> sp_blockchain::Result<(Vec<u8>, StorageProof)> {
        self.executor.prove_execution(at_hash, method, call_data)
    }
}

impl<B, E, Block> RuntimeVersionOf for ParallelLocalCallExecutor<Block, B, E>
where
    E: RuntimeVersionOf,
    Block: BlockT,
{
    fn runtime_version(
        &self,
        ext: &mut dyn sp_externalities::Externalities,
        runtime_code: &sp_core::traits::RuntimeCode,
    ) -> Result<sp_version::RuntimeVersion, sc_executor::error::Error> {
        self.executor.runtime_version(ext, runtime_code)
    }
}

impl<Block, B, E> sp_version::GetRuntimeVersionAt<Block> for ParallelLocalCallExecutor<Block, B, E>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
    Block: BlockT,
{
    fn runtime_version(&self, at: Block::Hash) -> Result<sp_version::RuntimeVersion, String> {
        sp_version::GetRuntimeVersionAt::runtime_version(&self.executor, at)
    }
}

impl<Block, B, E> sp_version::GetNativeVersion for ParallelLocalCallExecutor<Block, B, E>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + sp_version::GetNativeVersion + Clone + 'static,
    Block: BlockT,
{
    fn native_version(&self) -> &sp_version::NativeVersion {
        self.executor.native_version()
    }
}
