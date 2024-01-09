use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;

use codec::Decode;
use sc_client_api::execution_extensions::ExecutionExtensions;
use sc_client_api::{backend, CallExecutor};
use sc_executor::{RuntimeVersion, RuntimeVersionOf};
use sc_service::LocalCallExecutor;
use sp_api::ProofRecorder;
use sp_core::traits::{CallContext, CodeExecutor};
use sp_externalities::Extensions;
use sp_runtime::traits::{Block as BlockT, HashingFor};
use sp_state_machine::{OverlayedChanges, StorageKey};
use sp_trie::StorageProof;

pub mod captured_reads;
pub mod ext;
pub mod scheduler;
pub mod state_machine;
pub mod sync_wrapper;
pub mod txn_last_input_output;
pub mod types;
pub mod versioned_data;

/// ParallelExecutor enables parallel execution of batched Substrate transactions.
/// It can be used as a replacement for the substrate `LocalCallExecutor`.
pub struct ParallelLocalCallExecutor<Block: BlockT, B, E, Extrinsic> {
    // substrate local call executor.
    pub local_call_executor: LocalCallExecutor<Block, B, E>,

    executor: E,

    // Number of active concurrent tasks, corresponding to the maximum number of rayon
    // threads that may be concurrently participating in parallel execution.
    concurrency_level: usize,

    // which function should execute in parallel.
    parallel_execution_func: String,

    /// Some global reads and writes during transaction execution. For instance, `deposit_events`
    /// writes to the `Events` storage in `frame_system`. Although they physically write to the
    /// same storage, logically, they are unrelated. Therefore, we need to know the keys of
    /// these storages in advance.
    ///
    /// When multiple transactions write to this, we do not consider these transactions as
    /// conflicting at this point. Instead, conflicts are written and reassembled according to
    /// the order of transactions.
    global_access_key_set: HashSet<StorageKey>,

    _extrinsic: PhantomData<Extrinsic>,
}

impl<Block: BlockT, B, E, Extrinsic> Clone for ParallelLocalCallExecutor<Block, B, E, Extrinsic>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        ParallelLocalCallExecutor {
            local_call_executor: self.local_call_executor.clone(),
            executor: self.executor.clone(),
            concurrency_level: self.concurrency_level,
            parallel_execution_func: self.parallel_execution_func.clone(),
            global_access_key_set: self.global_access_key_set.clone(),
            _extrinsic: PhantomData,
        }
    }
}

impl<B, E, Block, Extrinsic> ParallelLocalCallExecutor<Block, B, E, Extrinsic>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
    Block: BlockT,
    Extrinsic: Decode,
{
    pub fn apply_extrinsics_parallel(
        &self,
        at_hash: Block::Hash,
        apply_single_extrinsic_method: &str,
        mut call_data: &[u8],
        changes: &RefCell<OverlayedChanges<HashingFor<Block>>>,
        recorder: &Option<ProofRecorder<Block>>,
        call_context: CallContext,
        extensions: &RefCell<Extensions>,
    ) -> Result<Vec<u8>, sp_blockchain::Error> {
        let batch_extrinsic: Vec<Extrinsic> = sp_core::Decode::decode(&mut call_data).map_err(|e| {
            sp_blockchain::Error::Execution(Box::new(format!(
                "`call_data` decode to Vec<Extrinsic> failed. error: {:#?}",
                e
            )))
        })?;

        Ok(Vec::new())
    }
}

impl<B, E, Block, Extrinsic> CallExecutor<Block> for ParallelLocalCallExecutor<Block, B, E, Extrinsic>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
    Block: BlockT,
    Extrinsic: Decode,
{
    type Error = E::Error;

    type Backend = B;

    fn execution_extensions(&self) -> &ExecutionExtensions<Block> {
        &self.local_call_executor.execution_extensions()
    }

    fn call(
        &self,
        at_hash: Block::Hash,
        method: &str,
        call_data: &[u8],
        context: CallContext,
    ) -> sp_blockchain::Result<Vec<u8>> {
        self.local_call_executor.call(at_hash, method, call_data, context)
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
        self.local_call_executor.contextual_call(
            at_hash,
            method,
            call_data,
            changes,
            recorder,
            call_context,
            extensions,
        )
    }

    fn runtime_version(&self, at_hash: Block::Hash) -> sp_blockchain::Result<RuntimeVersion> {
        CallExecutor::runtime_version(&self.local_call_executor, at_hash)
    }

    fn prove_execution(
        &self,
        at_hash: Block::Hash,
        method: &str,
        call_data: &[u8],
    ) -> sp_blockchain::Result<(Vec<u8>, StorageProof)> {
        self.local_call_executor.prove_execution(at_hash, method, call_data)
    }
}

impl<B, E, Block, Extrinsic> RuntimeVersionOf for ParallelLocalCallExecutor<Block, B, E, Extrinsic>
where
    E: RuntimeVersionOf,
    Block: BlockT,
{
    fn runtime_version(
        &self,
        ext: &mut dyn sp_externalities::Externalities,
        runtime_code: &sp_core::traits::RuntimeCode,
    ) -> Result<sp_version::RuntimeVersion, sc_executor::error::Error> {
        self.local_call_executor.runtime_version(ext, runtime_code)
    }
}

impl<Block, B, E, Extrinsic> sp_version::GetRuntimeVersionAt<Block>
    for ParallelLocalCallExecutor<Block, B, E, Extrinsic>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
    Block: BlockT,
{
    fn runtime_version(&self, at: Block::Hash) -> Result<sp_version::RuntimeVersion, String> {
        sp_version::GetRuntimeVersionAt::runtime_version(&self.local_call_executor, at)
    }
}

impl<Block, B, E, Extrinsic> sp_version::GetNativeVersion for ParallelLocalCallExecutor<Block, B, E, Extrinsic>
where
    B: backend::Backend<Block>,
    E: CodeExecutor + sp_version::GetNativeVersion + Clone + 'static,
    Block: BlockT,
{
    fn native_version(&self) -> &sp_version::NativeVersion {
        self.local_call_executor.native_version()
    }
}
