use crate::{constant::ID, pool::Service};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{Permill, RuntimeDebug};
use sp_std::vec::Vec;

pub struct CustomService<AccountId>{
	pub service: Service,
	pub sponsor: AccountId,
	pub targets: Vec<H160>
}

impl<AccountId> CustomService<AccountId>{
	pub fn new(targets: Vec<H160>, tx_limit: u32, discount: Permill, sponsor: AccountId)-> Self{
		CustomService{
			service: Service{tx_limit, discount},
			sponsor: sponsor,
			targets: targets
		}
	}

}

pub trait CustomPool<AccountId>{
	fn join(sender: AccountId, pool_id: ID)-> DispatchResult;
	fn leave(sender: AccountId)-> DispatchResult;

	fn is_pool(pool_id: ID)-> bool;
	fn get_service(pool_id: ID)-> Option<CustomService<AccountId>>;
	fn get_pool_owner(pool_id: ID)-> Option<AccountId>;

	fn add_default(owner: AccountId, pool_id: ID);
}
