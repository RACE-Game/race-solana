use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError, program_pack::{IsInitialized, Pack, Sealed}, pubkey::Pubkey
};

use crate::{constants::PROFILE_ACCOUNT_LEN, error::ProcessError};

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct LegacyPlayerState {
    pub is_initialized: bool,
    pub nick: String, // max: 16 chars
    pub pfp: Option<Pubkey>,
}

impl IsInitialized for LegacyPlayerState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for LegacyPlayerState {}

impl Pack for LegacyPlayerState {
    const LEN: usize = PROFILE_ACCOUNT_LEN;

    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(mut src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self::deserialize(&mut src).map_err(|_| ProcessError::RecipientDeserializationFailed)?)
    }
}

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct PlayerState {
    pub version: u8, // should always be PROFILE_VERSION(2)
    pub nick: String,
    pub pfp: Option<Pubkey>,
    pub credentials: Vec<u8>,
}

impl IsInitialized for PlayerState {
    fn is_initialized(&self) -> bool {
        self.version > 0
    }
}
