#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use frame_support::dispatch;
use frame_support::debug;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::AtLeast32BitUnsigned;
	use sp_std::fmt::Debug;
	use codec::Codec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy +
		MaybeSerializeDeserialize + Debug + From<u128> + Into<u128>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// 定义总供给量
	#[pallet::storage]
	#[pallet::getter(fn total_supply_by)]
	pub type TotalSupply<T: Config> = StorageValue<_, T::Balance>;

	// 定义一个账户对应的余额
	#[pallet::storage]
	#[pallet::getter(fn balances_by)]
	pub type Balances<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery >;

	// 定义A账户可以向B账户转账的金额
	#[pallet::storage]
	#[pallet::getter(fn allowance_by)]
	pub type Allowance<T: Config> = StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), T::Balance>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Transfer(T::AccountId, T::AccountId, T::Balance),
		Approve(T::AccountId, T::AccountId, T::Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// in sufficient balance
		InSufficientBalance,
		/// in sufficient allowance
		InSufficientAllowance,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_1000 + T::DbWeight::get().writes(1))]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, value: T::Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::transfer_help(who, to, value)?;

			Ok(().into())
		}

		#[pallet::weight(10_1000 + T::DbWeight::get().writes(1))]
		pub fn transfer_from(origin: OriginFor<T>, from: T::AccountId, to: T::AccountId, value: T::Balance) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			//
			// let from_balance = Balances::<T>::get(&from);
			//
			// if from_balance < value {
			// 	return Err(Error::<T>::InSufficientAllowance)?;
			// }
			//
			// Allowance::<T>::insert((from.clone(), to.clone()), from_balance - value);
			//
			// Self::transfer_help(from, to, value)?;
			Self::do_transfer_from(from, to, value)?;

			Ok(().into())
		}

		#[pallet::weight(10_1000 + T::DbWeight::get().writes(1))]
		pub fn allowance(origin: OriginFor<T>, spender: T::AccountId, value: T::Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// let who_balance = Balances::<T>::get(&who);
			// if who_balance < value {
			// 	return Err(Error::<T>::InSufficientBalance)?;
			// }
			//
			// Allowance::<T>::insert((who.clone(), spender.clone()), value);
			//
			// Self::deposit_event(Event::Approve(who, spender, value));

			Self::do_allowance(who, spender, value)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {

	pub fn create(owner: T::AccountId, total_supply: T::Balance) -> dispatch::DispatchResult {

		//set initial supply
		TotalSupply::<T>::put(total_supply);
		// set alias who and total supply
		Balances::<T>::insert(owner, total_supply);

		Ok(().into())
	}

	pub fn do_transfer_from(from: T::AccountId, to: T::AccountId, value: T::Balance) -> dispatch::DispatchResult {

		let from_balance = Balances::<T>::get(&from);

		if from_balance < value {
			return Err(Error::<T>::InSufficientAllowance)?;
		}

		Allowance::<T>::insert((from.clone(), to.clone()), from_balance - value);

		Self::transfer_help(from, to, value)?;

		Ok(().into())
	}

	pub fn do_allowance(who: T::AccountId, spender: T::AccountId, value: T::Balance) -> dispatch::DispatchResult {

		let who_balance = Balances::<T>::get(&who);
		if who_balance < value {
			return Err(Error::<T>::InSufficientBalance)?;
		}

		Allowance::<T>::insert((who.clone(), spender.clone()), value);

		Self::deposit_event(Event::Approve(who, spender, value));

		Ok(().into())
	}

	pub fn transfer_help(from: T::AccountId, to : T::AccountId, value: T::Balance) -> dispatch::DispatchResult {
		let from_balance = Balances::<T>::get(&from);
		if from_balance < value {
			return Err(Error::<T>::InSufficientBalance)?;
		}

		Balances::<T>::insert(from.clone(), from_balance - value);

		let to_balance = Balances::<T>::get(&to);

		Balances::<T>::insert(to.clone(), to_balance + value);

		Self::deposit_event(Event::Transfer(from, to, value));

		Ok(())
	}

	pub fn total_supply() -> <T as pallet::Config>::Balance {
		debug::info!("erc20 total supply of!");

		// Self::total_supply_by()
		TotalSupply::<T>::get().unwrap()
	}

	pub fn balance_of(owner: T::AccountId) -> T::Balance {
		debug::info!("erc20 balance_of!");
		debug::info!("owner is {:?}", owner);

		// Self::balance_by(owner)
		Balances::<T>::get(&owner)
	}

	pub fn allowance_of(owner: T::AccountId, spender: T::AccountId) -> T::Balance {
		debug::info!("erc20 allowance of!");
		debug::info!("owner: {:?}, spender: {:?}", owner, spender);

		// Self::alowance_by(owner, spender)
		Allowance::<T>::get(&(owner, spender)).unwrap()
	}
}