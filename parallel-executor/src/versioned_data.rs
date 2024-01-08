// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

// Modifications and additional contributions by GenerativeLabs.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::btree_map::BTreeMap;
use std::sync::Arc;

use claims::assert_some;
use crossbeam::utils::CachePadded;
use dashmap::DashMap;
use sp_state_machine::{StorageKey, StorageValue};

use crate::types::{Flag, Incarnation, MVDataError, MVDataOutput, ShiftedTxnIndex, TxnIndex};

/// Every entry in shared multi-version data-structure has an "estimate" flag
/// and some content.
struct Entry<V> {
    /// Actual contents.
    cell: (Incarnation, Arc<V>),

    /// Used to mark the entry as a "write estimate".
    flag: Flag,
}

/// A versioned value internally is represented as a BTreeMap from indices of
/// transactions that update the given access path & the corresponding entries.
struct VersionedValue<V> {
    versioned_map: BTreeMap<ShiftedTxnIndex, CachePadded<Entry<V>>>,
}

/// Maps each key (access path) to an internal versioned value representation.
pub struct VersionedData<K, V> {
    values: DashMap<K, VersionedValue<V>>,
}

type VersionedStorageValue = VersionedValue<Option<StorageValue>>;
type VersionedStorageData = VersionedData<StorageKey, Option<StorageValue>>;

impl<V> Entry<V> {
    fn new_write_from(incarnation: Incarnation, data: V) -> Entry<V> {
        Entry { cell: (incarnation, Arc::new(data)), flag: Flag::Done }
    }

    fn flag(&self) -> Flag {
        self.flag
    }

    fn mark_estimate(&mut self) {
        self.flag = Flag::Estimate;
    }
}

impl<V> Default for VersionedValue<V> {
    fn default() -> Self {
        Self { versioned_map: BTreeMap::new() }
    }
}

impl<V> VersionedValue<V> {
    fn read(&self, txn_idx: TxnIndex) -> anyhow::Result<MVDataOutput<V>, MVDataError> {
        use MVDataError::*;
        use MVDataOutput::*;

        let mut iter = self.versioned_map.range(ShiftedTxnIndex::zero()..ShiftedTxnIndex::new(txn_idx));

        while let Some((idx, entry)) = iter.next_back() {
            if entry.flag() == Flag::Estimate {
                // Found a dependency.
                return Err(Dependency(idx.idx().expect("May not depend on storage version")));
            }

            return Ok(Versioned(idx.idx().map(|idx| (idx, entry.cell.0)), entry.cell.1.clone()));
        }

        return Err(Uninitialized);
    }
}

impl VersionedData<StorageKey, StorageValue> {
    pub(crate) fn new() -> Self {
        Self { values: DashMap::new() }
    }

    /// Mark an entry from transaction 'txn_idx' at access path 'key' as an estimated write
    /// (for future incarnation). Will panic if the entry is not in the data-structure.
    pub fn mark_estimate(&self, key: &StorageKey, txn_idx: TxnIndex) {
        let mut v = self.values.get_mut(key).expect("Path must exist");
        v.versioned_map
            .get_mut(&ShiftedTxnIndex::new(txn_idx))
            .expect("Entry by the txn must exist to mark estimate")
            .mark_estimate();
    }

    /// Delete an entry from transaction 'txn_idx' at access path 'key'. Will panic
    /// if the corresponding entry does not exist.
    pub fn delete(&self, key: &StorageKey, txn_idx: TxnIndex) {
        // TODO: investigate logical deletion.
        let mut v = self.values.get_mut(key).expect("Path must exist");
        assert_some!(
            v.versioned_map.remove(&ShiftedTxnIndex::new(txn_idx)),
            "Entry for key / idx must exist to be deleted"
        );
    }

    pub fn fetch_data(
        &self,
        key: &StorageKey,
        txn_idx: TxnIndex,
    ) -> anyhow::Result<MVDataOutput<StorageValue>, MVDataError> {
        self.values.get(key).map(|v| v.read(txn_idx)).unwrap_or(Err(MVDataError::Uninitialized))
    }

    pub fn provide_base_value(&self, key: StorageKey, data: StorageValue) {
        let mut v = self.values.entry(key).or_default();
        let bytes_len = data.len();
        // For base value, incarnation is irrelevant, set to 0.
        let prev_entry =
            v.versioned_map.insert(ShiftedTxnIndex::zero(), CachePadded::new(Entry::new_write_from(0, data)));

        assert!(prev_entry.map_or(true, |entry| -> bool { entry.cell.0 == 0 && entry.cell.1.len() == bytes_len }));
    }

    /// Versioned write of data at a given key (and version).
    pub fn write(&self, key: StorageKey, txn_idx: TxnIndex, incarnation: Incarnation, data: StorageValue) {
        let mut v = self.values.entry(key).or_default();
        let prev_entry = v
            .versioned_map
            .insert(ShiftedTxnIndex::new(txn_idx), CachePadded::new(Entry::new_write_from(incarnation, data)));

        // Assert that the previous entry for txn_idx, if present, had lower incarnation.
        assert!(prev_entry.map_or(true, |entry| -> bool { entry.cell.0 < incarnation }));
    }
}
