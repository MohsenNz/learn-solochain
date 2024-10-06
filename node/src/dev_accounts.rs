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
            Alice => "//Alice",
            Bob => "//Bob",
            AliceStash => "//AliceStash",
            BobStash => "//BobStash",
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
    TPublic::Pair::from_string(seed, None)
        .expect("static values are valid; qed")
        .public()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use sp_core::crypto::Ss58Codec;
    use std::str::FromStr;

    use super::*;

    // TODO
    #[test]
    fn f() {
        let l = [Alice, Bob, AliceStash, BobStash]
            .into_iter()
            .map(|x| x.authority_keys());

        for (i, (a, g)) in l.enumerate() {
            println!("=== index: {i} ===");
            println!("aura SS58     : {}", a.to_ss58check());
            println!("aura Hex      : {:?}", a);
            println!("grandpa SS58  : {}", g.to_ss58check());
            println!("grandpa Hex   : {:?}", g);
        }
    }
}
