#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::{AccountId, Environment};
use ink_lang as ink;
use ink_prelude::vec::Vec;

use scale::{Decode, Encode};
// use rmrk_chain_extension_types::RmrkFunc;
// const RMRK_EXTENSION: u32 = 3500;

#[ink::chain_extension]
pub trait RmrkExtension {
    type ErrorCode = RmrkErrorCode;

    #[ink(extension = 3501)]
    fn next_nft_id(collection_id: u32) -> Result<u32, RmrkError>;

    #[ink(extension = 3502)]
    fn collection_index() -> Result<u32, RmrkError>;

    #[ink(extension = 3513)]
    fn mint_ntf(
        beneficiary: AccountId,
        collection_id: u32,
        royalty_recipient: Option<AccountId>,
        royalty: Option<u8>,
        metadata: Vec<u8>,
        transferable: bool,
        resources: Option< ((Vec<u8>, Vec<u8>), u32) >,
    ) -> Result<(), RmrkError>;

    #[ink(extension = 3515)]
    fn create_collection(
        metadata: Vec<u8>,
        max: Option<u32>,
        symbol: Vec<u8>,
    ) -> Result<(), RmrkError>;
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RmrkErrorCode {
    Failed,
}

#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
pub enum RmrkError {
    ErrorCode(RmrkErrorCode),
}

impl From<RmrkErrorCode> for RmrkError {
    fn from(error_code: RmrkErrorCode) -> Self {
        Self::ErrorCode(error_code)
    }
}

impl From<scale::Error> for RmrkError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink_env::chain_extension::FromStatusCode for RmrkErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Failed),
            _ => panic!("encountered unknown status code"),
        }
    }
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

    type ChainExtension = RmrkExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod rmrk_chain_test {

    use super::RmrkError;
    use ink_prelude::vec::Vec;

    #[ink(storage)]
    pub struct Rmrk {}

    impl Rmrk {

        #[ink(constructor)]
        pub fn new() -> Self {
            Rmrk {}
        }

        #[ink(message)]
        pub fn next_nft_id(&self, collection_id: u32) -> Result<u32, RmrkError> {
            let nft_id = self.env().extension().next_nft_id(collection_id)?;
            ink_env::debug_println!("collection_index: {:?}", nft_id);
            Ok(nft_id)
        }

        #[ink(message)]
        pub fn collection_index(&self) -> Result<u32, RmrkError> {
            let collection_id = self.env().extension().collection_index()?;
            ink_env::debug_println!("collection_index: {:?}", collection_id);
            Ok(collection_id)
        }

        #[ink(message)]
        pub fn mint_ntf(&mut self,
            beneficiary: AccountId,
            collection_id: u32,
            royalty_recipient: Option<AccountId>,
            _royalty: Option<u8>,
            metadata: Vec<u8>,
            transferable: bool,
            resources: Option< ((Vec<u8>, Vec<u8>), u32) >,
        ) -> Result<(), RmrkError>{
            let _result = self
            .env()
            .extension()
            .mint_ntf(
                beneficiary,
                collection_id,
                royalty_recipient,
                None, // fix to use Permill
                metadata,
                transferable,
                resources,
            )?;
            Ok(())
        }

        #[ink(message)]
        pub fn create_collection(
            &mut self,
            metadata: Vec<u8>,
            max: Option<u32>,
            symbol: Vec<u8>,
        ) -> Result<(), RmrkError> {
            let _result = self
                .env()
                .extension()
                .create_collection(metadata, max, symbol)?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn create_collection_works() {
            struct MockedExtension;
            impl ink_env::test::ChainExtension for MockedExtension {
                /// The static function id of the chain extension.
                fn func_id(&self) -> u32 {
                    3515
                }

                fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                    let ret: [u8; 32] = [1; 32];
                    scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink_env::test::register_chain_extension(MockedExtension);
            let rmrk = Rmrk::new();
            let metadata = "ipfs://ipfs/QmTG9ekqrdMh3dsehLYjC19fUSmPR31Ds2h6Jd7LnMZ9c7".to_string();
            let symbol = "ROO".to_string();

            // Get contract address.
            // let callee = ink_env::account_id::<ink_env::DefaultEnvironment>();

            let _result = rmrk.create_collection(
                metadata.into_bytes(),
                None,
                symbol.clone().into_bytes(),
            );
        }
    }
}
