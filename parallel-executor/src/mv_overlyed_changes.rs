use std::collections::BTreeMap;
use std::sync::Arc;

use crossbeam::utils::CachePadded;
use dashmap::DashMap;
use sp_state_machine::{StorageKey, StorageValue};

type ShiftedTxnIndex = u32;
type Incarnation = u32;

/// A versioned value internally is represented as a BTreeMap from indices of
/// transactions that update the given access path & the corresponding entries.
struct VersionedValue<V> {
    versioned_map: BTreeMap<ShiftedTxnIndex, CachePadded<Entry<V>>>,
}

/// Every entry in shared multi-version data-structure has an "estimate" flag
/// and some content.
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

/// Maps each key (access path) to an internal versioned value representation.
pub struct VersionedData<K, V> {
    values: DashMap<K, VersionedValue<V>>,
}

/// Main multi-version data-structure used by threads to read/write during parallel
/// execution.
pub struct MVHashMap<K, V> {
    data: VersionedData<K, V>,
}

pub struct MVOverlyedChanges {
    top: MVHashMap<StorageKey, Option<StorageValue>>,
}
