#![cfg_attr(not(feature = "std"), no_std)]

use scale::{Encode, Decode, HasCompact};
use ink_env::Environment;
use ink_lang as ink;

#[ink::chain_extension]
pub trait DappsStakingExt {
    type ErrorCode = DSErrorCode;

    #[ink(extension = 2001, returns_result = false)]
    fn read_current_era() -> u32;

    #[ink(extension = 2002)]
    fn read_era_info(
        era: u32,
    ) -> Result<EraInfo<<ink_env::DefaultEnvironment as Environment>::Balance>, DSError>;
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DSErrorCode {
    Failed,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DSError {
    ErrorCode(DSErrorCode),
}

impl From<DSErrorCode> for DSError {
    fn from(error_code: DSErrorCode) -> Self {
        Self::ErrorCode(error_code)
    }
}

impl From<scale::Error> for DSError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink_env::chain_extension::FromStatusCode for DSErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Failed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

/// A record of rewards allocated for stakers and dapps
#[derive(PartialEq, Debug, Eq, Clone, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RewardInfo<Balance: HasCompact> {
    /// Total amount of rewards for stakers in an era
    #[codec(compact)]
    pub stakers: Balance,
    /// Total amount of rewards for dapps in an era
    #[codec(compact)]
    pub dapps: Balance,
}

/// A record for total rewards and total amount staked for an era
#[derive(PartialEq, Debug, Eq, Clone, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct EraInfo<Balance: HasCompact> {
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
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = DappsStakingExt;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod dapp_staking_extension {
    use super::{DSError, EraInfo};

    #[ink(storage)]
    pub struct DappsStakingExtension {}

    #[ink(event)]
    pub struct CurrentEraUpdated {
        #[ink(topic)]
        new: u32,
    }

    impl DappsStakingExtension {
        #[ink(constructor)]
        pub fn new() -> Self {
            DappsStakingExtension {}
        }

        /// Calls current_era() in the pallet-dapps-staking
        #[ink(message)]
        pub fn read_current_era(&self) -> Result<u32, DSError> {
            let era = self.env().extension().read_current_era()?;
            ink_env::debug_println!("read_current_era: {:?}", era);
            self.env().emit_event(CurrentEraUpdated { new: era });
            Ok(era)
        }

        /// Calls general_era_info() in the pallet-dapps-staking
        #[ink(message)]
        pub fn read_era_info(&self, era: u32) -> Result<EraInfo<Balance>, DSError> {
            ink_env::debug_println!("read_era_info: entered");
            self.env().extension().read_era_info(era)

            // let era_info_result: Result<EraInfo<Balance>, _> = self.env().extension().read_era_info(era);
            // let era_info = match era_info_result{
            //     Ok(info)  => info,
            //     Err(e) => return Err(e),
            // };
            // ink_env::debug_println!("read_era_info: staked:{:?}", era_info.rewards.stakers);
            // Ok(era_info)
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
                /// `DappsStakingResponseError`.
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
            ds_extension
                .read_current_era()
                .expect("read_current_era must work");

            // then
            assert_eq!(ds_extension.get_current_era(), 1);
        }
    }
}
