#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]

extern crate alloc;

use alloc::vec::Vec;
use fp_evm::{ExitSucceed, LinearCostPrecompile, PrecompileFailure};
use log::info;

pub struct HelloPrecompile;

impl LinearCostPrecompile for HelloPrecompile {
    // The same as identity precompile
    const BASE: u64 = 15;
    const WORD: u64 = 1;

    fn execute(_: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        info!("=== hello-precompile ===");
        Ok((
            ExitSucceed::Returned,
            // "hello precompile".as_bytes().to_vec(),
            [1, 2, 3, 4].to_vec(),
        ))
    }
}
