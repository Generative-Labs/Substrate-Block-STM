// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// Modifications and additional contributions by GenerativeLabs.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::Iterator;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use arc_swap::ArcSwapOption;
use crossbeam::utils::CachePadded;
use sp_state_machine::{StorageKey, StorageValue};

use crate::captured_reads::CapturedReads;
use crate::types::{ExecutionStatus, TxnIndex};

type TxnInput = CapturedReads<StorageKey, StorageValue>;

/// The output of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOutput {
    /// The list of writes this transaction intends to do.
    write_set: BTreeMap<StorageKey, StorageValue>,
}

#[derive(Debug)]
pub(crate) struct TxnOutput {
    output_status: ExecutionStatus<TransactionOutput>,
}

impl TxnOutput {
    pub fn from_output_status(output_status: ExecutionStatus<TransactionOutput>) -> Self {
        Self { output_status }
    }

    pub fn output_status(&self) -> &ExecutionStatus<TransactionOutput> {
        &self.output_status
    }
}

pub struct TxnLastInputOutput {
    inputs: Vec<CachePadded<ArcSwapOption<TxnInput>>>, // txn_idx -> input.

    outputs: Vec<CachePadded<ArcSwapOption<TxnOutput>>>, // txn_idx -> output.

    module_read_write_intersection: AtomicBool,
}

impl TxnLastInputOutput {
    pub fn new(num_txns: TxnIndex) -> Self {
        Self {
            inputs: (0..num_txns).map(|_| CachePadded::new(ArcSwapOption::empty())).collect(),
            outputs: (0..num_txns).map(|_| CachePadded::new(ArcSwapOption::empty())).collect(),
            module_read_write_intersection: AtomicBool::new(false),
        }
    }

    /// Returns false on an error - if a module path that was read was previously written to, and
    /// vice versa. Since parallel executor is instantiated per block, any module that is in the
    /// Move-VM loader cache must previously be read and would be recorded in the 'module_reads'
    /// set. Any module that is written (published or re-published) goes through transaction
    /// output write-set and gets recorded in the 'module_writes' set. If these sets have an
    /// intersection, it is currently possible that Move-VM loader cache loads a module and
    /// incorrectly uses it for another transaction (e.g. a smaller transaction, or if the
    /// speculative execution of the publishing transaction later aborts). The intersection is
    /// guaranteed to be found because we first record the paths then check the other set (flags
    /// principle), and in this case we return an error that ensures a fallback to a correct
    /// sequential execution. When the sets do not have an intersection, it is impossible for
    /// the race to occur as any module in the loader cache may not be published by a
    /// transaction in the ongoing block.
    pub(crate) fn record(
        &self,
        txn_idx: TxnIndex,
        input: CapturedReads<StorageKey, StorageValue>,
        output: ExecutionStatus<TransactionOutput>,
    ) -> bool {
        self.inputs[txn_idx as usize].store(Some(Arc::new(input)));
        self.outputs[txn_idx as usize].store(Some(Arc::new(TxnOutput::from_output_status(output))));

        true
    }

    pub(crate) fn read_set(&self, txn_idx: TxnIndex) -> Option<Arc<CapturedReads<StorageKey, StorageValue>>> {
        self.inputs[txn_idx as usize].load_full()
    }

    /// Does a transaction at txn_idx have SkipRest or Abort status.
    pub(crate) fn block_skips_rest_at_idx(&self, txn_idx: TxnIndex) -> bool {
        matches!(
            &self.outputs[txn_idx as usize]
                .load_full()
                .expect("[BlockSTM]: Execution output must be recorded after execution")
                .output_status,
            ExecutionStatus::SkipRest(_)
        )
    }

    pub(crate) fn update_to_skip_rest(&self, txn_idx: TxnIndex) {
        if let ExecutionStatus::Success(output) = self.take_output(txn_idx) {
            self.outputs[txn_idx as usize]
                .store(Some(Arc::new(TxnOutput { output_status: ExecutionStatus::SkipRest(output) })));
        } else {
            unreachable!();
        }
    }

    pub(crate) fn txn_output(&self, txn_idx: TxnIndex) -> Option<Arc<TxnOutput>> {
        self.outputs[txn_idx as usize].load_full()
    }

    // Extracts a set of paths (keys) written or updated during execution from transaction
    // output, .1 for each item is false for non-module paths and true for module paths.
    pub(crate) fn modified_keys(&self, txn_idx: TxnIndex) -> Option<impl Iterator<Item = StorageKey>> {
        self.outputs[txn_idx as usize].load_full().and_then(|txn_output| match &txn_output.output_status {
            ExecutionStatus::Success(t) | ExecutionStatus::SkipRest(t) => Some(t.write_set.clone().into_keys()),
            ExecutionStatus::Abort => None,
        })
    }

    // Must be executed after parallel execution is done, grabs outputs. Will panic if
    // other outstanding references to the recorded outputs exist.
    pub(crate) fn take_output(&self, txn_idx: TxnIndex) -> ExecutionStatus<TransactionOutput> {
        let owning_ptr =
            self.outputs[txn_idx as usize].swap(None).expect("[BlockSTM]: Output must be recorded after execution");

        Arc::try_unwrap(owning_ptr)
            .map(|output| output.output_status)
            .expect("[BlockSTM]: Output should be uniquely owned after execution")
    }
}
