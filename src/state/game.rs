use crate::types::VoteType;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::IsInitialized,
    pubkey::Pubkey,
};

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub enum EntryType {
    Cash {
        min_deposit: u64,
        max_deposit: u64,
    },
    Ticket {
        slot_id: u8,
        amount: u64,
    },
    Gating {
        collection: String,
    }
}

impl Default for EntryType {
    fn default() -> Self {
        EntryType::Cash {
            min_deposit: 1,
            max_deposit: 9999,
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct PlayerJoin {
    pub addr: Pubkey,
    pub balance: u64,
    pub position: u16,
    pub access_version: u64,
    pub verify_key: String,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct ServerJoin {
    pub addr: Pubkey,
    pub endpoint: String,
    pub access_version: u64,
    pub verify_key: String,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct Vote {
    pub voter: Pubkey,
    pub votee: Pubkey,
    pub vote_type: VoteType,
}

// State of on-chain GameAccount
#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(Default, BorshDeserialize, BorshSerialize, Debug)]
pub struct GameState {
    pub is_initialized: bool,
    // the contract version, used for upgrade
    pub version: String,
    // game name displayed on chain
    pub title: String,
    // addr to the game core logic program on Arweave
    pub bundle_addr: Pubkey,
    // addr to the account that holds all players' deposits
    pub stake_account: Pubkey,
    // game owner who created this game account
    pub owner: Pubkey,
    // mint id of the token used for game
    pub token_mint: Pubkey,
    // addr of the first server joined the game
    pub transactor_addr: Option<Pubkey>,
    // a serial number, increased by 1 after each PlayerJoin or ServerJoin
    pub access_version: u64,
    // a serial number, increased by 1 after each settlement
    pub settle_version: u64,
    // game size
    pub max_players: u16,
    // game players
    pub players: Box<Vec<PlayerJoin>>,
    // game servers (max: 10)
    pub servers: Box<Vec<ServerJoin>>,
    // length of game-specific data
    pub data_len: u32,
    // serialized data of game-specific data such as sb/bb in Texas Holdem
    pub data: Box<Vec<u8>>,
    // game votes
    pub votes: Box<Vec<Vote>>,
    // unlock time
    pub unlock_time: Option<u64>,
    // the entry type
    pub entry_type: EntryType,
    // the recipient account
    pub recipient_addr: Pubkey,
    // the checkpoint state
    pub checkpoint: Box<Vec<u8>>,
}

impl IsInitialized for GameState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
