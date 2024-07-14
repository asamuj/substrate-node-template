#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ReservableCurrency};
pub use pallet::*;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;

		/// Reservation fee.
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;
	}

	/// Error for the Nicks pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShort,
		/// A name is too long.
		TooLong,
		/// An account isn't named.
		Unnamed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A name was set.
		NameSet {
			/// The account for which the name was set.
			who: T::AccountId,
		},
		/// A name was changed.
		NameChanged {
			/// The account for which the name was changed.
			who: T::AccountId,
		},
		NameQueried {
			who: T::AccountId,
			name: BoundedVec<u8, T::MaxLength>,
		},
	}

	/// The lookup table for names.
	#[pallet::storage]
	pub(super) type NameOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (BoundedVec<u8, T::MaxLength>, BalanceOf<T>)>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({50_000_000})]
		pub fn set_name(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let bounded_name: BoundedVec<u8, T::MaxLength> =
				name.try_into().map_err(|_| Error::<T>::TooLong)?;

			let deposit = if let Some((_, deposit)) = <NameOf<T>>::get(sender.clone()) {
				Self::deposit_event(Event::<T>::NameChanged { who: sender.clone() });
				deposit
			} else {
				let deposit = T::ReservationFee::get();
				T::Currency::reserve(&sender, deposit)?;
				Self::deposit_event(Event::<T>::NameSet { who: sender.clone() });
				deposit
			};

			<NameOf<T>>::insert(&sender, (bounded_name, deposit));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		pub fn get_name(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// 通过sender从存储中获取名字
			let (name, _) = <NameOf<T>>::get(&sender).ok_or(Error::<T>::Unnamed)?;

			// 触发一个事件，通知查询成功
			Self::deposit_event(Event::<T>::NameQueried { who: sender, name });

			Ok(())
		}
	}
}
