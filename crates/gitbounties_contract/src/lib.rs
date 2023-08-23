use std::sync::Arc;

use ethers::{
    prelude::{abigen, k256::ecdsa::SigningKey, Abigen, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::Address,
};

mod abi {

    use ethers::prelude::abigen;

    abigen!(GitbountiesNFT, "./contract/GitbountiesNFT.json");
}
pub use abi::*;
pub use ethers::types::H160;

pub type Contract = GitbountiesNFT<Provider<Http>>;

pub fn parse_address(address: &str) -> anyhow::Result<H160> {
    let res = address.parse()?;
    Ok(res)
}

pub async fn get_contract(contract_address: &str) -> anyhow::Result<Contract> {
    let provider = Provider::<Http>::try_from("http://127.0.0.1:8545")?;
    let client = Arc::new(provider);
    let address: Address = contract_address.parse()?;
    let contract = GitbountiesNFT::new(address, client);

    Ok(contract)
}
