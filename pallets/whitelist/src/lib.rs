#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

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
use rustc_hex::ToHex;
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
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
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

	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T>{
		fn offchain_worker(block_number: T::BlockNumber){
			let res = Self::verify_whitelist_and_send_raw_unsign(block_number);
			if let Err(e) = res{
				log::error!("Error:{}",e);
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		// pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
		// 	// Check that the extrinsic was signed and get the signer.
		// 	// This function will return an error if the extrinsic is not signed.
		// 	// https://docs.substrate.io/main-docs/build/origins/
		// 	let who = ensure_signed(origin)?;
		//
		// 	// Update storage.
		// 	<Something<T>>::put(something);
		//
		// 	// Emit an event.
		// 	Self::deposit_event(Event::SomethingStored(something, who));
		// 	// Return a successful DispatchResultWithPostInfo
		// 	Ok(())
		// }

		/// An example dispatchable that may throw a custom error.
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		//
		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => return Err(Error::<T>::NoneValue.into()),
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn approve_whitelist(origin: OriginFor<T>, player: T::AccountId, pool_id: ID)-> DispatchResult{
			let sender = ensure_signed(origin)?;

			Self::is_pool_owner(pool_id, sender)?;

			ensure!(Self::is_whitelist_player(&player, pool_id), Error::<T>::NotWhitelist );

			T::WhitelistPool::join_pool(&player, pool_id);
			Whitelist::<T>::remove(&player);

			Self::deposit_event(Event::<T>::Whitelisted {sender: player, pool_id});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn apply_whitelist(origin: OriginFor<T>, pool_id: ID)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			Self::insert_whitelist(pool_id, sender);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn enable_whitelist(origin: OriginFor<T>, pool_id: ID, url: Vec<u8>)-> DispatchResult{
			let sender = ensure_signed(origin)?;

			Self::is_pool_owner(pool_id, sender.clone());

			let bounded_url: BoundedVec<_, _> = url.clone().try_into().map_err( |()| Error::<T>::URLTooLong )?;

			let deposit = T::WhitelistFee::get();

			if Self::whitelist_source(pool_id) == None{
				T::Currency::reserve(&sender, deposit);
				Self::deposit_event(Event::<T>::WhitelistEnabled {pool_id, url});
			}else{
				Self::deposit_event(Event::<T>::WhitelistChanged {pool_id, url});
			}
			WhitelistSource::<T>::insert(pool_id, (bounded_url, deposit));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn withdraw_whitelist(origin: OriginFor<T>, pool_id: ID)-> DispatchResult{
			let sender = ensure_signed(origin)?;

			Self::is_pool_owner(pool_id, sender.clone());

			if let Some(source) = Self::whitelist_source(pool_id){
				let deposit = source.1;
				T::Currency::unreserve(&sender, deposit);
			}else{
				return Err(Error::<T>::PoolNotWhitelist.into())
			}

			WhitelistSource::<T>::remove(pool_id);

			Self::deposit_event(Event::<T>::WhitelistWithdrew {pool_id});

			Ok(())
		}


		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn approve_whitelist_unsigned(origin: OriginFor<T>, player: T::AccountId, pool_id: ID)-> DispatchResult{

			Ok(())
		}

	}
}

impl<T: Config> Pallet<T> {
	pub fn verify_whitelist_and_send_raw_unsign(block_number: T::BlockNumber)-> Result<(), &'static str>{
		for query in Whitelist::<T>::iter(){
			let player = query.0;
			let pool_id = query.1;

			if let Some(url)= Self::get_url(pool_id){
				let api = Self::get_api(&url, pool_id, &player);
				log::info!("api: {} ", api);
				let _ = Self::verify_and_approve(&api, player, pool_id);
			}
		}

		Ok(())
	}

	pub fn verify_and_approve(uri: &str, player: T::AccountId, pool_id: ID)-> Result<(), &'static str> {
		let verify = Self::fetch_whitelist(uri);

		if verify == Ok(true){
			let call = Call::approve_whitelist_unsigned::<T>{player, pool_id};

			let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(|_|{
				log::error!("failed in offchain_unsigned_tx");
			});
		}

		Ok(())
	}

	pub fn is_whitelist_player(player: &T::AccountId, pool_id: ID)-> bool{
		if let Some(id) = Self::whitelist(player){
			if id == pool_id{
				return true
			}
		}
		false
	}

	pub fn fetch_whitelist(url: &str)-> Result<bool, http::Error>{
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
		let request = http::Request::get(url);
		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError )?;
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		if response.code != 200 {
			log::warn!("Unexpected status code: {} ", response.code);
			return Err(http::Error::Unknown)
		}
		let body = response.body().collect::<Vec<u8>>();
		let body_str = sp_std::str::from_utf8(&body).map_err(|_|{
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;
		let verify = matches!(body_str, "true");

		Ok(verify)
	}


	pub fn get_api(link: &str, pool_id: ID, player: &T::AccountId)-> String{
		let pool_id_hex: String = pool_id.to_hex();
		let address = player.encode();

		let hex_address: String = address.to_hex();

		let uri = format!("{link}?pool_id={pool_id_hex}&address={hex_address}");
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

impl<T: Config> IWhiteList<T::AccountId> for Pallet<T>{
	fn is_whitelist(pool_id: ID)-> bool{
		WhitelistSource::<T>::get(pool_id).is_some()
	}

	fn insert_whitelist(pool_id: ID, player: T::AccountId)-> Result<(), &'static str >{
		ensure!(T::SponsoredPool::is_pool(pool_id), Error::<T>::PoolNotFound );

		ensure!(Self::is_whitelist(pool_id), Error::<T>::PoolNotWhitelist);

		ensure!( !Self::is_whitelist_player(&player, pool_id), Error::<T>::AlreadyWhitelist );

		ensure!(!T::WhitelistPool::is_joined_pool(player.clone(), pool_id), Error::<T>::AlreadyJoined);

		Whitelist::<T>::insert(player, pool_id);

		Ok(())
	}
}
