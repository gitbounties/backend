use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use ethers::{
    contract::Abigen,
    solc::{CompilerInput, Solc},
};

pub fn main() {
    compile_contract(
        "Gitbounties",
        &PathBuf::from("contracts/Gitbounties.sol"),
        &PathBuf::from("abi/Gitbounties.json"),
    );
}

pub fn compile_contract(contract_name: &str, contract_path: &Path, output_path: &Path) {
    let solc = Solc::default();
    let output = solc.compile_source(contract_path).unwrap();

    if output.has_error() {
        for e in output.errors.iter() {
            eprintln!("{:?}", e);
        }
    }

    let abi = output
        .get(contract_path.to_str().unwrap(), contract_name)
        .unwrap()
        .abi
        .unwrap();
    let abi_spec = serde_json::to_string(abi).unwrap();

    // let bindings = Abigen::new(&contract_name, abi_spec).unwrap().generate().unwrap();

    // write output
    std::fs::create_dir_all("abi").unwrap();
    // bindings.write_to_file("abi/Gitbounties.json").unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("abi/Gitbounties.json")
        .unwrap();
    file.write_all(abi_spec.to_string().as_bytes()).unwrap();
}
