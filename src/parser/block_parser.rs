// Bitcoin protocol (BP) daemon node
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use bitcoin::{Block, Transaction, TxIn, TxOut, Txid};
use bp::short_id::{BlockChecksum, Descriptor, Dimension};
use std::collections::{hash_map::Entry, HashMap};

use super::{error::Error, *};
use crate::db::models as index_models;

#[derive(Debug, Display)]
#[display_from(Debug)]
pub(super) struct BlockParser<'a> {
    coinbase_amount: Option<u64>,
    descriptor: Descriptor,
    // The difference between `data.utxo` and `base_utxo`: `base_utxo` is immutable "base" layer
    // for all UTXO's from previous blocks, while `data.utxo` is local/ephemeral mutable collector
    // for new UTXOs from the currently parsed block(s)
    data: &'a mut ParseData,
    base_utxo: &'a UtxoMap,
}

impl<'a> BlockParser<'a> {
    pub(super) fn parse(
        block: Block,
        data: &'a mut ParseData,
        utxo: &'a UtxoMap,
    ) -> Result<Self, Error> {
        let block_checksum = BlockChecksum::from(block.block_hash());
        let mut parser = Self {
            coinbase_amount: None,
            descriptor: Descriptor::OnchainBlock {
                block_height: data.state.processed_height as u32,
                block_checksum,
            },
            data,
            base_utxo: utxo,
        };
        parser.parse_block(&block)?;
        Ok(parser)
    }
}

impl BlockParser<'_> {
    fn parse_block(&mut self, block: &Block) -> Result<(), Error> {
        debug!("Processing block {}", block.block_hash());

        self.descriptor = Descriptor::OnchainBlock {
            block_height: self.data.state.processed_height as u32,
            block_checksum: BlockChecksum::from(block.block_hash()),
        };

        block
            .txdata
            .iter()
            .enumerate()
            .try_for_each(|(index, tx)| self.parse_tx(index, tx))?;

        self.data.blocks.push(
            index_models::Block::compose(block, self.descriptor)
                .map_err(|_| Error::CorruptedShortId)?,
        );

        self.data.state.processed_height += 1;
        // TODO: Update the rest of the state

        Ok(())
    }

    fn parse_tx(&mut self, index: usize, tx: &Transaction) -> Result<(), Error> {
        self.coinbase_amount = if tx.is_coin_base() {
            Some(tx.output[0].value)
        } else {
            None
        };

        self.descriptor = self
            .descriptor
            .upgraded(index as u16, None)
            .expect("Descriptor upgrade for an onchain block does not fail");

        let txid = tx.txid();
        tx.output
            .iter()
            .enumerate()
            .try_for_each(|(index, txout)| self.parse_txout(index, txid, txout))?;
        tx.input
            .iter()
            .enumerate()
            .try_for_each(|(index, txin)| self.parse_txin(index, txin))?;

        self.descriptor = self
            .descriptor
            .downgraded()
            .expect("Descriptor downgrade from an onchain transaction can't fail");

        self.data.txs.push(
            index_models::Tx::compose(tx, self.descriptor).map_err(|_| Error::CorruptedShortId)?,
        );

        // TODO: Update state stats

        Ok(())
    }

    fn parse_txin(&mut self, index: usize, txin: &TxIn) -> Result<(), Error> {
        let block_descriptor = self
            .descriptor
            .downgraded()
            .expect("Transaction to block descriptor downgrade can't fail");

        let txo_descriptor = if let Some(coinbase_amount) = self.coinbase_amount {
            self.data.txouts.push(index_models::Txout {
                id: block_descriptor.try_into_u64()
                    .expect("Block descriptor is generated from other already used descriptor, so can't fail")
                    as i64,
                amount: coinbase_amount as i64,
                script: vec![]
            });
            block_descriptor
        } else {
            // TODO: Update state stats
            self.base_utxo
                .get_descriptor(&txin.previous_output)
                .map(|d| {
                    self.data.spent.push(txin.previous_output);
                    d.clone()
                })
                .or_else(|| {
                    self.data
                        .state
                        .utxo
                        .extract_descriptor(&txin.previous_output)
                })
                .ok_or(Error::BlockValidationIncosistency)?
                .clone()
        };

        let descriptor = self
            .descriptor
            .upgraded(index as u16, Some(Dimension::Input))
            .expect("Descriptor upgrade for an onchain transaction does not fail");

        self.data.txins.push(
            index_models::Txin::compose(txin, descriptor, txo_descriptor)
                .map_err(|_| Error::CorruptedShortId)?,
        );

        // TODO: Update state stats

        Ok(())
    }

    fn parse_txout(&mut self, index: usize, txid: Txid, txout: &TxOut) -> Result<(), Error> {
        let descriptor = self
            .descriptor
            .upgraded(index as u16, Some(Dimension::Output))
            .expect("Descriptor upgrade for an onchain transaction does not fail");

        let txoset = match self.data.state.utxo.entry(txid) {
            Entry::Vacant(entry) => entry.insert(HashMap::new()),
            Entry::Occupied(entry) => entry.into_mut(),
        };
        txoset.insert(index as u16, self.descriptor);

        self.data.txouts.push(
            index_models::Txout::compose(txout, descriptor).map_err(|_| Error::CorruptedShortId)?,
        );

        // TODO: Update state stats

        Ok(())
    }
}
