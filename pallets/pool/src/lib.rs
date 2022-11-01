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

use frame_support::{pallet_prelude::*, traits::Currency, transactional};
use gafi_primitives::{
	constant::ID,
	custom_services::CustomPool,
	pool::{MasterPool, PoolType, Service},
	system_services::SystemPool,
	ticket::{PlayerTicket, TicketInfo, TicketType},
	whitelist::WhitelistPool
};

use pallet_timestamp::{self as timestamp};

use gafi_primitives::cache::Cache;
use sp_core::H160;
use sp_runtime::traits::StaticLookup;
use sp_std::{str, vec::Vec};


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type UpfrontPool: SystemPool<AccountIdLookupOf<Self>, Self::AccountId>;

		type StakingPool: SystemPool<AccountIdLookupOf<Self>, Self::AccountId>;

		type SponsorePool: CustomPool<Self::AccountId>;

		#[pallet::constant]
		type MaxJoinedSponsoredPool: Get<u32>;

		#[pallet::constant]
		type TimeServiceStorage: Get<u128>;

		type Cache: Cache<Self::AccountId, ID, TicketInfo>;

	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;


	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub type Tickets<T:Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, ID, TicketInfo> ;

	#[pallet::type_value]
	pub fn  DefaultMarkTime<T:Config>()-> u128{
		<timestamp::Pallet<T>>::get().try_into().ok().unwrap()
	}

	#[pallet::storage]
	#[pallet::getter(fn mark_time)]
	pub type MarkTime<T:Config> = StorageValue<_, u128, ValueQuery, DefaultMarkTime<T>>;

	#[pallet::type_value]
	pub fn DefaultTimeService()-> u128{
		3_600_000u128
	}

	#[pallet::storage]
	#[pallet::getter(fn time_service)]
	pub type TimeService<T:Config> = StorageValue<_, u128, ValueQuery, DefaultTimeService>;

	#[pallet::storage]
	pub type Whitelist<T:Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

	// renew tickets, update new marktime
	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T>{
		fn on_finalize(block_number: BlockNumberFor<T>){
			let now:u128 = <timestamp::Pallet<T>>::get().try_into().ok().unwrap();
			if now - Self::mark_time() >= Self::time_service(){
				//Self::renew
				MarkTime::<T>::put(now);
			}
		}
	}

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
			<TimeService<T>>::put(T::TimeServiceStorage::get());
			let now: u128 = <timestamp::Pallet<T>>::get().try_into().ok().unwrap_or_default();
			<MarkTime<T>>::put(now);
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

		Joined{
			sender: T::AccountId,
			pool_id: ID
		},

		Leaved{
			sender: T::AccountId,
			ticket: TicketType,
		},

		LeavedAll{
			sender: T::AccountId,
			pool_type: PoolType,
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		AlreadyJoined,
		NotFoundInPool,
		TicketNotFound,
		ComingSoon,
		ExceedJoinedPool,
		PoolNotFound,
		NotPoolOwner,
		NotWhitelist,
		ParseIdFail
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
		pub fn join(origin: OriginFor<T>, pool_id: ID)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			let sender_lookup = T::Lookup::unlookup(sender.clone());

			ensure!(!Self::is_joined_pool(sender.clone(), pool_id), Error::<T>::AlreadyJoined);
			let ticket_type = Self::get_ticket_type(pool_id)?;

			let pool_match = match ticket_type{
				TicketType::Upfront(_) => T::UpfrontPool::join(sender_lookup, pool_id),
				TicketType::Staking(_)=> T::StakingPool::join(sender_lookup, pool_id),
				TicketType::Sponsored(_)=> {
					let join_sponsored_pool = Tickets::<T>::iter_prefix_values(sender.clone());
					let count_joined_pool = join_sponsored_pool.count();

					ensure!(count_joined_pool <= T::MaxJoinedSponsoredPool::get() as usize , Error::<T>::ExceedJoinedPool );

					T::SponsorePool::join(sender.clone(), pool_id)
				}
			};

			if let Err(err) = pool_match{
				Ok(())
			}else{
				Self::join_pool(&sender, pool_id);
				Self::deposit_event(Event::<T>::Joined {sender, pool_id});
				Ok(())
			}
		}


		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn leave(origin: OriginFor<T>, pool_id: ID)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			let sender_lookup = T::Lookup::unlookup(sender.clone());

			if let Some(ticket) = Self::tickets(sender.clone(), pool_id){
				match ticket.ticket_type{
					TicketType::Upfront(_)=> T::UpfrontPool::leave(sender_lookup)?,
					TicketType::Staking(_) => T::StakingPool::leave(sender_lookup)?,
					TicketType::Sponsored(_)=> T::SponsorePool::leave(sender.clone())?
				}
				T::Cache::insert(sender.clone(), pool_id, ticket);
				Tickets::<T>::remove(sender.clone(), pool_id);
				Self::deposit_event(Event::<T>::Leaved {sender, ticket: ticket.ticket_type});
				Ok(())
			}else{
				Err(Error::<T>::NotFoundInPool.into())
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn leave_all(origin: OriginFor<T>)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			let sender_lookup = T::Lookup::unlookup(sender.clone());
			if T::UpfrontPool::leave(sender_lookup.clone()).is_ok(){
				Self::deposit_event(Event::LeavedAll {sender: sender.clone(), pool_type: PoolType::Upfront});
			}else if T::StakingPool::leave(sender_lookup).is_ok(){
				Self::deposit_event(Event::LeavedAll {sender: sender.clone(), pool_type: PoolType::Staking});
			}else if T::SponsorePool::leave(sender.clone()).is_ok(){
				Self::deposit_event(Event::LeavedAll {sender: sender.clone(), pool_type: PoolType::Sponsored});
			}

			Tickets::<T>::clear_prefix(sender, 6u32, None);

			Ok(())
		}


	}
}

impl<T: Config> Pallet<T> {
	fn create_ticket(sender: T::AccountId, pool_id: ID)-> Result<TicketInfo, Error<T>>{
		let ticket_type = Self::get_ticket_type(pool_id)?;
		if let Some(cache) = Self::get_cache(sender, pool_id){
			return Ok(TicketInfo{
				ticket_type,
				tickets: cache.tickets
			})
		}

		let service = Self::get_ticket_service(ticket_type)?;
		Ok(TicketInfo{
			ticket_type,
			tickets: service.tx_limit
		})

	}

	fn get_cache(sender: T::AccountId, pool_id: ID)-> Option<TicketInfo>{
		if let Some(info) = T::Cache::get(sender, pool_id){
			return Some(info)
		}
		None
	}

	pub fn renew_tickets(){
		let _ = Tickets::<T>::iter().for_each(|player|{
			if let Some(ticket_info) = Self::tickets(player.0.clone(), player.1){
				if let Some(service) = Self::get_service(player.1){
					let new_ticket = ticket_info.renew_ticket(service.tx_limit);
					Tickets::<T>::insert(player.0, player.1, new_ticket);
				}
			}
		});
	}

	fn is_joined_pool(sender: T::AccountId, pool_id: ID)-> bool{
		let joined_pools = Tickets::<T>::iter_prefix_values(sender);
		let mut is_joined = false;

		for joined_ticket in joined_pools{
			match joined_ticket.ticket_type{
				TicketType::Upfront(_) => is_joined = true,
				TicketType::Staking(_) => is_joined = true,
				TicketType::Sponsored(joined_pool_id) => {
					if joined_pool_id == pool_id{
						is_joined = true;
					}
				}
			}
		}

		is_joined
	}

	fn get_ticket_service(ticket: TicketType)-> Result<Service, Error<T>>{
		match ticket{
			TicketType::Upfront(pool_id)=>{
				if let Some(service) = T::UpfrontPool::get_service(pool_id){
					return Ok(service.service)
				}
			},
			TicketType::Staking(pool_id)=>{
				if let Some(service) =T::StakingPool::get_service(pool_id){
					return Ok(service.service)
				}
			},
			TicketType::Sponsored(pool_id)=>{
				if let Some(service) = T::SponsorePool::get_service(pool_id){
					return Ok(service.service)
				}
			}
		}

		Err(Error::<T>::PoolNotFound)
	}

	fn get_ticket_type(pool_id: ID)-> Result<TicketType, Error<T>>{
		if T::UpfrontPool::get_service(pool_id).is_some(){
			return Ok(TicketType::Upfront(pool_id))
		}

		if T::StakingPool::get_service(pool_id).is_some(){
			return Ok(TicketType::Staking(pool_id))
		}

		if T::SponsorePool::get_service(pool_id).is_some(){
			return Ok(TicketType::Sponsored(pool_id))
		}

		Err(Error::<T>::PoolNotFound)
	}
}

impl<T:Config> PlayerTicket<T::AccountId> for Pallet<T>{
	fn use_ticket(player: T::AccountId, target: Option<H160>)-> Option<(TicketType, ID)>{
		let ticket_infos = Tickets::<T>::iter_prefix_values(player.clone());

		for ticket_info in ticket_infos{
			match ticket_info.ticket_type{
				TicketType::Upfront(pool_id) | TicketType::Staking(pool_id)=> {
					if let Some(new_ticket_info) = ticket_info.withdraw_ticket(){
						Tickets::<T>::insert(player, pool_id, new_ticket_info);
						return Some((new_ticket_info.ticket_type, pool_id))
					}
				},
				TicketType::Sponsored(pool_id)=>
					if let Some(contract) = target{
						let targets = Self::get_targets(pool_id);
						if targets.contains(&contract){
							if let Some(new_ticket_info) = ticket_info.withdraw_ticket(){
								Tickets::<T>::insert(player, pool_id, new_ticket_info);
								return Some((new_ticket_info.ticket_type, pool_id))
							}
						}
					}
			}
		}

		None
	}

	fn get_service(pool_id: ID)-> Option<Service>{
		let upfront_service = T::UpfrontPool::get_service(pool_id);
		let staking_service = T::StakingPool::get_service(pool_id);
		let sponsored_service = T::SponsorePool::get_service(pool_id);

		if upfront_service.is_some(){
			return Some(upfront_service.unwrap().service)
		}

		if staking_service.is_some(){
			return Some(staking_service.unwrap().service)
		}

		if sponsored_service.is_some(){
			return Some(sponsored_service.unwrap().service)
		}

		None
	}

	fn get_targets(pool_id: ID)-> Vec<H160>{
		match T::SponsorePool::get_service(pool_id){
			Some(service)=> service.targets,
			None=> [].to_vec()
		}
	}
}


impl<T:Config> WhitelistPool<T::AccountId> for Pallet<T>{
	fn join_pool(sender: &T::AccountId, pool_id: ID)-> Result<(), &'static str> {
		let ticket_info = Self::create_ticket(sender.clone(), pool_id)?;
		Tickets::<T>::insert(sender, pool_id, ticket_info);

		Ok(())
	}

	fn is_joined_pool(sender: T::AccountId, pool_id: ID)-> bool{
		Self::is_joined_pool(sender, pool_id)
	}
}


