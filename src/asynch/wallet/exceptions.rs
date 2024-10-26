use thiserror_no_std::Error;

#[derive(Error, PartialEq, Debug)]
pub enum XRPLFaucetException {
    #[error(
        "Cannot fund an account on an issuing chain. Accounts must be created via the bridge."
    )]
    CannotFundSidechainAccount,
    #[error("Cannot derive a faucet URL from the client host.")]
    CannotDeriveFaucetUrl,
    #[error("Funding request timed out.")]
    FundingTimeout,
}
