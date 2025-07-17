use xrpl::asynch::transaction::sign;
use xrpl::models::transactions::{AccountSet, CommonFields, TransactionType};
use xrpl::wallet::Wallet;

fn main() {
    // Create a new wallet we can use to sign the transaction
    let wallet = Wallet::create(None).unwrap();

    // Define the transaction using the builder pattern
    let mut account_set = AccountSet {
        common_fields: CommonFields {
            account: wallet.classic_address.clone().into(),
            transaction_type: TransactionType::AccountSet,
            ..Default::default()
        },
        ..Default::default()
    }
    .with_domain("6578616d706c652e636f6d".into()) // example.com
    .with_fee("12".into());

    println!("AccountSet transaction before signing: {:?}", account_set);

    sign(&mut account_set, &wallet, false).unwrap();

    println!("AccountSet transaction after signing: {:?}", account_set);
}
