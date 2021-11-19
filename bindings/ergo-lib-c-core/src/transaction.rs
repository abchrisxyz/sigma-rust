//! Ergo transaction

use ergo_lib::{chain, ergotree_ir::chain::base16_bytes::Base16EncodedBytes};

use crate::{
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Unsigned (inputs without proofs) transaction
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedTransaction(chain::transaction::unsigned::UnsignedTransaction);
pub type UnsignedTransactionPtr = *mut UnsignedTransaction;
pub type ConstUnsignedTransactionPtr = *const UnsignedTransaction;

pub unsafe fn unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
) -> Result<String, Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    Ok(Base16EncodedBytes::new(unsigned_tx.0.id().0 .0.as_ref()).into())
}

pub unsafe fn unsigned_tx_from_json(
    json: &str,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> Result<(), Error> {
    let unsigned_tx_out = mut_ptr_as_mut(unsigned_tx_out, "unsigned_tx_out")?;
    let unsigned_tx = serde_json::from_str(json)
        .map(UnsignedTransaction)
        .map_err(|_| Error::Misc("UnsignedTransaction: can't deserialize from JSON".into()))?;
    *unsigned_tx_out = Box::into_raw(Box::new(unsigned_tx));
    Ok(())
}

pub unsafe fn unsigned_tx_to_json(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
) -> Result<String, Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    serde_json::to_string(&unsigned_tx.0)
        .map_err(|_| Error::Misc("UnsignedTransaction: can't serialize into JSON".into()))
}
