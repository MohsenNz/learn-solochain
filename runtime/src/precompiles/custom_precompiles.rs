#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]

extern crate alloc;

use crate::{pallet_offchain_worker, Runtime};
use alloc::vec::Vec;
use fp_evm::ExitError;
use fp_evm::{ExitSucceed, LinearCostPrecompile, PrecompileFailure};
use frame_support::StorageValue;
use log::info;
use sp_runtime::offchain::http::Request;

pub struct HelloPrecompile;

impl LinearCostPrecompile for HelloPrecompile {
    // The same as identity precompile
    const BASE: u64 = 15;
    const WORD: u64 = 1;

    fn execute(_: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        Ok((
            ExitSucceed::Returned,
            "hello precompile".as_bytes().to_vec(),
        ))
    }
}

pub struct OraclePrecompile;

impl LinearCostPrecompile for OraclePrecompile {
    // The same as identity precompile
    const BASE: u64 = 15;
    const WORD: u64 = 1;

    fn execute(_: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let price = pallet_offchain_worker::Pallet::<Runtime>::average_price().unwrap_or(0);
        info!("hello-precompile: price: {price:?}");
        Ok((ExitSucceed::Returned, price.to_be_bytes().to_vec()))
    }
}
