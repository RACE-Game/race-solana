use crate::{constants::RECIPIENT_ACCOUNT_LEN, error::ProcessError};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum RecipientSlotType {
    #[default]
    Token,
    Nft,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum RecipientSlotOwner {
    Unassigned { identifier: String },
    Assigned { addr: Pubkey },
}

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RecipientSlotShare {
    pub owner: RecipientSlotOwner,
    pub weights: u16,
    pub claim_amount: u64,
}

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RecipientSlot {
    pub id: u8,
    pub slot_type: RecipientSlotType,
    pub token_addr: Pubkey,
    pub stake_addr: Pubkey,
    pub shares: Vec<RecipientSlotShare>,
}

// State of on-chain RecipientAccount
#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RecipientState {
    pub is_initialized: bool,
    pub cap_addr: Option<Pubkey>,
    pub slots: Vec<RecipientSlot>,
}

impl IsInitialized for RecipientState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for RecipientState {}

impl Pack for RecipientState {
    const LEN: usize = RECIPIENT_ACCOUNT_LEN;

    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(mut src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self::deserialize(&mut src).map_err(|_| ProcessError::RecipientDeserializationFailed)?)
    }
}
