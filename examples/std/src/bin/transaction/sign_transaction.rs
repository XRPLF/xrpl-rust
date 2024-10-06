use xrpl::asynch::transaction::sign;
use xrpl::models::transactions::account_set::AccountSet;
use xrpl::wallet::Wallet;

fn main() {
    // Create a new wallet we can use to sign the transaction
    let wallet = Wallet::create(None).unwrap();
    // Define the transaction we want to sign
    let mut account_set = AccountSet::new(
        wallet.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("6578616d706c652e636f6d".into()), // example.com
        None,
        None,
        None,
        None,
        None,
        None,
    );
    println!("AccountSet transaction before signing: {:?}", account_set);
    sign(&mut account_set, &wallet, false).unwrap();
    println!("AccountSet transaction after signing: {:?}", account_set);
}
