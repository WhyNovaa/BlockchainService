# BlockchainService 

## Getting started

###  Ensure you have Rust installed. If not, install it from [here](https://www.rust-lang.org/)

### Set up virtual environment(use guide [here](https://github.com/paritytech/polkadot-sdk-minimal-template))

### Clone the repository:
```
git clone https://github.com/WhyNovaa/BlockchainService.git
cd BlockchainService
```
### Build & run project:
```
cargo build
```
```
cargo run
```

## GET http://localhost:8080/api/balances/:address/:block_no:
### Returns the balance for the account address at the time of block number block_no.

## POST  http://localhost:8080/api/balances/:address:
### Adds the address account to the list of monitored addresses.
