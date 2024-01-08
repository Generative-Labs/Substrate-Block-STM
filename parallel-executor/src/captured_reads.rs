// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// Modifications and additional contributions by GenerativeLabs.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::bail;
use derivative::Derivative;
use sp_state_machine::{StorageKey, StorageValue};

use crate::types::{MVDataError, MVDataOutput, TxnIndex, Version};
use crate::versioned_data::VersionedData;

/// The enum variants should not be re-ordered, as it defines a relation
/// Existence < Metadata < Value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReadKind {
    Exists,
    Value,
}

/// The enum captures the state that the transaction execution extracted from
/// a read callback to block executor, in order to be validated by Block-STM.
/// The captured state is fine-grained, e.g. it distinguishes between reading
/// a full value, and other kinds of reads that may access only the metadata
/// information, or check whether data exists at a given key.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""), PartialEq(bound = ""))]
pub(crate) enum DataRead<V> {
    // Version supercedes V comparison.
    Versioned(
        Version,
        // Currently, we are conservative and check the version for equality
        // (version implies value equality, but not vice versa). TODO: when
        // comparing the instances of V is cheaper, compare those instead.
        #[derivative(PartialEq = "ignore", Debug = "ignore")] Arc<V>,
    ),
    Exists(bool),
}

// Represents the result of comparing DataReads ('self' and 'other').
#[derive(Debug)]
enum DataReadComparison {
    // Information in 'self' DataRead contains information about the kind of the
    // 'other' DataRead, and is consistent with 'other'.
    Contains,
    // Information in 'self' DataRead contains information about the kind of the
    // 'other' DataRead, but is inconsistent with 'other'.
    Inconsistent,
    // All information about the kind of 'other' is not contained in 'self' kind.
    // For example, exists does not provide enough information about metadata.
    Insufficient,
}

impl DataRead<StorageValue> {
    // Assigns highest rank to Versioned / Resolved, then Metadata, then Exists.
    // (e.g. versioned read implies metadata and existence information, and
    // metadata information implies existence information).
    fn get_kind(&self) -> ReadKind {
        use DataRead::*;
        match self {
            Versioned(_, _) => ReadKind::Value,
            Exists(_) => ReadKind::Exists,
        }
    }

    // A convenience method, since the same key can be read in different modes, producing
    // different DataRead / ReadKinds. Returns true if self has >= kind than other, i.e.
    // contains more or equal information, and is consistent with the information in other.
    fn contains(&self, other: &DataRead<StorageValue>) -> DataReadComparison {
        let self_kind = self.get_kind();
        let other_kind = other.get_kind();

        if self_kind < other_kind {
            DataReadComparison::Insufficient
        } else {
            let downcast_eq = if self_kind == other_kind {
                // Optimization to avoid unnecessary clones (e.g. during validation).
                self == other
            } else {
                self.downcast(other_kind).expect("Downcast to lower kind must succeed") == *other
            };

            if downcast_eq { DataReadComparison::Contains } else { DataReadComparison::Inconsistent }
        }
    }

    /// If the reads contains sufficient information, extract this information and generate
    /// a new DataRead of the desired kind (e.g. Metadata kind from Value).
    pub(crate) fn downcast(&self, kind: ReadKind) -> Option<DataRead<StorageValue>> {
        let self_kind = self.get_kind();
        if self_kind == kind {
            return Some(self.clone());
        }

        (self_kind > kind).then(|| match (self, &kind) {
            (DataRead::Versioned(_, v), ReadKind::Exists) => DataRead::Exists(true),
            (_, _) => unreachable!("{:?}, {:?} must be covered", self_kind, kind),
        })
    }
}

/// Serves as a "read-set" of a transaction execution, and provides APIs for capturing reads,
/// resolving new reads based on already captured reads when possible, and for validation.
///
/// The intended use is that all reads should be attempted to be resolved from CapturedReads.
/// If not possible, then after proper resolution from MVHashMap/storage, they should be
/// captured. This enforces an invariant that 'capture_read' will never be called with a
/// read that has a kind <= already captured read (for that key / tag).
#[derive(Derivative)]
#[derivative(Default(bound = "", new = "true"))]
pub(crate) struct CapturedReads<K, V> {
    data_reads: HashMap<K, DataRead<V>>,

    /// If there is a speculative failure (e.g. delta application failure, or an
    /// observed inconsistency), the transaction output is irrelevant (must be
    /// discarded and transaction re-executed). We have a global flag, as which
    /// read observed the inconsistency is irrelevant (moreover, typically,
    /// an error is returned to the VM to wrap up the ongoing execution).
    speculative_failure: bool,
    /// Set if the invarint on CapturedReads intended use is violated. Leads to an alert
    /// and sequential execution fallback.
    incorrect_use: bool,
}

#[derive(Debug)]
enum UpdateResult {
    Inserted,
    Updated,
    IncorrectUse(String),
    Inconsistency(String),
}

impl CapturedReads<StorageKey, StorageValue> {
    // Given a hashmap entry for a key, incorporate a new DataRead. This checks
    // consistency and ensures that the most comprehensive read is recorded.
    fn update_entry(entry: Entry<StorageKey, DataRead<StorageValue>>, read: DataRead<StorageValue>) -> UpdateResult {
        match entry {
            Vacant(e) => {
                e.insert(read);
                UpdateResult::Inserted
            }
            Occupied(mut e) => {
                let existing_read = e.get_mut();
                if read.get_kind() <= existing_read.get_kind() {
                    UpdateResult::IncorrectUse(format!(
                        "Incorrect use CaptureReads, read {:?}, existing {:?}",
                        read, existing_read
                    ))
                } else {
                    match read.contains(existing_read) {
                        DataReadComparison::Contains => {
                            *existing_read = read;
                            UpdateResult::Updated
                        }
                        DataReadComparison::Inconsistent => UpdateResult::Inconsistency(format!(
                            "Read {:?} must be consistent with the already stored read {:?}",
                            read, existing_read
                        )),
                        DataReadComparison::Insufficient => {
                            unreachable!("{:?} Insufficient for {:?}, but has higher kind", read, existing_read)
                        }
                    }
                }
            }
        }
    }

    // Error means there was a inconsistency in information read (must be due to the
    // speculative nature of reads).
    pub(crate) fn capture_read(&mut self, state_key: StorageKey, read: DataRead<StorageValue>) -> anyhow::Result<()> {
        let ret = Self::update_entry(self.data_reads.entry(state_key), read);

        match ret {
            UpdateResult::IncorrectUse(m) => {
                self.incorrect_use = true;
                bail!(m);
            }
            UpdateResult::Inconsistency(m) => {
                // Record speculative failure.
                self.speculative_failure = true;
                bail!(m);
            }
            UpdateResult::Updated | UpdateResult::Inserted => Ok(()),
        }
    }

    // If maybe_tag is provided, then we check the group, otherwise, normal reads.
    pub(crate) fn get_by_kind(&self, state_key: &StorageKey, kind: ReadKind) -> Option<DataRead<StorageValue>> {
        self.data_reads.get(state_key).and_then(|r| r.downcast(kind))
    }

    pub(crate) fn validate_data_reads(
        &self,
        data_map: &VersionedData<StorageKey, StorageValue>,
        idx_to_validate: TxnIndex,
    ) -> bool {
        if self.speculative_failure {
            return false;
        }

        use MVDataError::*;
        use MVDataOutput::*;
        self.data_reads.iter().all(|(k, r)| {
            match data_map.fetch_data(k, idx_to_validate) {
                Ok(Versioned(version, v)) => {
                    matches!(DataRead::Versioned(version, v).contains(r), DataReadComparison::Contains)
                }
                // Dependency implies a validation failure, and if the original read were to
                // observe an unresolved delta, it would set the aggregator base value in the
                // multi-versioned data-structure, resolve, and record the resolved value.
                Err(Dependency(_)) | Err(DeltaApplicationFailure) | Err(Uninitialized) => false,
            }
        })
    }

    pub(crate) fn mark_failure(&mut self) {
        self.speculative_failure = true;
    }
}
