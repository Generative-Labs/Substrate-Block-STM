use std::cell::RefCell;
use std::marker::PhantomData;

use hash_db::Hasher;
use sp_core::storage::ChildInfo;
use sp_externalities::{ExtensionStore, Externalities};
use sp_state_machine::backend::Backend;
use sp_state_machine::{StorageKey, StorageValue};

use crate::captured_reads::CapturedReads;
use crate::txn_last_input_output::TxnLastInputOutput;
use crate::types::TxnIndex;
use crate::versioned_data::VersionedData;

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

    captured_reads: RefCell<CapturedReads<StorageKey, StorageValue>>,
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
