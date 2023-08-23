use std::sync::Arc;

use ethers::{
    middleware::SignerMiddleware,
    prelude::{
        abigen,
        k256::{ecdsa::SigningKey, SecretKey},
        Abigen,
    },
    providers::{Http, Provider},
    signers::{LocalWallet, Signer, Wallet},
    solc::{Artifact, Project, ProjectPathsConfig},
    types::{Address, Chain, U256},
};

mod abi {

    use ethers::prelude::abigen;

    abigen!(GitbountiesNFT, "./contract/GitbountiesNFT.json");
}
pub use abi::*;
pub use ethers::types::H160;

pub type Contract = GitbountiesNFT<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

pub fn parse_address(address: &str) -> anyhow::Result<H160> {
    let res = address.parse()?;
    Ok(res)
}

pub async fn get_contract(
    provider: &Provider<Http>,
    contract_address: &str,
    private_key: &str,
) -> anyhow::Result<Contract> {
    let wallet: LocalWallet = private_key
        .parse::<LocalWallet>()?
        .with_chain_id(Chain::AnvilHardhat);

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let client = Arc::new(client);
    let address: Address = contract_address.parse()?;
    let contract = GitbountiesNFT::new(address, client);

    Ok(contract)
}

pub fn http_provider() -> Provider<Http> {
    Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap()
}

#[cfg(test)]
mod tests {
    use ethers::{
        providers::{Http, Middleware, Provider},
        types::{Address, U256},
        utils::WEI_IN_ETHER,
    };

    use crate::{get_contract, http_provider, TransferFilter};

    /*
    #[tokio::test]
    async fn get_contract_test() {
        // TODO hardcoded test contract address
        let contract_addr = "0xb19b36b1456E65E3A6D514D3F715f204BD59f431";
        let priv_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

        assert!(get_contract(contract_addr, priv_key).await.is_ok());
    }
    */

    /// Test entire gitbounty nft lifecycle
    #[tokio::test]
    async fn end_to_end() -> anyhow::Result<()> {
        let provider = http_provider();

        // hardcoded address for use with anvil
        let contract_addr = "0xb19b36b1456E65E3A6D514D3F715f204BD59f431";
        let op_addr: Address = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720".parse()?;
        let u1_addr: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;
        let u2_addr: Address = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".parse()?;
        let op_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
        let u1_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
        let u2_key = "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a";

        let u1_contract = get_contract(&provider, contract_addr, u1_key).await?;
        let op_contract = get_contract(&provider, contract_addr, op_key).await?;

        let u1_balance = provider.get_balance(u1_addr, None).await?;
        let u2_balance = provider.get_balance(u2_addr, None).await?;
        println!("u1: {}, u2: {}", u1_balance, u2_balance);

        // mint the nft
        let _reciept = u1_contract.mint().send().await?.await?;

        let logs = u1_contract
            .transfer_filter()
            .from_block(0u64)
            .query()
            .await?;

        let token_id = logs.last().unwrap().token_id;
        println!("minted token with id {:?}", token_id);

        // deposit ether into it
        let _reciept = u1_contract
            .add_eth(token_id.into())
            .value(U256::from(WEI_IN_ETHER) * 69)
            .send()
            .await?
            .await?;

        // operator transfer ownership
        let _reciept = op_contract
            .transfer_token(token_id.into(), u2_addr)
            .send()
            .await?
            .await?;
        let _reciept = op_contract.burn(token_id.into()).send().await?.await?;

        let u1_balance = provider.get_balance(u1_addr, None).await?;
        let u2_balance = provider.get_balance(u2_addr, None).await?;
        println!("u1: {}, u2: {}", u1_balance, u2_balance);

        // let events = u1_contract.event::<TransferFilter>();
        // let mut stream = events.stream().await?;
        // while let Some(Ok(f)) = stream.next().await {
        //     println!("ApprovalFilter event: {f:?}");
        // }

        Ok(())
    }
}
