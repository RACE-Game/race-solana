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

#[derive(Default, Debug, PartialEq, Eq, Clone, BorshSerialize, BorshDeserialize)]
pub enum EntryLock {
    #[default]
    Open,
    JoinOnly,
    DepositOnly,
    Closed,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct PlayerJoin {
    pub addr: Pubkey,
    pub position: u16,
    pub access_version: u64,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct ServerJoin {
    pub addr: Pubkey,
    pub endpoint: String,
    pub access_version: u64,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct PlayerBalance {
    pub player_id: u64,
    pub balance: u64,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub enum DepositStatus {
    #[default]
    Pending,
    Rejected,
    Refunded,
    Accepted,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct PlayerDeposit {
    pub addr: Pubkey,
    pub amount: u64,
    pub access_version: u64,
    pub settle_version: u64,
    pub status: DepositStatus,
}


#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct Vote {
    pub voter: Pubkey,
    pub votee: Pubkey,
    pub vote_type: VoteType,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct Bonus {
    pub identifier: String,
    pub stake_addr: Pubkey,
    pub token_addr: Pubkey,
    pub amount: u64,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq, Clone)]
pub enum GameStatus {
    #[default]
    Initializing,
    Initialized,
    Closed,
}

// State of on-chain GameAccount
#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(Default, BorshDeserialize, BorshSerialize, Debug)]
pub struct GameState {
    pub game_status: GameStatus,
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
    // the account to save all players
    pub players_reg_account: Pubkey,
    // deposits
    pub deposits: Vec<PlayerDeposit>,
    // game servers (max: 10)
    pub servers: Vec<ServerJoin>,
    // length of game-specific data
    pub data_len: u32,
    // serialized data of game-specific data such as sb/bb in Texas Holdem
    pub data: Vec<u8>,
    // game votes
    pub votes: Vec<Vote>,
    // unlock time
    pub unlock_time: Option<u64>,
    // the entry type
    pub entry_type: EntryType,
    // the recipient account
    pub recipient_addr: Pubkey,
    // the checkpoint state
    pub checkpoint: Vec<u8>,
    // the lock for game entry
    pub entry_lock: EntryLock,
    // a list of bonuses that can be awarded in game
    pub bonuses: Vec<Bonus>,
    // a list of balance snapshot for current checkpoint
    pub balances: Vec<PlayerBalance>,
}

impl IsInitialized for GameState {
    fn is_initialized(&self) -> bool {
        self.game_status == GameStatus::Initialized
    }
}
