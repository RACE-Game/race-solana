use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::IsInitialized,
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
