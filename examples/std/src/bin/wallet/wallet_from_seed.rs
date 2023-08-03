use xrpl::wallet::Wallet;

fn main() {
    let wallet = Wallet::new("sEdVWgwiHxBmFoMGJBoPZf6H1XSLLGd", 0)
        .expect("Failed to create wallet from seed");

    assert_eq!(wallet.classic_address, "rsAhdjbE7YXqQtubcaSwb6xHn6mU2bSFHY");
    println!("Wallet: {:?}", wallet);
}
