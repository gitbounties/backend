//! Interface with blockchain
/*

use ethers::prelude::*;

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

async fn setup() -> anyhow::Result<()> {
    let provider = Provider::<Http>::try_from("ENDPOINT")?;

    let wallet: LocalWallet = "PRIVATE_KEY"
        .parse::<LocalWallet>()?
        .with_chain_id(Chain::AnvilHardhat);

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    Ok(())
}
*/
