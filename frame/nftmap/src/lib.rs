//! ## Genesis Config
//!
//! The  pallet depends on the [`GenesisConfig`].

// All pallets must be configured for `no_std`.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, storage::types::StorageMap,dispatch::Vec};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NFTAdded { who: T::AccountId, val: u32 },
        UpdatededNFT{ who: T::AccountId, val: u32 },
        RemovedNFT{who: T::AccountId},
    }

    #[pallet::error]
    pub enum Error<T> {
        AlreadyAdded,
        NotPresent
    }

    #[pallet::storage]
    #[pallet::getter(fn nfts)]
    pub type NFTs<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;



    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T:Config> {
        pub nft_mappers: Vec<(T::AccountId, u32)>,
    }



    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (account, value) in &self.nft_mappers {
                NFTs::<T>::insert(account, *value);
            }
        }
    }




// pub(super) type Claims<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, BlockNumberFor<T>)>;

// Dispatchable functions allow users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
#[pallet::call]
impl<T: Config> Pallet<T> {

 #[pallet::weight(Weight::default())]
 #[pallet::call_index(0)]
 pub fn add_nft(origin: OriginFor<T>, account: T::AccountId, val: u32) -> DispatchResult {
   // Check that the extrinsic was signed and get the signer.
   // This function will return an error if the extrclaiminsic is not signed.
   let _sender = ensure_root(origin)?;

   // Verify that the specified account has not already been stored.
   ensure!(!NFTs::<T>::contains_key(&account), Error::<T>::AlreadyAdded);
  

   // Store the nft with the address and number.
   NFTs::<T>::insert(&account, val);

   // Emit an event that the nft was created.
   Self::deposit_event(Event:: NFTAdded { who: account, val: val });

   Ok(())
 }


 #[pallet::weight(Weight::default())]
 #[pallet::call_index(1)]
 pub fn update_nft(origin: OriginFor<T>, account: T::AccountId, val: u32) -> DispatchResult {
   // Check that the extrinsic was signed and get the signer.
   // This function will return an error if the extrclaiminsic is not signed.
   let _sender = ensure_root(origin)?;

   // Verify that the specified account has not already been stored.
   ensure!(NFTs::<T>::contains_key(&account), Error::<T>::NotPresent);
  

   // Store the nft with the address and number.
   NFTs::<T>::insert(&account, val);

   // Emit an event that the nft was created.
   Self::deposit_event(Event:: UpdatededNFT{ who: account, val: val });

   Ok(())
 }

 #[pallet::weight(Weight::default())]
 #[pallet::call_index(2)]
 pub fn delete_nft(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
   // Check that the extrinsic was signed and get the signer.
   // This function will return an error if the extrclaiminsic is not signed.
   let _sender = ensure_root(origin)?;

   // Verify that the specified account has not already been stored.
   ensure!(NFTs::<T>::contains_key(&account), Error::<T>::NotPresent);
  

   // Store the nft with the address and number.
   NFTs::<T>::remove(&account);

   // Emit an event that the nft was created.
   Self::deposit_event(Event:: RemovedNFT{ who: account});

   Ok(())
 }




}

}

pub mod weights {
  // Placeholder struct for the pallet weights
  pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
}
