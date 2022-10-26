#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency}
};

use gafi_primitives::{
	constant::ID,
	custom_services::CustomPool,
	whitelist::{IWhiteList, WhitelistPool}
};

use scale_info::prelude::{format, string::String};
use sp_std::{prelude::*, str};

use frame_system::offchain::{CreateSignedTransaction, SubmitTransaction};
use sp_core::crypto::KeyTypeId;
use sp_runtime::offchain::{http, Duration};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"gafi");
pub const UNSIGNED_TXS_PRIORITY: u64 = 10;

pub mod crypto{
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner
	};
	app_crypto!(sr25519, KEY_TYPE);
	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type WhitelistPool: WhitelistPool<Self::AccountId>;

		type SponsoredPool: CustomPool<Self::AccountId>;

		type MaxWhitelistLength: Get<u32>;

		#[pallet::constant]
		type WhitelistFee: Get<BalanceOf<Self>>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn whitelist)]
	pub type Whitelist<T:Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

	#[pallet::storage]
	#[pallet::getter(fn whitelist_source)]
	pub type WhitelistSource<T:Config> = StorageMap<_, Twox64Concat, ID, (BoundedVec<u8, T::MaxWhitelistLength>, BalanceOf<T>)>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		Whitelisted{ sender: T::AccountId, pool_id: ID},
		WhitelistEnabled{pool_id: ID, url: Vec<u8>},
		WhitelistChanged{pool_id: ID, url: Vec<u8>},
		WhitelistWithdrew{pool_id: ID}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		NotWhitelist,
		AlreadyWhitelist,
		NotPoolOwner,
		PoolNotFound,
		PoolNotWhitelist,
		URLTooLong,
		AlreadyJoined
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn verify_whitelist_and_send_raw_unsign(block_number: T::BlockNumber)-> Result<(), &'static str>{
		for query in Whitelist::<T>::iter(){
			let player = query.0;
			let pool_id = query.1;

			//if let Some(url)=
		}

		Ok(())
	}

	pub fn get_api(link: &str, pool_id: ID, player: T::AccountId)-> String{
		let pool_id_hex:String = pool_id.to_hex();
		let address = player.encode();

		let hex_address: String = address.to_hex();

		let uri = format!("");
		uri
	}

	pub fn get_url(pool_id: ID)-> Option<String>{
		if let Some(source) = WhitelistSource::<T>::get(pool_id){
			if let Ok(url) = sp_std::str::from_utf8(&source.0){
				return Some(format!("{}", url))
			}
		}

		None
	}

	fn is_pool_owner(pool_id: ID, sender: T::AccountId)-> Result<(), Error<T>>{
		if let Some(owner) = T::SponsoredPool::get_pool_owner(pool_id){
			if owner == sender{
				return Ok(())
			}else{
				return Err(Error::<T>::NotPoolOwner)
			}
		}

		Err(Error::<T>::PoolNotFound)
	}
}
