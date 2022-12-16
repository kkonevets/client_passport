#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod user_passport {
    use ink_storage::traits::SpreadAllocate;

    /// User Passport storage
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct UserPassport {
        /// User surname
        surname: String,
        /// User name
        name: String,
        /// User birthday as Unix Timestamp
        birthday: u64,
        /// Counter of user assets
        assets: ink_storage::Mapping<AccountId, u32>,
        /// User sercret metadate: INN, ...
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
        /// Constructor that initializes a user password with empty assets
        #[ink(constructor)]
        pub fn new(surname: String, name: String, birthday: u64, metadata: Vec<u8>) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.surname = surname;
                contract.name = name;
                contract.birthday = birthday;
                contract.metadata = metadata;
                contract.active = true;
                contract.owner = Self::env().caller();
                // assets are empty initialized
            })
        }

        /// Get user name and surname if a caller is a contract owner, else only a name
        #[ink(message)]
        pub fn get_user_name(&self) -> String {
            if Self::env().caller() == self.owner {
                format!("{} {}", self.surname, &self.name)
            } else {
                self.surname.to_owned()
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
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use crate::user_passport::{Error, UserPassport};

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut passport = UserPassport::new(
                "Иванов".to_owned(),
                "Иван".to_owned(),
                503556108,
                vec![0, 0, 0],
            );
            assert_eq!(passport.get_user_name(), "Иванов Иван");

            let the_owner = passport.owner.to_owned();

            let array = [0; 32];
            let account_id: ink_env::AccountId = array.into();
            passport.owner = account_id;
            assert_eq!(passport.get_user_name(), "Иванов");

            let result = passport.deactivate();
            assert_eq!(result, Err(Error::CallerIsNotAnOwner));

            passport.owner = the_owner;
            let result = passport.deactivate();
            assert_eq!(result, Ok(()));
            assert_eq!(passport.is_active(), false);
        }
    }
}
