use crate::{constant::ID, pool::Service, ticket:: Ticket};
use frame_support::pallet_prelude::*;

use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::{Permill, RuntimeDebug};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct SystemService{
	pub id: ID,
	pub service: Service,
	pub value: u128
}

impl SystemService{
	pub fn new(id: ID, tx_limit: u32, discount: Permill, value: u128)-> Self{
		SystemService{
			id,
			service: Service{tx_limit, discount},
			value
		}
	}
}

pub trait SystemPool<AccountIdLookup, AccountId>{
	fn join(sender: AccountIdLookup, pool_id: ID)-> DispatchResult;
	fn leave(sender: AccountIdLookup)-> DispatchResult;
	fn get_service(pool_id: ID)-> Option<SystemService>;
	fn get_ticket(sender: AccountId)-> Option<Ticket<AccountId>>;
}

impl <AccountIdLookup, AccountId> SystemPool<AccountIdLookup, AccountId> for (){
	fn join(sender: AccountIdLookup, pool_id: ID)-> DispatchResult{
		Ok(())
	}

	fn leave(sender: AccountIdLookup)-> DispatchResult{
		Ok(())
	}

	fn get_service(pool_id: ID)-> Option<SystemService>{
		Default::default()
	}

	fn get_ticket(sender: AccountId)->Option<Ticket<AccountId>>{
		Default::default()
	}
}

pub trait SystemDefaultServices{
	fn get_default_services()-> [(ID, SystemService); 3];
}
