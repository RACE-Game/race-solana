use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError, program_pack::{IsInitialized, Pack, Sealed}, pubkey::Pubkey
};

use crate::{constants::PROFILE_ACCOUNT_LEN, error::ProcessError};

// =======================================================
// ====================== PLAYER ACCOUNT =================
// =======================================================
#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct PlayerState {
    pub is_initialized: bool,
    pub nick: String, // max: 16 chars
    pub pfp: Option<Pubkey>,
}

impl IsInitialized for PlayerState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for PlayerState {}

impl Pack for PlayerState {
    const LEN: usize = PROFILE_ACCOUNT_LEN;

    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(mut src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self::deserialize(&mut src).map_err(|_| ProcessError::RecipientDeserializationFailed)?)
    }
}
