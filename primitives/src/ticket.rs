use crate::{constant::ID, pool::Service};
use frame_support::pallet_prelude::*;
#[cfg(feature="std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

pub struct Ticket<AccountId>{
	pub address: AccountId,
	pub join_time: u128,
	pub ticket_type: TicketType
}
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq,PartialEq, Copy, Clone, Encode, Decode )]
pub enum TicketType{
	Upfront(ID),
	Staking(ID),
	Sponsored(ID)
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq,PartialEq, Copy, Clone, Encode, Decode )]
pub struct TicketInfo{
	pub ticket_type: TicketType,
	pub tickets: u32
}

impl TicketInfo{
	pub fn withdraw_ticket(&self)-> Option<Self>{
		if let Some(new_tickets) = self.tickets.checked_sub(1){
			return Some(TicketInfo{
				ticket_type: self.ticket_type,
				tickets: new_tickets
			});
		}
		None
	}
	pub fn renew_ticket(&self, new_remain: u32)-> Self{
		TicketInfo {
			tickets: new_remain,
			ticket_type: self.ticket_type
		}
	}
}

pub trait PlayerTicket<AccountId>{
	fn use_ticket(player: AccountId, target: Option<H160>)-> Option<(TicketType, ID)>;
	fn get_service(pool_id: ID)-> Option<Service>;
	fn get_targets(pool_id: ID)-> Vec<H160>;
}

impl<AccountId> PlayerTicket<AccountId> for(){
	fn use_ticket(_player: AccountId, _target: Option<H160>)-> Option<(TicketType, ID)>{
		None
	}
	fn get_service(_pool_id: ID)-> Option<Service>{
		None
	}
	fn get_targets(_pool_id: ID)-> Vec<H160>{
		[].to_vec()
	}
}


