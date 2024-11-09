use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{Amount, FlagCollection, Model, NoFlags, XRPAmount, XRPLModelResult};

use super::{
    exceptions::{XRPLAMMCreateException, XRPLTransactionException},
    CommonFields, Memo, Signer, Transaction, TransactionType,
};

pub const AMM_CREATE_MAX_FEE: u16 = 1000;

/// Create a new Automated Market Maker (AMM) instance for trading a pair of
/// assets (fungible tokens or XRP).
///
/// Creates both an AMM object and a special AccountRoot object to represent the AMM.
/// Also transfers ownership of the starting balance of both assets from the sender to
/// the created AccountRoot and issues an initial balance of liquidity provider
/// tokens (LP Tokens) from the AMM account to the sender.
///
/// Caution: When you create the AMM, you should fund it with (approximately)
/// equal-value amounts of each asset.
/// Otherwise, other users can profit at your expense by trading with
/// this AMM (performing arbitrage).
/// The currency risk that liquidity providers take on increases with the
/// volatility (potential for imbalance) of the asset pair.
/// The higher the trading fee, the more it offsets this risk,
/// so it's best to set the trading fee based on the volatility of the asset pair.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AMMCreate<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The first of the two assets to fund this AMM with. This must be a positive amount.
    pub amount: Amount<'a>,
    /// The second of the two assets to fund this AMM with. This must be a positive amount.
    pub amount2: Amount<'a>,
    /// The fee to charge for trades against this AMM instance, in units of 1/100,000;
    /// a value of 1 is equivalent to 0.001%.
    /// The maximum value is 1000, indicating a 1% fee.
    /// The minimum value is 0.
    pub trading_fee: u16,
}

impl Model for AMMCreate<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_tranding_fee_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMCreate<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> super::TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> AMMCreate<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        amount2: Amount<'a>,
        trading_fee: u16,
    ) -> AMMCreate<'a> {
        AMMCreate {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::AMMCreate,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            amount,
            amount2,
            trading_fee,
        }
    }

    fn get_tranding_fee_error(&self) -> XRPLModelResult<()> {
        if self.trading_fee > AMM_CREATE_MAX_FEE {
            Err(
                XRPLTransactionException::from(XRPLAMMCreateException::TradingFeeOutOfRange {
                    max: AMM_CREATE_MAX_FEE,
                    found: self.trading_fee,
                })
                .into(),
            )
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test_errors {
    use crate::models::IssuedCurrencyAmount;

    use super::*;

    #[test]
    fn test_trading_fee_error() {
        let amm_create = AMMCreate::new(
            Cow::Borrowed("rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY"),
            None,
            Some(XRPAmount::from("1000")),
            Some(20),
            None,
            Some(1),
            None,
            None,
            None,
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            1001,
        );

        assert!(amm_create.get_errors().is_err());
    }

    #[test]
    fn test_no_error() {
        let amm_create = AMMCreate::new(
            Cow::Borrowed("rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY"),
            None,
            Some(XRPAmount::from("1000")),
            Some(20),
            None,
            Some(1),
            None,
            None,
            None,
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            1000,
        );

        assert!(amm_create.get_errors().is_ok());
    }
}
