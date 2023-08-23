mod abi {

    use ethers::prelude::abigen;

    abigen!(GitbountiesNFT, "./contract/GitbountiesNFT.json");
}
pub use abi::*;
