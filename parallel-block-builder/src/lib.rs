use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{ApplyExtrinsicFailed, Error, HeaderBackend};
use sp_runtime::traits::{Block as BlockT, Extrinsic, Hash, Digest,legacy, Header as HeaderT};
use sp_state_machine::Backend;

// use std::vec::Vec;


struct ParallelBlockBuilder<'a, Block, C> {
    block_builder: BlockBuilder<'a, Block, C>,
}

impl<'a, Block, C> ParallelBlockBuilder<'a, Block, C>
    where
        Block: BlockT,
        C: HeaderBackend<Block> + CallApiAt<Block>,
{
    pub fn new(call_api_at: &'a C, parent_block: Block::Hash) -> Result<Self, Error> {
        let block_builder = BlockBuilder::new(call_api_at, parent_block, 0, false, Digest::default())?;
        Ok(Self { block_builder })
    }

    pub fn batch_push(&mut self, xts: Vec<Block::Extrinsic>) -> Result<(), Error> {
        let parent_hash = self.block_builder.parent_hash;
        let extrinsics = &mut self.block_builder.extrinsics;
        let version = self.block_builder.version;

        self.block_builder.api.execute_in_transaction(|api| {
            let res = if version < 6 {
                #[allow(deprecated)]
                api.batch_apply_extrinsic_before_version_6(parent_hash, xts.clone())
                    .map(legacy::byte_sized_error::convert_to_latest)
            } else {
                api.batch_apply_extrinsic(parent_hash, xts.clone())
            };

            match res {
                Ok(Ok(_)) => {
                    extrinsics.extend(xts[1..]);
                    TransactionOutcome::Commit(Ok(()))
                },
                Ok(Err(tx_validity)) => TransactionOutcome::Rollback(Err(
                    ApplyExtrinsicFailed::Validity(tx_validity).into(),
                )),
                Err(e) => TransactionOutcome::Rollback(Err(Error::from(e))),
            }
        })
    }
}
