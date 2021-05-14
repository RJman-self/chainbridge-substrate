#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use chainx_primitives::AssetId;
use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::StaticLookup;
use sp_runtime::SaturatedConversion;
use sp_std::vec::Vec;
use xpallet_assets::BalanceOf;

type ResourceId = chainbridge::ResourceId;

mod mock;
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + chainbridge::Config + xpallet_assets::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type RegistorOrigin: EnsureOrigin<Self::Origin>;

        /// Specifies the origin check provided by the bridge for calls that can
        /// only be called by the bridge pallet
        type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidDestChainId,
        ResourceIdAlreadyRegistered,
        ResourceIdNotRegistered,
        ResourceIdCurrencyIdNotMatch,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        RegisterResourceId(ResourceId, AssetId),
        UnregisterResourceId(ResourceId, AssetId),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn resource_ids)]
    pub type ResourceIds<T: Config> = StorageMap<_, Twox64Concat, AssetId, ResourceId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn currency_ids)]
    pub type CurrencyIds<T: Config> = StorageMap<_, Twox64Concat, ResourceId, AssetId, OptionQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000_000)]
        #[transactional]
        pub fn register_resource_id(
            origin: OriginFor<T>,
            resource_id: ResourceId,
            #[pallet::compact] currency_id: AssetId,
        ) -> DispatchResultWithPostInfo {
            T::RegistorOrigin::ensure_origin(origin)?;
            ensure!(
                !ResourceIds::<T>::contains_key(currency_id)
                    && !CurrencyIds::<T>::contains_key(resource_id),
                Error::<T>::ResourceIdAlreadyRegistered,
            );

            ResourceIds::<T>::insert(currency_id, resource_id);
            CurrencyIds::<T>::insert(resource_id, currency_id);
            Self::deposit_event(Event::RegisterResourceId(resource_id, currency_id));
            Ok(().into())
        }

        #[pallet::weight(1_000_000)]
        #[transactional]
        pub fn remove_resource_id(
            origin: OriginFor<T>,
            resource_id: ResourceId,
        ) -> DispatchResultWithPostInfo {
            T::RegistorOrigin::ensure_origin(origin)?;
            if let Some(currency_id) = CurrencyIds::<T>::take(resource_id) {
                ResourceIds::<T>::remove(currency_id);
                Self::deposit_event(Event::UnregisterResourceId(resource_id, currency_id));
            }
            Ok(().into())
        }

        #[pallet::weight(1_000_000)]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            #[pallet::compact] currency_id: AssetId,
            dest_chain_id: chainbridge::ChainId,
            recipient: Vec<u8>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::destroy(who, currency_id, dest_chain_id, recipient, amount)?;
            Ok(().into())
        }

        #[pallet::weight(1_000_000)]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            to: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: BalanceOf<T>,
            resource_id: ResourceId,
        ) -> DispatchResultWithPostInfo {
            T::BridgeOrigin::ensure_origin(origin)?;

            let currency_id =
                Self::currency_ids(resource_id).ok_or(Error::<T>::ResourceIdNotRegistered)?;

            let to = T::Lookup::lookup(to)?;
            xpallet_assets::Module::<T>::issue(&currency_id, &to, amount)?;

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn destroy(
        from: T::AccountId,
        currency_id: AssetId,
        dest_chain_id: chainbridge::ChainId,
        recipient: Vec<u8>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        ensure!(
            chainbridge::Module::<T>::chain_whitelisted(dest_chain_id),
            Error::<T>::InvalidDestChainId
        );

        let resource_id =
            Self::resource_ids(currency_id).ok_or(Error::<T>::ResourceIdNotRegistered)?;

        xpallet_assets::Module::<T>::destroy_usable(&currency_id, &from, amount)?;

        chainbridge::Module::<T>::transfer_fungible(
            dest_chain_id,
            resource_id,
            recipient,
            sp_core::U256::from(amount.saturated_into::<u128>()),
        )
    }
}
