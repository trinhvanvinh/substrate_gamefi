use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_std::prelude::*;

pub struct Token {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub id: u8,
}

pub enum NativeToken {
	GAFI,
	GAKI,
}

pub trait TokenInfo {
	fn token_info(token: NativeToken) -> Token;
}

pub type Balance = u128;

pub struct GafiCurrency {}

impl TokenInfo for GafiCurrency {
	fn token_info(token: NativeToken) -> Token {
		let gafi: Token = Token { name: b"GAFI TOKEN".to_vec(), symbol: b"GAFI".to_vec(), decimals: 18, id: 1 };

		let gaki: Token = Token { name: b"GAKI TOKEN".to_vec(), symbol: b"GAKI".to_vec(), decimals: 18, id: 2 };

		match token {
			NativeToken::GAFI => gafi,
			NativeToken::GAKI => gaki,
		}
	}
}

pub fn unit(token: NativeToken) -> u128 {
	GafiCurrency::token_info(token).decimals as u128
}
