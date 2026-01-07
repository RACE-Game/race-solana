use crate::{constants::SERVER_ACCOUNT_LEN, error::ProcessError};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct LegacyServerState {
    pub is_initialized: bool,
    pub addr: Pubkey,
    pub owner: Pubkey,
    pub endpoint: String, // max: 50 chars
}

impl IsInitialized for LegacyServerState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for LegacyServerState {}

impl Pack for LegacyServerState {
    const LEN: usize = SERVER_ACCOUNT_LEN;

    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(mut src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self::deserialize(&mut src).map_err(|_| ProcessError::ServerDeserializationFailed)?)
    }
}

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct ServerState {
    pub version: u8,
    pub addr: Pubkey,
    pub owner: Pubkey,
    pub endpoint: String,
    pub credentials: Vec<u8>,
}

impl IsInitialized for ServerState {
    fn is_initialized(&self) -> bool {
        self.version > 0
    }
}
