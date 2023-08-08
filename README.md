<div align="center">

# gitbounties_backend

[![crates.io](https://img.shields.io/crates/v/gitbounties_backend.svg)](https://crates.io/crates/gitbounties_backend)
[![docs.rs](https://docs.rs/gitbounties_backend/badge.svg)](https://docs.rs/gitbounties_backend)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#)

</div>

## Prerequisites

Some tools that will be used
- [just](https://github.com/casey/just): used as command runner
- [foundry](https://github.com/foundry-rs/foundry): ethereum testing suite
- [solc](#): Compiler for solidity
- [surrealdb](): used to run repl to surrealdb docker container

You will also need a metamask account to do local testing of the smart contract

## Setting up for development

Install git hooks for developers:
```
just devsetup
```

Create your own copy of env file and fill out the variables with the secrets
```
cp .env-example .env
```

## Running for development

We need an instance of surreal db running. You can run a local test database with.
```
just dev_db
```

If doing smart contract development, you can run a local node
```
anvil
```
