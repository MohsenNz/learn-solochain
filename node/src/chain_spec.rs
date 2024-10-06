use std::{collections::BTreeMap, str::FromStr};

use crate::dev_accounts::EthAccountKeyring::*;
use crate::dev_accounts::*;
use hex_literal::hex;
use sc_service::{ChainType, Properties};
use solochain_template_runtime::{AccountId, Balance, SS58Prefix, Signature, WASM_BINARY};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, H160, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenDecimals".into(), 18.into());
    // properties.insert("tokenSymbol".to_string(), "XYZ".into()); // TODO
    properties.insert("ss58Format".into(), SS58Prefix::get().into());
    properties
}

pub fn development_config(enable_manual_seal: bool) -> ChainSpec {
    ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
        .with_name("Development")
        .with_id("dev")
        .with_chain_type(ChainType::Development)
        .with_properties(properties())
        .with_genesis_config_patch(testnet_genesis(
            // Sudo account
            Alice.account_id20(),
            // Pre-funded accounts
            vec![
                // Mohsen's metamask
                AccountId::from(hex!("7B0A6D8EC377ef7458d4fE40E86bC74884C87b00")),
                Alice.account_id20(),
                Bob.account_id20(),
                AliceStash.account_id20(),
                BobStash.account_id20(),
            ],
            // Initial PoA authorities
            vec![Alice.authority_keys()],
            // Ethereum chain ID
            SS58Prefix::get() as u64,
            enable_manual_seal,
        ))
        .build()
}

pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
        .with_name("Local Testnet")
        .with_id("local_testnet")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_patch(testnet_genesis(
            // Sudo account
            Alice.account_id20(),
            // Pre-funded accounts
            vec![
                Alice.account_id20(),
                Bob.account_id20(),
                AliceStash.account_id20(),
                BobStash.account_id20(),
                // TODO
                // Charlie.account_id20(),
                // Dave.account_id20(),
                // Eve.account_id20(),
                // Ferdie.account_id20(),
                // CharlieStash.account_id20(),
                // DaveStash.account_id20(),
                // EveStash.account_id20(),
                // FerdieStash.account_id20(),
            ],
            // Initial PoA authorities
            vec![Alice.authority_keys(), Bob.authority_keys()],
            42,
            false,
        ))
        .build()
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    sudo_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    chain_id: u64,
    enable_manual_seal: bool,
) -> serde_json::Value {
    const UNITS: Balance = 1_000_000_000_000_000_000;

    let evm_accounts = {
        BTreeMap::from([
            (
                // H160 address of Alice dev account
                // Derived from SS58 (42 prefix) address
                // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
                // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
                // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
                H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
                    .expect("internal H160 is valid; qed"),
                fp_evm::GenesisAccount {
                    balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                        .expect("internal U256 is valid; qed"),
                    code: Default::default(),
                    nonce: Default::default(),
                    storage: Default::default(),
                },
            ),
            (
                // private-key: 0x1650222a42e77da936f8c183458814cd144e7648d683b0ab7321bd2032030c08
                H160::from_str("AE4F6dFcdB369A6C43565b1658Ac759960a60aCb")
                    .expect("internal H160 is valid; qed"),
                fp_evm::GenesisAccount {
                    balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                        .expect("internal U256 is valid; qed"),
                    code: Default::default(),
                    nonce: Default::default(),
                    storage: Default::default(),
                },
            ),
            (
                // private-key: 0x096e09370e3b22537c854ffc517514ad7f31639b377488a252868b4276d35826
                H160::from_str("4dE6268744b93Ed85b4e2CF683686f4A6086293a")
                    .expect("internal H160 is valid; qed"),
                fp_evm::GenesisAccount {
                    nonce: U256::from(1),
                    balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                    storage: Default::default(),
                    code: vec![0x00],
                },
            ),
        ])
    };

    serde_json::json!({
        "sudo": {
            // Assign network admin rights.
            "key": Some(sudo_key),
        },
        "balances": {
            "balances": endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 2_000_000 * UNITS))
                .collect::<Vec<_>>()
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
        },
        "evmChainId": { "chainId": chain_id },
        "evm": { "accounts": evm_accounts },
        "manualSeal": { "enable": enable_manual_seal }
    })
}
