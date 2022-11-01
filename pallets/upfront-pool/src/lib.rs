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
	dispatch::{DispatchResult, Vec},
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency, ReservableCurrency
	}
};

use gafi_primitives::{
	constant::ID,
	pool::MasterPool,
	system_services::{SystemDefaultServices, SystemPool, SystemService},
	ticket::{Ticket, TicketType}
};

use pallet_timestamp::{self as timestamp};
use sp_runtime::traits::StaticLookup;
use gafi_primitives::players::PlayersTime;


#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance ;
	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config + pallet_balances::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type MasterPool: MasterPool<Self::AccountId>;
		#[pallet::constant]
		type MaxPlayerStorage: Get<u32>;

		type UpfrontServices: SystemDefaultServices;

		type Players: PlayersTime<Self::AccountId>;
	}

	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T>{
		fn on_finalize(block_number: BlockNumberFor<T>){
			//let now = Self::get
		}
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;


	#[pallet::storage]
	#[pallet::getter(fn max_player)]
	pub type MaxPlayer<T:Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub type PlayerCount<T:Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub type Tickets<T:Config> = StorageMap<_, Twox64Concat, T::AccountId, Ticket<T::AccountId>> ;

	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub type Services<T:Config> = StorageMap<_, Twox64Concat, ID, SystemService>;

	#[pallet::storage]
	#[pallet::getter(fn new_players)]
	pub type NewPlayers<T:Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxPlayerStorage>, ValueQuery >;

	#[pallet::storage]
	#[pallet::getter(fn ingame_players)]
	pub type IngamePlayers<T:Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxPlayerStorage>, ValueQuery>;


	#[pallet::genesis_config]
	pub struct GenesisConfig{}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig{
		fn default()-> Self{
			Self{}
		}
	}

	#[pallet::genesis_build]
	impl<T:Config> GenesisBuild<T> for GenesisConfig{
		fn build(&self){
			MaxPlayer::<T>::put( T::MaxPlayerStorage::get() );
			for service in T::UpfrontServices::get_default_services(){
				Services::<T>::insert(service.0, service.1);
			}
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

		ChargePoolService,
		UpfrontSetMaxPlayer{
			new_max_player: u32
		},
		UpfrontSetServiceTXLimit{
			service: ID,
			tx_limit: u32
		},
		UpfrontSetServiceDiscount{
			service: ID,
			discount: sp_runtime::Permill
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		PlayerNotFound,
		PlayerCountOverflow,
		ExceedMaxPlayer,
		CanNotClearNewPlayers,
		IntoBalanceFail,
		PoolNotFound
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

	fn get_service(pool_id: ID)-> Option<SystemService>{
		Self::services(pool_id)
	}

	fn get_ticket(sender: T::AccountId)-> Option<Ticket<T::AccountId>>{
		match Self::tickets(sender){
			Some(data)=> Some(data),
			None => None
		}
	}

	fn join(sender: AccountIdLookupOf<T>, pool_id: ID)-> DispatchResult{
		let sender = T::Lookup::lookup(sender)?;
		let new_player_count = Self::player_count().check_add(1).ok_or(Error::<T>::PlayerCountOverflow)?;

		ensure!(new_player_count <= Self::max_player(), Error::<T>::ExceedMaxPlayer);

		{
			//let service = Self::get
		}

		Ok(())
	}


	fn charge_ingame()-> Result<(), Error<T>>{
		let ingame_players = Self::ingame_players();

		for player in ingame_players{
			if let Some(service) = Self::get_player_service(player);

			let fee_value = 0;
			match T::Currency::withdraw(player, fee_value, WithdrawReasons::FEE, ExistenceRequirement::KeepAlive){
				Ok(_)=> {

				},
				Err(_)=> {
					let new_player_count = Self::player_count()
						.checked_sub(1)
						.ok_or(Error::<T>::PlayerCountOverflow);
					let _ = Self::remo
				}

			}
		}

		Ok(())
	}

	fn get_player_service(player: T::AccountId)-> Option<SystemService>{
		if let Some(pool_id) = Self::get_pool_joined(player){
			return Self::get_service(pool_id)
		}
		None
	}

	fn get_pool_joined(player: T::AccountId)-> Option<ID>{
		if let Some(ticket) = Self::tickets(player){
			if let TicketType::Upfront(pool_id) = ticket.ticket_type {
				return Some(pool_id)
			}
		}
		None
	}

	fn moment_to_u128(input: T::Moment)-> u128{
		sp_runtime::SaturatedConversion::saturated_into(input)
	}

	pub fn get_timestamp()-> u128{
		let now = <timestamp::Pallet<T>>::get().try_into().ok().unwrap_or_default() ;
		now
	}

	fn get_pool_by_id(pool_id: ID)-> Result<SystemService, Error<T>>{
		match Self::services(pool_id){
			Some(service)=> Ok(service),
			None => Err(Error::<T>::PoolNotFound)
		}
	}


}
