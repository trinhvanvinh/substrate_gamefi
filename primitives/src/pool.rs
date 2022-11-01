use frame_support::pallet_prelude::*;
#[cfg(feature="std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::{Permill, RuntimeDebug};
use sp_std::vec::Vec;

use crate::constant::ID;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum PoolType{
	Upfront,
	Staking,
	Sponsored
}
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Service{
	pub tx_limit: u32,
	pub discount: Permill
}

pub trait MasterPool<AccountId>{
	fn remove_player(player: AccountId, pool_id: ID);
	fn get_timeservice()-> u128;
	fn get_marktime()-> u128;
}

impl<AccountId> MasterPool<AccountId> for (){
	fn remove_player(_player: AccountId, _pool_id: ID){}
	fn get_timeservice()-> u128{
		30*60_000 // 30minutes
	}
	fn get_marktime()-> u128{
		u128::default()
	}
}

pub enum SponsoredPoolJoinType{
	Default,
	Whitelist
}

pub trait SponsoredPoolJoinTypeHandle<AccountId>{
	fn set_join_type(pool_id: ID, join_type: SponsoredPoolJoinType, call_check_url: Vec<u8>, account_id: AccountId)-> DispatchResult;
	fn reset(pool_id: ID, account_id: AccountId)-> DispatchResult;
	fn get_join_type(pool_id: ID)-> Option<(SponsoredPoolJoinType, Vec<u8>)>;
}

pub trait GetSponsorePoolJoinType{
	fn get_join_type(pool_id: ID)-> (SponsoredPoolJoinType, Vec<u8>);
}
