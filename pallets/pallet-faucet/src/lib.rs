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

use frame_support::traits::{Currency, ExistenceRequirement};
use gafi_primitives::cache::Cache;


#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T> =<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AccountOf<T> = <T as frame_system::Config>::AccountId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type MaxGenesisAccount: Get<u32>;

		type Cache: Cache<Self::AccountId, AccountOf<Self>, u128>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;


	#[pallet::storage]
	#[pallet::getter(fn genesis_account)]
	pub type GenesisAccounts<T:Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxGenesisAccount>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn faucet_amount)]
	pub type FaucetAmount<T:Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;


	#[pallet::genesis_config]
	pub struct GenesisConfig<T:Config>{
		pub genesis_accounts: Vec<T::AccountId>,
		pub faucet_amount: BalanceOf<T>
	}

	#[cfg(feature = "std")]
	impl<T:Config> Default for GenesisConfig<T>{
		fn default()-> Self{
			Self{genesis_accounts: vec![], faucet_amount: BalanceOf::<T>::default()}
		}
	}

	#[pallet::genesis_build]
	impl<T:Config> GenesisBuild<T> for GenesisConfig<T>{
		fn build(&self){
			for i in 0..self.genesis_accounts.len(){
				GenesisAccounts::<T>::try_append(self.genesis_accounts[i].clone()).map_or((), |_|{});
			}
			FaucetAmount::<T>::put(self.faucet_amount);
		}
	}





	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		Transfered(T::AccountId, T::AccountId, BalanceOf<T>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		TransferToSelf,
		NotEnoughBalance,
		DontBeGreedy,
		PleaseWait
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
		pub fn faucet(origin: OriginFor<T>)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			let genesis_accounts = Self::genesis_account();

			let faucet_amount = Self::faucet_amount();

			ensure!(Self::get_cache(sender.clone()) == None, Error::<T>::PleaseWait );

			ensure!( T::Currency::free_balance(&sender) < faucet_amount/10u128.try_into().ok().unwrap(), Error::<T>::DontBeGreedy );

			for account in genesis_accounts{
				match T::Currency::transfer(&account, &sender, faucet_amount, ExistenceRequirement::KeepAlive){
					Ok(_)=>{
						Self::insert_cache(sender, faucet_amount);
						return Ok(())
					},
					Err(_)=> continue
				}
			}

			Err(DispatchError::Other("Out of Faucet"))
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn donate(origin: OriginFor<T>, amount: BalanceOf<T>)-> DispatchResult{
			let sender = ensure_signed(origin)?;

			ensure!(T::Currency::free_balance(&sender) > amount , Error::<T>::NotEnoughBalance );

			let genesis_accounts = Self::genesis_account();

			ensure!( genesis_accounts[0] != sender , Error::<T>::TransferToSelf);

			T::Currency::transfer( &sender, &genesis_accounts[0], amount, ExistenceRequirement::KeepAlive );

			Self::deposit_event(Event::Transfered(sender, genesis_accounts[0].clone(), amount));

			Ok(())
		}

	}
}

impl<T:Config> Pallet<T>{
	fn insert_cache(sender: T::AccountId, faucet_amount: BalanceOf<T>)-> Option<()>{
		match faucet_amount.try_into(){
			Ok(value)=> Some(T::Cache::insert(sender.clone(), sender, value)),
			Err(_)=> None
		}
	}

	fn get_cache(sender: T::AccountId)-> Option<u128>{
		if let Some(faucet_cache) = T::Cache::get(sender.clone(), sender){
			return Some(faucet_cache)
		}
		None
	}
}


#[cfg(feature = "std")]
impl<T:Config> GenesisConfig<T>{
	pub fn build_storage(&self)-> Result<sp_runtime::Storage, String>{
		<Self as frame_support::pallet_prelude::GenesisBuild<T>>::build_storage(self)
	}

	pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage )-> Result<(), String>{
		<Self as frame_support::pallet_prelude::GenesisBuild<T>>::assimilate_storage(self, storage)
	}
}
