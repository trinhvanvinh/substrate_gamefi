#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
use gafi_primitives::constant::ID;
use sp_runtime::DispatchError;
use codec::{Decode};

pub enum ConvertError{
	TryIntoBalanceFail
}

pub type BalanceOf<C,A> = <C as Currency<A>>::Balance;

pub fn u128_try_to_balance<C, A>(input: u128)-> Result<BalanceOf<C, A>, DispatchError> where C: Currency<A>{
	match input.try_into().ok(){
		Some(val)=> Ok(val),
		None=> Err(DispatchError::Other("Can not convert u128 to balance"))
	}
}

pub fn u128_to_balance<C, A>(input: u128)-> BalanceOf<C, A> where C: Currency<A>{
	input.try_into().ok().unwrap_or_default()
}

pub fn balance_try_to_u128<C, A>(input: BalanceOf<C, A>) -> Result<u128, DispatchError> where C: Currency<A>{
	match input.try_into().ok(){
		Some(val)=> Ok(val),
		None=> Err(DispatchError::Other("Can not convert balance to u128"))
	}
}

pub fn into_account<AccountId>(id: ID)-> Option<AccountId> where AccountId: Decode{
	match AccountId::decode(&mut &id[..]){
		Ok(account)=> Some(account),
		Err(_)=> None
	}
}
