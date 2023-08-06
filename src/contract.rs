use std::{path::PathBuf, sync::Arc, time::Duration};

use ethers::{
    prelude::*,
    solc::{Artifact, Project, ProjectPathsConfig},
    utils::Anvil,
};
use log::{error, info};

abigen!(Gitbounties, "./abi/Gitbounties.json");

pub async fn deploy() -> anyhow::Result<()> {
    // project setup
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let paths = ProjectPathsConfig::builder()
        .root(&root)
        .sources(&root)
        .build()
        .unwrap();
    let project = Project::builder()
        .paths(paths)
        .ephemeral()
        .no_artifacts()
        .build()
        .unwrap();

    // compile project
    let output = project.compile().unwrap();
    let contract = output.find_first("Gitbounties").unwrap().clone();

    let (abi, bytecode, _) = contract.into_parts();

    // init anvil
    let anvil = Anvil::new().spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();

    // connect to anvil network
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));
    let client = Arc::new(client);

    let factory = ContractFactory::new(abi.unwrap(), bytecode.unwrap(), client.clone());
    let contract = factory.deploy(0u64).unwrap().send().await.unwrap();

    let addr = contract.address();

    println!("contract address {}", addr);

    let contract = Gitbounties::new(addr, client.clone());

    Ok(())
}

#[cfg(test)]
mod tests {
    use log::{error, LevelFilter};

    use super::deploy;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    async fn test_deploy() {
        init();

        deploy().await.unwrap();
    }
}
