pub type TxnIndex = u32;
pub type Incarnation = u32;
pub type ShiftedTxnIndex = u32;

/// Custom error type representing storage version. Result<Index, StorageVersion>
/// then represents either index of some type (i.e. TxnIndex, Version), or a
/// version corresponding to the storage (pre-block) state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageVersion;

// TODO: Find better representations for this, a similar one for TxnIndex.
pub type Version = Result<(TxnIndex, Incarnation), StorageVersion>;
