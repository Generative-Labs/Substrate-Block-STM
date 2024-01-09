use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::AtomicU32;

use hash_db::Hasher;
use sp_core::storage::ChildInfo;
use sp_externalities::{ExtensionStore, Externalities};
use sp_state_machine::backend::Backend;
use sp_state_machine::{StorageKey, StorageValue};

use crate::captured_reads::{CapturedReads, ReadKind, DataRead};
use crate::txn_last_input_output::TxnLastInputOutput;
use crate::types::{TxnIndex, MVDataError, MVDataOutput};
use crate::versioned_data::VersionedData;
use crate::scheduler::{DependencyResult, DependencyStatus, Scheduler};

/// A struct which describes the result of the read from the proxy. The client
/// can interpret these types to further resolve the reads.
#[derive(Debug)]
pub(crate) enum ReadResult {
    Value(Option<StorageValue>),
    Exists(bool),
    Uninitialized,
    // Must halt the execution of the calling transaction. This might be because
    // there was an inconsistency in observed speculative state, or dependency
    // waiting indicated that the parallel execution had been halted. The String
    // parameter provides more context (error description / message).
    HaltSpeculativeExecution(String),
}

impl ReadResult {
    fn from_data_read(data: DataRead<StorageValue>) -> Self {
        match data {
            DataRead::Versioned(_, v) => ReadResult::Value(Some(v.to_vec())),
            DataRead::Exists(exists) => ReadResult::Exists(exists),
        }
    }
}

/// A struct that represents a single block execution worker thread's view into the state,
pub struct Ext<'a, H, B>
where
    H: Hasher,
    B: 'a + Backend<H>,
{
    latest_state: ParallelState<'a>,

    last_input_output: &'a TxnLastInputOutput,

    /// The storage backend to read from.
    backend: &'a B,

    /// Pseudo-unique id used for tracing.
    pub id: u16,
    /// Extensions registered with this instance.
    #[cfg(feature = "std")]
    extensions: Option<OverlayedExtensions<'a>>,

    txn_idx: TxnIndex,

    _phantom: PhantomData<H>,
}

pub struct ParallelState<'a> {
    versioned_top: &'a VersionedData<StorageKey, StorageValue>,
    
    scheduler: &'a Scheduler,

    _counter: &'a AtomicU32,
    
    captured_reads: RefCell<CapturedReads<StorageKey, StorageValue>>,
}

impl<'a, H, B>  Ext<'a, H, B>
where
    H: Hasher,
    H::Out: Ord + 'static + codec::Codec,
    B: Backend<H>,
{
    pub(crate) fn take_reads(&self)->CapturedReads<StorageKey, StorageValue>{
        self.latest_state.captured_reads.take()
    }
}

impl<'a, H, B> Externalities for Ext<'a, H, B>
where
    H: Hasher,
    H::Out: Ord + 'static + codec::Codec,
    B: Backend<H>,
{
    fn set_offchain_storage(&mut self, key: &[u8], value: Option<&[u8]>) {
        todo!()
    }

    fn storage(&self, key: &[u8]) -> Option<Vec<u8>> {
        //self.latest_state.read_top_data_by_kind(key, self.txn_idx, ReadKind::Value);
        todo!()
    }

    fn storage_hash(&self, key: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    fn child_storage_hash(&self, child_info: &sc_client_api::ChildInfo, key: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    fn child_storage(&self, child_info: &sc_client_api::ChildInfo, key: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    fn next_storage_key(&self, key: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    fn next_child_storage_key(&self, child_info: &sc_client_api::ChildInfo, key: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    fn kill_child_storage(
        &mut self,
        child_info: &sc_client_api::ChildInfo,
        maybe_limit: Option<u32>,
        maybe_cursor: Option<&[u8]>,
    ) -> sp_externalities::MultiRemovalResults {
        todo!()
    }

    fn clear_prefix(
        &mut self,
        prefix: &[u8],
        maybe_limit: Option<u32>,
        maybe_cursor: Option<&[u8]>,
    ) -> sp_externalities::MultiRemovalResults {
        todo!()
    }

    fn clear_child_prefix(
        &mut self,
        child_info: &sc_client_api::ChildInfo,
        prefix: &[u8],
        maybe_limit: Option<u32>,
        maybe_cursor: Option<&[u8]>,
    ) -> sp_externalities::MultiRemovalResults {
        todo!()
    }

    fn place_storage(&mut self, key: Vec<u8>, value: Option<Vec<u8>>) {
        todo!()
    }

    fn place_child_storage(&mut self, child_info: &sc_client_api::ChildInfo, key: Vec<u8>, value: Option<Vec<u8>>) {
        todo!()
    }

    fn storage_root(&mut self, state_version: sp_runtime::StateVersion) -> Vec<u8> {
        todo!()
    }

    fn child_storage_root(
        &mut self,
        child_info: &sc_client_api::ChildInfo,
        state_version: sp_runtime::StateVersion,
    ) -> Vec<u8> {
        todo!()
    }

    fn storage_append(&mut self, key: Vec<u8>, value: Vec<u8>) {
        todo!()
    }

    fn storage_start_transaction(&mut self) {
        todo!()
    }

    fn storage_rollback_transaction(&mut self) -> Result<(), ()> {
        todo!()
    }

    fn storage_commit_transaction(&mut self) -> Result<(), ()> {
        todo!()
    }

    fn wipe(&mut self) {
        todo!()
    }

    fn commit(&mut self) {
        todo!()
    }

    fn read_write_count(&self) -> (u32, u32, u32, u32) {
        todo!()
    }

    fn reset_read_write_count(&mut self) {
        todo!()
    }

    fn get_whitelist(&self) -> Vec<sp_core::storage::TrackedStorageKey> {
        todo!()
    }

    fn set_whitelist(&mut self, new: Vec<sp_core::storage::TrackedStorageKey>) {
        todo!()
    }

    fn get_read_and_written_keys(&self) -> Vec<(Vec<u8>, u32, u32, bool)> {
        todo!()
    }
}

impl<'a, H, B> ExtensionStore for Ext<'a, H, B>
where
    H: Hasher,
    B: 'a + Backend<H>,
{
    fn extension_by_type_id(&mut self, type_id: std::any::TypeId) -> Option<&mut dyn std::any::Any> {
        todo!()
    }

    fn register_extension_with_type_id(
        &mut self,
        type_id: std::any::TypeId,
        extension: Box<dyn sp_externalities::Extension>,
    ) -> Result<(), sp_externalities::Error> {
        todo!()
    }

    fn deregister_extension_by_type_id(&mut self, type_id: std::any::TypeId) -> Result<(), sp_externalities::Error> {
        todo!()
    }
}

impl<'a, H, B> Ext<'a, H, B>
where
    H: Hasher,
    H::Out: Ord + 'static + codec::Codec,
    B: Backend<H>,
{
    fn limit_remove_from_backend(
        &mut self,
        child_info: Option<&ChildInfo>,
        prefix: Option<&[u8]>,
        maybe_limit: Option<u32>,
        start_at: Option<&[u8]>,
    ) -> (Option<Vec<u8>>, u32, u32) {
        todo!()
    }
}

impl<'a> ParallelState<'a>{
    pub(crate) fn new(
        shared_top: &'a VersionedData<StorageKey, StorageValue>,
        shared_scheduler: &'a Scheduler,
        shared_counter: &'a AtomicU32,
    ) -> Self {
        Self {
            versioned_top: shared_top,
            scheduler: shared_scheduler,
            _counter: shared_counter,
            captured_reads: RefCell::new(CapturedReads::new()),
        }
    }

    fn wait_for_dependency(&self, txn_idx: TxnIndex, dep_idx: TxnIndex) -> bool {
        match self.scheduler.wait_for_dependency(txn_idx, dep_idx) {
            DependencyResult::Dependency(dep_condition) => {
                // let _timer = counters::DEPENDENCY_WAIT_SECONDS.start_timer();
                let (lock, cvar) = &*dep_condition;
                let mut dep_resolved = lock.lock();
                while let DependencyStatus::Unresolved = *dep_resolved {
                    dep_resolved = cvar.wait(dep_resolved).unwrap();
                }
                // dep resolved status is either resolved or execution halted.
                matches!(*dep_resolved, DependencyStatus::Resolved)
            },
            DependencyResult::ExecutionHalted => false,
            DependencyResult::Resolved => true,
        }
    }

    fn read_top_data_by_kind(
        &self,
        key: &StorageKey,
        txn_idx: TxnIndex,
        target_kind: ReadKind,
    ) -> ReadResult {
        use MVDataError::*;
        use MVDataOutput::*;

        if let Some(data) = self
            .captured_reads
            .borrow()
            .get_by_kind(key, target_kind.clone())
        {
            return ReadResult::from_data_read(data);
        }

        loop {
            match self.versioned_top.fetch_data(key, txn_idx) {
                Ok(Versioned(version, v)) => {
                    let data_read = DataRead::Versioned(version, v.clone())
                        .downcast(target_kind)
                        .expect("Downcast from Versioned must succeed");

                    if self
                        .captured_reads
                        .borrow_mut()
                        .capture_read(key.clone(), data_read.clone())
                        .is_err()
                    {
                        // Inconsistency in recorded reads.
                        return ReadResult::HaltSpeculativeExecution(
                            "Inconsistency in reads (must be due to speculation)".to_string(),
                        );
                    }

                    return ReadResult::from_data_read(data_read);
                },
                Err(Uninitialized) => {
                    // The underlying assumption here for not recording anything about the read is
                    // that the caller is expected to initialize the contents and serve the reads
                    // solely via the 'fetch_read' interface. Thus, the later, successful read,
                    // will make the needed recordings.
                    return ReadResult::Uninitialized;
                },
                Err(Dependency(dep_idx)) => {
                    if !self.wait_for_dependency(txn_idx, dep_idx) {
                        return ReadResult::HaltSpeculativeExecution(
                            "Interrupted as block execution was halted".to_string(),
                        );
                    }
                },
                Err(DeltaApplicationFailure) => {
                    // AggregatorV1 may have delta application failure due to speculation.
                    self.captured_reads.borrow_mut().mark_failure();
                    return ReadResult::HaltSpeculativeExecution(
                        "Delta application failure (must be speculative)".to_string(),
                    );
                },
            };
        }
    }
}