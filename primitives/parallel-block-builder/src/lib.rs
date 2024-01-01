//! The block builder runtime api.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::{traits::Block as BlockT, ApplyExtrinsicResult};

sp_api::decl_runtime_apis! {
	/// The `BlockBuilder` api trait that provides the required functionality for building a block.
	#[api_version(6)]
	pub trait BlockBuilder {
		/// Apply the given extrinsic.
		///
		/// Returns an inclusion outcome which specifies if this extrinsic is included in
		/// this block or not.
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult;

		fn batch_apply_extrinsic(extrinsic: Vec<<Block as BlockT>::Extrinsic>) -> ApplyExtrinsicResult;

		#[changed_in(6)]
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> sp_runtime::legacy::byte_sized_error::ApplyExtrinsicResult;

		#[changed_in(6)]
		fn batch_apply_extrinsic(
			extrinsic: Vec<<Block as BlockT>::Extrinsic>,
		) -> sp_runtime::legacy::byte_sized_error::ApplyExtrinsicResult;

		/// Finish the current block.
		#[renamed("finalise_block", 3)]
		fn finalize_block() -> <Block as BlockT>::Header;

		/// Generate inherent extrinsics. The inherent data will vary from chain to chain.
		fn inherent_extrinsics(
			inherent: InherentData,
		) -> sp_std::vec::Vec<<Block as BlockT>::Extrinsic>;

		/// Check that the inherents are valid. The inherent data will vary from chain to chain.
		fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult;
	}
}
