//! ## Overview
//!
//! In this example we are going to build a very simplistic, naive and definitely NOT
//! production-ready oracle for BTC/USD price.
//! Offchain Worker (OCW) will be triggered after every block, fetch the current price
//! and prepare either signed or unsigned transaction to feed the result back on chain.
//! The on-chain logic will simply aggregate the results and store last `64` values to compute
//! the average price.
//! Additional logic in OCW is put in place to prevent spamming the network with both signed
//! and unsigned transactions, and custom `UnsignedValidator` makes sure that there is only
//! one unsigned transaction floating in the network.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::{
    self as system,
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
        SignedPayload, Signer, SigningTypes, SubmitTransaction,
    },
    pallet_prelude::BlockNumberFor,
};
use lite_json::json::JsonValue;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain::{
        http,
        storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
        Duration,
    },
    traits::Zero,
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    RuntimeDebug,
};

// TODO: add test.rs according to
//       "https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/examples/offchain-worker/src/tests.rs"
// #[cfg(test)]
// mod tests;

// TODO: consideration
/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"prc!");

// TODO: consideration
/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
    use super::KEY_TYPE;
    use alloc::string::String;
    use fp_account::{AccountId20, EthereumSignature, EthereumSigner};
    use frame_system::offchain::AppCrypto;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, ecdsa, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };

    // TODO
    // app_crypto!(ecdsa, KEY_TYPE);
    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    // TODO
    // impl AppCrypto<<ecdsa::Signature as Verify>::Signer, ecdsa::Signature> for TestAuthId {
    //     type RuntimeAppPublic = Public;
    //     type GenericSignature = ecdsa::Signature;
    //     type GenericPublic = ecdsa::Public;
    // }

    // TODO
    // impl AppCrypto<AccountId20, EthereumSignature> for TestAuthId {
    //     type RuntimeAppPublic = Public;
    //     type GenericSignature = EthereumSignature;
    //     type GenericPublic = AccountId20; // ecdsa::Public;
    // }

    impl AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // TODO
    // implemented for mock runtime in test
    impl AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

// mod abc {
//     use sp_core::ecdsa;
//     // use frame_support::crypto::ecdsa;
//     use fp_account::{AccountId20, EthereumSignature, EthereumSigner};
//     use sp_runtime::CryptoTypeId;

//     pub type Public = EthereumSigner;
//     pub type Signature = EthereumSignature;
//     pub type Pair = ecdsa::Pair;
//     pub const CRYPTO_ID: CryptoTypeId = ecdsa::CRYPTO_ID;
// }

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::{offchain::SendTransactionTypes, pallet_prelude::*};

    #[rustfmt::skip]
    #[pallet::config]
    pub trait Config: SendTransactionTypes<Call<Self>> + frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent       : From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A grace period after we send transaction.
        ///
        /// To avoid sending too many transactions, we only attempt to send one
        /// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
        /// sending between distinct runs of this offchain worker.
        #[pallet::constant]
        type GracePeriod        : Get<BlockNumberFor<Self>>;
        /// Maximum number of prices.
        #[pallet::constant]
        type MaxPrices          : Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Offchain Worker entry point.
        ///
        /// By implementing `fn offchain_worker` you declare a new offchain worker.
        /// This function will be called when the node is fully synced and a new best block is
        /// successfully imported.
        /// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
        /// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
        /// so the code should be able to handle that.
        /// You can use `Local Storage` API to coordinate runs of the worker.
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // Note that having logs compiled to WASM may cause the size of the blob to increase
            // significantly. You can use `RuntimeDebug` custom derive to hide details of the types
            // in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
            // all logging and thus, remove any logging from the WASM.
            log::info!("Hello World from offchain workers!");

            // Since off-chain workers are just part of the runtime code, they have direct access
            // to the storage and other included pallets.
            //
            // We can easily import `frame_system` and retrieve a block hash of the parent block.
            let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
            log::debug!(
                "Current block: {:?} (parent hash: {:?})",
                block_number,
                parent_hash
            );

            // It's a good practice to keep `fn offchain_worker()` function minimal, and move most
            // of the code to separate `impl` block.
            // Here we call a helper function to calculate current average price.
            // This function reads storage entries of the current state.
            let average: Option<u32> = Self::average_price();
            log::debug!("Current price: {:?}", average);

            let should_send = Self::choose_transaction_type(block_number);
            let res = match should_send {
                TransactionType::Raw => Self::fetch_price_and_send_raw_unsigned(block_number),
                TransactionType::None => Ok(()),
            };
            if let Err(e) = res {
                log::error!("Error: {}", e);
            }
        }
    }

    /// A public part of the pallet.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit new price to the list via unsigned transaction.
        ///
        /// Works exactly like the `submit_price` function, but since we allow sending the
        /// transaction without a signature, and hence without paying any fees,
        /// we need a way to make sure that only some transactions are accepted.
        /// This function can be called only once every `T::UnsignedInterval` blocks.
        /// Transactions that call that function are de-duplicated on the pool level
        /// via `validate_unsigned` implementation and also are rendered invalid if
        /// the function has already been called in current "session".
        ///
        /// It's important to specify `weight` for unsigned calls as well, because even though
        /// they don't charge fees, we still don't want a single block to contain unlimited
        /// number of such transactions.
        ///
        /// This example is not focused on correctness of the oracle itself, but rather its
        /// purpose is to showcase offchain worker capabilities.
        #[pallet::call_index(0)]
        #[pallet::weight({0})] // TODO
        pub fn submit_price_unsigned(
            origin: OriginFor<T>,
            _block_number: BlockNumberFor<T>,
            price: u32,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the price to the on-chain list, but mark it as coming from an empty address.
            Self::add_price(None, price);
            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <system::Pallet<T>>::block_number();
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event generated when new price is accepted to contribute to the average.
        NewPrice {
            price: u32,
            maybe_who: Option<T::AccountId>,
        },
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            if let Call::submit_price_unsigned {
                block_number,
                price: new_price,
            } = call
            {
                Self::validate_transaction_parameters(block_number, new_price)
            } else {
                InvalidTransaction::Call.into()
            }
        }
    }

    /// A vector of recently submitted prices.
    ///
    /// This is used to calculate average price, should have bounded size.
    #[pallet::storage]
    pub type Prices<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxPrices>, ValueQuery>;
}

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct PricePayload<Public, BlockNumber> {
    block_number: BlockNumber,
    price: u32,
    public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, BlockNumberFor<T>> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

enum TransactionType {
    Raw,
    None,
}

impl<T: Config> Pallet<T> {
    /// Chooses which transaction type to send.
    ///
    /// This function serves mostly to showcase `StorageValue` helper
    /// and local storage usage.
    ///
    /// Returns a type of transaction that should be produced in current run.
    fn choose_transaction_type(block_number: BlockNumberFor<T>) -> TransactionType {
        /// A friendlier name for the error that is going to be returned in case we are in the grace
        /// period.
        const RECENTLY_SENT: () = ();

        // Start off by creating a reference to Local Storage value.
        // Since the local storage is common for all offchain workers, it's a good practice
        // to prepend your entry with the module name.
        let val = StorageValueRef::persistent(b"example_ocw::last_send");
        // The Local Storage is persisted and shared between runs of the offchain workers,
        // and offchain workers may run concurrently. We can use the `mutate` function, to
        // write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
        // low-level method of local storage API, which means that only one worker
        // will be able to "acquire a lock" and send a transaction if multiple workers
        // happen to be executed concurrently.
        let res = val.mutate(
            |last_send: Result<Option<BlockNumberFor<T>>, StorageRetrievalError>| {
                match last_send {
                    // If we already have a value in storage and the block number is recent enough
                    // we avoid sending another transaction at this time.
                    Ok(Some(block)) if block_number < block + T::GracePeriod::get() => {
                        Err(RECENTLY_SENT)
                    }
                    // In every other case we attempt to acquire the lock and send a transaction.
                    _ => Ok(block_number),
                }
            },
        );

        // The result of `mutate` call will give us a nested `Result` type.
        // The first one matches the return of the closure passed to `mutate`, i.e.
        // if we return `Err` from the closure, we get an `Err` here.
        // In case we return `Ok`, here we will have another (inner) `Result` that indicates
        // if the value has been set to the storage correctly - i.e. if it wasn't
        // written to in the meantime.
        match res {
            // The value has been set correctly, which means we can safely send a transaction now.
            Ok(_block_number) => TransactionType::Raw,
            // We are in the grace period, we should not send a transaction this time.
            Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => TransactionType::None,
            // We wanted to send a transaction, but failed to write the block number (acquire a
            // lock). This indicates that another offchain worker that was running concurrently
            // most likely executed the same logic and succeeded at writing to storage.
            // Thus we don't really want to send the transaction, knowing that the other run
            // already did.
            Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
        }
    }

    /// A helper function to fetch the price and send a raw unsigned transaction.
    fn fetch_price_and_send_raw_unsigned(
        block_number: BlockNumberFor<T>,
    ) -> Result<(), &'static str> {
        // Make an external HTTP request to fetch the current price.
        // Note this call will block until response is received.
        let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

        // Received price is wrapped into a call to `submit_price_unsigned` public function of this
        // pallet. This means that the transaction, when executed, will simply call that function
        // passing `price` as an argument.
        let call = Call::submit_price_unsigned {
            block_number,
            price,
        };

        // Now let's create a transaction out of this call and submit it to the pool.
        // Here we showcase two ways to send an unsigned transaction / unsigned payload (raw)
        //
        // By default unsigned transactions are disallowed, so we need to whitelist this case
        // by writing `UnsignedValidator`. Note that it's EXTREMELY important to carefully
        // implement unsigned validation logic, as any mistakes can lead to opening DoS or spam
        // attack vectors. See validation logic docs for more details.
        //
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|()| "Unable to submit unsigned transaction.")?;

        Ok(())
    }

    /// Fetch current price and return the result in cents.
    fn fetch_price() -> Result<u32, http::Error> {
        // We want to keep the offchain worker execution time reasonable, so we set a hard-coded
        // deadline to 2s to complete the external call.
        // You can also wait indefinitely for the response, however you may still get a timeout
        // coming from the host machine.
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
        // Initiate an external HTTP GET request.
        // This is using high-level wrappers from `sp_runtime`, for the low-level calls that
        // you can find in `sp_io`. The API is trying to be similar to `request`, but
        // since we are running in a custom WASM execution environment we can't simply
        // import the library here.
        let request =
            // TODO
            // http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD");
            http::Request::get("http://127.0.0.1:8080/price?currency=ETH/USD");
        // We set the deadline for sending of the request, note that awaiting response can
        // have a separate deadline. Next we send the request, before that it's also possible
        // to alter request headers or stream body content in case of non-GET requests.
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;

        // The request is already being processed by the host, we are free to do anything
        // else in the worker (we can send multiple concurrent requests too).
        // At some point however we probably want to check the response though,
        // so we can block current thread and wait for it to finish.
        // Note that since the request is being driven by the host, we don't have to wait
        // for the request to have it complete, we will just not read the response.
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        // Let's check the status code before we proceed to reading the response.
        if response.code != 200 {
            log::warn!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        // Next we want to fully read the response body and collect it to a vector of bytes.
        // Note that the return object allows you to read the body in chunks as well
        // with a way to control the deadline.
        let body = response.body().collect::<Vec<u8>>();

        // Create a str slice from the body.
        let body_str = alloc::str::from_utf8(&body).map_err(|_| {
            log::warn!("No UTF8 body");
            http::Error::Unknown
        })?;

        let price = match Self::parse_price(body_str) {
            Some(price) => Ok(price),
            None => {
                log::warn!("Unable to extract price from the response: {:?}", body_str);
                Err(http::Error::Unknown)
            }
        }?;

        log::warn!("Got price: {} cents", price);

        Ok(price)
    }

    /// Parse the price from the given JSON string using `lite-json`.
    ///
    /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
    fn parse_price(price_str: &str) -> Option<u32> {
        price_str.parse().ok()
        // let val = lite_json::parse_json(price_str);
        // let price = match val.ok()? {
        //     JsonValue::Object(obj) => {
        //         let (_, v) = obj
        //             .into_iter()
        //             .find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
        //         match v {
        //             JsonValue::Number(number) => number,
        //             _ => return None,
        //         }
        //     }
        //     _ => return None,
        // };

        // let exp = price.fraction_length.saturating_sub(2);
        // Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
    }

    /// Parse the price from the given JSON string using `lite-json`.
    ///
    /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
    fn parse_price_(price_str: &str) -> Option<u32> {
        let val = lite_json::parse_json(price_str);
        let price = match val.ok()? {
            JsonValue::Object(obj) => {
                let (_, v) = obj
                    .into_iter()
                    .find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
                match v {
                    JsonValue::Number(number) => number,
                    _ => return None,
                }
            }
            _ => return None,
        };

        let exp = price.fraction_length.saturating_sub(2);
        Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
    }

    /// Add new price to the list.
    fn add_price(maybe_who: Option<T::AccountId>, price: u32) {
        log::info!("Adding to the average: {}", price);
        <Prices<T>>::mutate(|prices| {
            if prices.try_push(price).is_err() {
                prices[(price % T::MaxPrices::get()) as usize] = price;
            }
        });

        let average = Self::average_price()
            .expect("The average is not empty, because it was just mutated; qed");
        log::info!("Current average price is: {}", average);
        // here we are raising the NewPrice event
        Self::deposit_event(Event::NewPrice { price, maybe_who });
    }

    /// Calculate current average price.
    pub fn average_price() -> Option<u32> {
        let prices = Prices::<T>::get();
        if prices.is_empty() {
            None
        } else {
            Some(prices.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / prices.len() as u32)
        }
    }

    fn validate_transaction_parameters(
        block_number: &BlockNumberFor<T>,
        new_price: &u32,
    ) -> TransactionValidity {
        // TODO: remove
        // // Now let's check if the transaction has any chance to succeed.
        // let next_unsigned_at = NextUnsignedAt::<T>::get();
        // if &next_unsigned_at > block_number {
        //     return InvalidTransaction::Stale.into();
        // }
        // Let's make sure to reject transactions from the future.
        let current_block = <system::Pallet<T>>::block_number();
        if &current_block < block_number {
            return InvalidTransaction::Future.into();
        }

        // We prioritize transactions that are more far away from current average.
        //
        // Note this doesn't make much sense when building an actual oracle, but this example
        // is here mostly to show off offchain workers capabilities, not about building an
        // oracle.
        let avg_price = Self::average_price()
            .map(|price| {
                if &price > new_price {
                    price - new_price
                } else {
                    new_price - price
                }
            })
            .unwrap_or(0);

        ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
            // TODO
            // // We set base priority to 2**20 and hope it's included before any other
            // // transactions in the pool. Next we tweak the priority depending on how much
            // // it differs from the current average. (the more it differs the more priority it
            // // has).
            // .priority(T::UnsignedPriority::get().saturating_add(avg_price as _))
            // // This transaction does not require anything else to go before into the pool.
            // // In theory we could require `previous_unsigned_at` transaction to go first,
            // // but it's not necessary in our case.
            // //.and_requires()
            // // We set the `provides` tag to be the same as `next_unsigned_at`. This makes
            // // sure only one transaction produced after `next_unsigned_at` will ever
            // // get to the transaction pool and will end up in the block.
            // // We can still have multiple transactions compete for the same "spot",
            // // and the one with higher priority will replace other one in the pool.
            // .and_provides(next_unsigned_at)
            // The transaction is only valid for next 5 blocks. After that it's
            // going to be revalidated by the pool.
            .longevity(5)
            // It's fine to propagate that transaction to other peers, which means it can be
            // created even by nodes that don't produce blocks.
            // Note that sometimes it's better to keep it for yourself (if you are the block
            // producer), since for instance in some schemes others may copy your solution and
            // claim a reward.
            .propagate(true)
            .build()
    }
}
