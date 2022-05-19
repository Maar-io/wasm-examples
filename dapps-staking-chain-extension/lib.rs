#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::{Environment};
use ink_lang as ink;


#[ink::chain_extension]
pub trait DappsStakingExt
{
    type ErrorCode = CurrentEraErr;

    #[ink(extension = 2001, returns_result = false)]
    fn read_current_era() -> u32;

    #[ink(extension = 2002, returns_result = false)]
    fn read_era_info(era: u32) -> EraInfo<<ink_env::DefaultEnvironment as Environment>::Balance>;
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

/// A record of rewards allocated for stakers and dapps
#[derive(PartialEq, Eq, Clone, Default, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RewardInfo<Balance> {
    /// Total amount of rewards for stakers in an era
    pub stakers: Balance,
    /// Total amount of rewards for dapps in an era
    pub dapps: Balance,
}

/// A record for total rewards and total amount staked for an era
#[derive(PartialEq, Eq, Clone, Default, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct EraInfo<Balance> {
    /// Total amount of earned rewards for an era
    pub rewards: RewardInfo<Balance>,
    /// Total staked amount in an era
    #[codec(compact)]
    pub staked: Balance,
    /// Total locked amount in an era
    #[codec(compact)]
    pub locked: Balance,
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
    use super::{CurrentEraErr, EraInfo};
    /// Defines the storage of our contract.
    ///
    /// Here we store the random seed fetched from the chain.
    #[ink(storage)]
    pub struct DappsStakingExtension {}

    #[ink(event)]
    pub struct CurrentEraUpdated {
        #[ink(topic)]
        new: u32,
    }

    impl DappsStakingExtension {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            DappsStakingExtension {}
        }

        /// Calls current_era() in the pallet-dapps-staking
        #[ink(message)]
        pub fn read_current_era(&self) -> Result<u32, CurrentEraErr> {
            let era = self.env().extension().read_current_era()?;
            ink_env::debug_println!("read_current_era: {:?}", era);
            self.env().emit_event(CurrentEraUpdated { new: era });
            Ok(era)
        }

        /// Calls current_era() in the pallet-dapps-staking
        #[ink(message)]
        pub fn read_era_info(&self, era:u32) -> Result<u128, CurrentEraErr> {
            ink_env::debug_println!("read_era_info: entered");
            let era_info: EraInfo<Balance> = self.env().extension().read_era_info(era)?;
            ink_env::debug_println!("read_era_info: staked:{:?}", era_info.staked);
            Ok(era_info.staked as u128)
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