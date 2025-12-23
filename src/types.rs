//! Parameters for sonala contracts

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::{EntryLock, EntryType, RecipientSlot, RecipientSlotOwner, RecipientSlotShare, RecipientSlotType};

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
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

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
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
    pub credentials: Vec<u8>,
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
    pub credentials: Vec<u8>,
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

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum BalanceChange {
    Add(u64),
    Sub(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Settle {
    pub player_id: u64,
    pub amount: u64,
    pub change: Option<BalanceChange>,
    pub eject: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Transfer {
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Award {
    pub player_id: u64,
    pub bonus_identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SettleParams {
    pub settles: Box<Vec<Settle>>,
    pub transfer: Option<Transfer>,
    pub awards: Box<Vec<Award>>,
    pub checkpoint: Box<Vec<u8>>,
    pub access_version: u64,
    pub settle_version: u64,
    pub next_settle_version: u64,
    pub entry_lock: Option<EntryLock>,
    pub accept_deposits: Box<Vec<u64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct JoinParams {
    pub amount: u64,
    pub access_version: u64,
    pub settle_version: u64,
    pub position: u16,
    pub verify_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ServeParams {
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

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct CreateRecipientParams {
    pub slots: Box<Vec<RecipientSlotInit>>
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct AssignRecipientParams {
    pub identifier: String
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct AttachBonusParams {
    pub identifiers: Vec<String>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RejectDepositsParams {
    pub reject_deposits: Vec<u64>,
}
