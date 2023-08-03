use xrpl::wallet::Wallet;

fn main() {
    let wallet = Wallet::create(None).expect("Failed to generate new wallet");

    println!("Wallet: {:?}", wallet);
}
