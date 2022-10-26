use crate::constant::ID;

pub trait WhitelistPool<AccountId>{
	fn join_pool(sender: AccountId, pool_id: ID)-> Result<(), &'static str>;
	fn is_joined_pool(sender: AccountId, pool_id: ID)-> bool;
}

impl<AccountId> WhitelistPool<AccountId> for (){
	fn join_pool( sender: AccountId, pool_id: ID )-> Result<(), &'static str>{
		Err("default")
	}
	fn is_joined_pool(sender: AccountId, pool_id: ID)-> bool{
		false
	}
}

pub trait IWhiteList<AccountId>{
	fn is_whitelist(pool_id: ID)-> bool;
	fn insert_whitelist(pool_id: ID, player: AccountId)-> Result<(), &'static str>;
}

impl <AccountId> IWhiteList<AccountId> for (){
	fn is_whitelist(pool_id: ID)-> bool{
		false
	}

	fn insert_whitelist(pool_id: ID, player: AccountId)-> Result<(), &'static str>{
		Err("default")
	}
}
