//! Parameters for sonala contracts

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::{EntryType, RecipientSlotOwner, RecipientSlotType, RecipientSlot, RecipientSlotShare};

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RecipientSlotShareInit {
    pub owner: RecipientSlotOwner,
    pub weights: u16,
}

impl From<RecipientSlotShareInit> for RecipientSlotShare {
    fn from(value: RecipientSlotShareInit) -> Self {
        let RecipientSlotShareInit { owner, weights } = value;
        Self {
            owner,
            weights,
            claim_amount: 0,
        }
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RecipientSlotInit {
    pub id: u8,
    pub slot_type: RecipientSlotType,
    pub token_addr: Pubkey,
    pub stake_addr: Pubkey,
    pub init_shares: Vec<RecipientSlotShareInit>,
}

impl From<RecipientSlotInit> for RecipientSlot {
    fn from(value: RecipientSlotInit) -> Self {
        let RecipientSlotInit { id, slot_type, token_addr, stake_addr, init_shares } = value;
        let shares = init_shares.into_iter().map(Into::into).collect();
        Self {
            id,
            slot_type,
            token_addr,
            stake_addr,
            shares,
        }
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct TokenInfo {
    name: String,
    addr: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CreateGameAccountParams {
    pub title: String,
    pub max_players: u16,
    pub entry_type: EntryType,
    pub data: Vec<u8>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct RegisterServerParams {
    pub endpoint: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct UnregisterTransactorParams {
    pub addr: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CreateRegistrationParams {
    pub is_private: bool,
    pub size: u16,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RegisterGameParams {
    pub game_addr: String,
    pub reg_addr: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct UnregisterGameParams {
    pub game_addr: String,
    pub reg_addr: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GetTransactorInfoParams {
    pub addr: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CreatePlayerProfileParams {
    pub nick: String,
}

/// The player status in settlement.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum PlayerStatus {
    Normal,
    Left,
    Dropout,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum AssetChange {
    Add,
    Sub,
    NoChange,
}

/// The data represents how a player's asset & status changed.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum SettleOp {
    Add(u64),
    Sub(u64),
    Eject,
    AssignSlot(String),
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Settle {
    pub addr: Pubkey,
    pub op: SettleOp,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Transfer {
    pub slot_id: u8,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SettleParams {
    pub settles: Vec<Settle>,
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct JoinParams {
    pub amount: u64,
    pub access_version: u64,
    pub position: u16,
    pub verify_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ServeParams {
    pub verify_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct DepositParams {
    pub amount: u64,
    pub settle_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum VoteType {
    ServerVoteTransactorDropOff,
    ClientVoteTransactorDropOff,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct VoteParams {
    pub vote_type: VoteType,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct PublishParams {
    // Arweave IDX pointing to bundled game data
    pub uri: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CreateRecipientParams {
    pub slots: Vec<RecipientSlotInit>
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct AssignRecipientParams {
    pub identifier: String
}
