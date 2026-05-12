use xrpl::models::{transactions::payment::Payment, Amount, XRPAmount};

/// Build a minimal XRP-only Payment for tests. All optional fields are left
/// None; callers typically autofill (or use `test_transaction` / submit_and_wait)
/// to populate sequence, fee, and last_ledger_sequence.
pub fn xrp_payment<'a>(from: String, to: String, drops: impl Into<XRPAmount<'a>>) -> Payment<'a> {
    Payment::new(
        from.into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Amount::XRPAmount(drops.into()),
        to.into(),
        None,
        None,
        None,
        None,
        None,
    )
}
