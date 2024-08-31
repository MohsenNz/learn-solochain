use fp_account::AccountId20;
use hex_literal::hex;
use solochain_template_runtime::{AccountId, Signature};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ecdsa, sr25519, ByteArray, Pair, Public, H160, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::{collections::HashMap, fmt::Display};
use EthAccountKeyring::*;

pub enum EthAccountKeyring {
    Alice,
    Bob,
    AliceStash,
    BobStash,
}

impl EthAccountKeyring {
    pub fn account_id20(&self) -> AccountId20 {
        AccountId20::from(match self {
            Alice => hex!("1f31d7740f9b822edd0ea965f4cfcf1034c450a2"),
            Bob => hex!("8347187707fea1e2418c8534998d9b8d26cd6430"),
            AliceStash => hex!("81593023a7f69346a047421099a8085ed0f44f7e"),
            BobStash => hex!("e1cb409bc4e2002e12875a1b0c528994fe3ef23e"),
        })
    }

    pub fn authority_keys(&self) -> (AuraId, GrandpaId) {
        let s = self.seed();
        authority_keys_from_seed(&s)
    }

    pub fn seed(&self) -> String {
        String::from(match self {
            Alice => "Alice",
            Bob => "Bob",
            AliceStash => "AliceStash",
            BobStash => "BobStash",
        })
    }

    pub fn ecdsa_pair(&self) -> ecdsa::Pair {
        ecdsa::Pair::from_string(&self.seed(), None).expect("static values are valid; qed")
    }
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use sp_core::crypto::Ss58Codec;
    use std::str::FromStr;
    use AccountKeyring::*;

    use super::*;

    #[test]
    fn test_account_keyring_string_name() {
        assert_eq!(AccountKeyring::Bob.to_string(), "Bob".to_string());
        assert_eq!(AccountKeyring::Bob.to_seed(), "//Bob".to_string());
        assert_eq!(AccountKeyring::BobStash.to_string(), "BobStash".to_string());
        assert_eq!(AccountKeyring::BobStash.to_seed(), "//BobStash".to_string());
    }

    #[test]
    fn test_getting_account_id20() {
        let alice = AccountKeyring::Alice;

        let alice_account_id32 = alice.to_account_id();

        // check SS58 address
        assert_eq!(
            alice_account_id32.to_ss58check(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()
        );

        assert_eq!(
            account_id20_of(alice),
            // H160 address of Alice dev account
            // Derived from SS58 (42 prefix) address
            // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
            // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
            // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
            AccountId20::from_str("d43593c715fdd31c61141abd04a99fd6822c8558").unwrap(),
        );

        assert_eq!(
            account_id20_of(alice),
            // H160 address of Alice dev account
            // Derived from SS58 (42 prefix) address
            // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
            // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
            // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
            AccountId20::from(hex!("d43593c715fdd31c61141abd04a99fd6822c8558")),
        );
    }
}
