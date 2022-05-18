#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;


#[ink::chain_extension]
pub trait DappsStakingExt {
    type ErrorCode = CurrentEraErr;

    #[ink(extension = 2001, returns_result = false)]
    fn read_current_era() -> u32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CurrentEraErr {
    FailGetRandomSource,
}

impl ink_env::chain_extension::FromStatusCode for CurrentEraErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailGetRandomSource),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = DappsStakingExt;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod dapp_staking_extension {
    use super::CurrentEraErr;
    /// Defines the storage of our contract.
    ///
    /// Here we store the random seed fetched from the chain.
    #[ink(storage)]
    pub struct DappsStakingExtension {
        /// Stores a single `bool` value on the storage.
        value: u32,
    }

    #[ink(event)]
    pub struct CurrentEraUpdated {
        #[ink(topic)]
        new: u32,
    }

    impl DappsStakingExtension {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: u32) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors may delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Calls current_era() in the pallet-dapps-staking
        #[ink(message)]
        pub fn read_current_era(&mut self) -> Result<(), CurrentEraErr> {
            let era = self.env().extension().read_current_era()?;
            self.value = era;
            ink_env::debug_println!("read_current_era: {:?}", era);
            // Emit the `CurrentEraUpdated` event when the current_era
            // is successfully fetched.
            self.env().emit_event(CurrentEraUpdated { new: era });
            Ok(())
        }

        /// returns last read current era value.
        #[ink(message)]
        pub fn get_current_era(&self) -> u32 {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let ds_extension = DappsStakingExtension::default();
            assert_eq!(ds_extension.get_current_era(), 0);
        }

        #[ink::test]
        fn chain_extension_works() {
            // given
            struct MockedExtension;
            impl ink_env::test::ChainExtension for MockedExtension {
                /// The static function id of the chain extension.
                fn func_id(&self) -> u32 {
                    2001
                }

                /// The chain extension is called with the given input.
                ///
                /// Returns an error code and may fill the `output` buffer with a
                /// SCALE encoded result. The error code is taken from the
                /// `ink_env::chain_extension::FromStatusCode` implementation for
                /// `CurrentEraErr`.
                fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                    let ret: u32 = 1;
                    scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink_env::test::register_chain_extension(MockedExtension);
            let mut ds_extension = DappsStakingExtension::default();
            assert_eq!(ds_extension.get_current_era(), 0);

            // when
            ds_extension.read_current_era().expect("read_current_era must work");

            // then
            assert_eq!(ds_extension.get_current_era(), 1);
        }
    }
}