#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod user_passport {
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;

    /// User Passport storage
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct UserPassport {
        /// User surname
        surname: Vec<u8>,
        /// User name
        name: Vec<u8>,
        /// User birthday as Unix Timestamp
        birthday: u64,
        /// Counter of user assets
        assets: ink_storage::Mapping<AccountId, u32>,
        /// User sercret metadate in BASE64: INN, ...
        metadata: Vec<u8>,
        /// Marks client's smart contract as active
        active: bool,
        // Store a contract owner
        owner: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Event emitted when a caller is not a contract owner
        CallerIsNotAnOwner,
    }

    impl UserPassport {
        /// Constructor that initializes a user passport with the contract as a single asset
        #[ink(constructor)]
        pub fn new(surname: Vec<u8>, name: Vec<u8>, birthday: u64, metadata: Vec<u8>) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.assets.insert(&caller, &1);

                contract.surname = surname;
                contract.name = name;
                contract.birthday = birthday;
                contract.metadata = metadata;
                contract.active = true;
                contract.owner = caller;
            })
        }

        /// Get user name and surname if a caller is a contract owner, else only a surname
        #[ink(message)]
        pub fn get_user_name(&self) -> Vec<u8> {
            if Self::env().caller() == self.owner {
                let mut ret: Vec<u8> = self.surname.to_vec();
                ret.push(b' ');
                ret.extend_from_slice(&self.name.to_vec());
                ret
            } else {
                self.surname.clone()
            }
        }

        /// Check if a contract is active
        #[ink(message)]
        pub fn is_active(&self) -> bool {
            self.active
        }

        /// Deactivate a user contract
        #[ink(message)]
        pub fn deactivate(&mut self) -> Result<(), Error> {
            if Self::env().caller() == self.owner {
                self.active = false;
                Ok(())
            } else {
                Err(Error::CallerIsNotAnOwner)
            }
        }

        /// Get user metadata
        #[ink(message)]
        pub fn get_metadata(&self) -> Result<Vec<u8>, Error> {
            if Self::env().caller() == self.owner {
                Ok(self.metadata.clone())
            } else {
                Err(Error::CallerIsNotAnOwner)
            }
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use crate::user_passport::*;
        use borsh::{BorshDeserialize, BorshSerialize};
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
        pub struct UserMetadata {
            /// User INN number
            inn: u64,
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let metadata = UserMetadata { inn: 3664069397 };

            let metadata_encoded = base64::encode(metadata.try_to_vec().unwrap());
            println!("base64 encoded metadata: {}", &metadata_encoded);

            let mut passport = UserPassport::new(
                "Иванов".into(),
                "Иван".into(),
                503556108,
                metadata_encoded.into(),
            );

            assert_eq!(&passport.get_user_name(), "Иванов Иван".as_bytes());

            let data = passport.get_metadata().unwrap();
            let decoded: Vec<u8> = base64::decode(data).unwrap();
            let decoded = UserMetadata::try_from_slice(&decoded).unwrap();
            assert_eq!(decoded, metadata);

            let the_owner = passport.owner.to_owned();

            let account_id: ink_env::AccountId = [0; 32].into();
            passport.owner = account_id;
            assert_eq!(&passport.get_user_name(), "Иванов".as_bytes());

            let result = passport.deactivate();
            assert_eq!(result, Err(Error::CallerIsNotAnOwner));

            let result = passport.get_metadata();
            assert_eq!(result, Err(Error::CallerIsNotAnOwner));

            passport.owner = the_owner;
            let result = passport.deactivate();
            assert_eq!(result, Ok(()));
            assert_eq!(passport.is_active(), false);
        }
    }
}
