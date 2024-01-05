use std::collections::BTreeMap;
use std::sync::Arc;

use crossbeam::utils::CachePadded;
use dashmap::DashMap;
use sp_state_machine::{StorageKey, StorageValue};

use crate::types::*;    


struct VersionedValue<V> {
    versioned_map: BTreeMap<ShiftedTxnIndex, CachePadded<Entry<V>>>,
}

struct Entry<V> {
    /// Actual contents.
    cell: (Incarnation, Arc<V>),

    /// Used to mark the entry as a "write estimate".
    flag: Flag,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Flag {
    Done,
    Estimate,
}

pub struct VersionedData<K, V> {
    values: DashMap<K, VersionedValue<V>>,
}


pub struct MVHashMap<K, V> {
    data: VersionedData<K, V>,
}

impl<V> Entry<V> {
    fn new_write_from(incarnation: Incarnation, data: V) -> Entry<V> {
        Entry {
            cell: (incarnation, Arc::new(data)),
            flag: Flag::Done,
        }
    }

    fn flag(&self) -> Flag {
        self.flag
    }

    fn mark_estimate(&mut self) {
        self.flag = Flag::Estimate;
    }
}


impl VersionedValue<StorageValue> {
    fn read(&self, txn_idx: TxnIndex) -> anyhow::Result<StorageValue> { 
        unimplemented!()
    }
}

impl VersionedData<StorageKey, StorageValue> {
    pub(crate) fn new() -> Self {
        Self {
            values: DashMap::new(),
        }
    }

    /// Mark an entry from transaction 'txn_idx' at access path 'key' as an estimated write
    /// (for future incarnation). Will panic if the entry is not in the data-structure.
    pub fn mark_estimate(&self, key: &StorageKey, txn_idx: TxnIndex) {
        unimplemented!()
    }

    /// Delete an entry from transaction 'txn_idx' at access path 'key'. Will panic
    /// if the corresponding entry does not exist.
    pub fn delete(&self, key: &StorageKey, txn_idx: TxnIndex) {
        unimplemented!()
    }

    pub fn fetch_data(
        &self,
        key: &StorageKey,
        txn_idx: TxnIndex,
    ) -> anyhow::Result<StorageValue> {
        unimplemented!()
    }

    pub fn provide_base_value(&self, key: StorageKey, data: StorageValue) {
        unimplemented!()
    }

    /// Versioned write of data at a given key (and version).
    pub fn write(&self, key: StorageKey, txn_idx: TxnIndex, incarnation: Incarnation, data: StorageValue) {
        unimplemented!()
    }
}