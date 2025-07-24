use crate::ser::{CursorType as CT};

pub const IS_INITIALIZED: u8 = 0;
pub const VERSION: u8 = 1;
pub const TITLE: u8 = 2;
pub const BUNDLE_ADDR: u8 = 3;
pub const STAKE_ACCOUNT: u8 = 4;
pub const OWNER: u8 = 5;
pub const TOKEN_MINT: u8 = 6;
pub const TRANSACTOR_ADDR: u8 = 7;
pub const ACCESS_VERSION: u8 = 8;
pub const SETTLE_VERSION: u8 = 9;
pub const MAX_PLAYERS: u8 = 10;
pub const PLAYERS: u8 = 11;
pub const DEPOSITS: u8 = 12;
pub const SERVERS: u8 = 13;
pub const DATA_LEN: u8 = 14;
pub const DATA: u8 = 15;
pub const VOTES: u8 = 16;
pub const UNLOCK_TIME: u8 = 17;
pub const ENTRY_TYPE: u8 = 18;
pub const RECIPIENT_ADDR: u8 = 19;
pub const CHECKPOINT: u8 = 20;
pub const ENTRY_LOCK: u8 = 21;
pub const BONUSES: u8 = 22;
pub const BALANCES: u8 = 23;

pub const PLAYER_ADDR: u8 = 0;
pub const PLAYER_POSITION: u8 = 1;
pub const PLAYER_ACCESS_VERSION: u8 = 2;
pub const PLAYER_VERIFY_KEY: u8 = 3;

pub const DEPOSIT_ADDR: u8 = 0;
pub const DEPOSIT_AMOUNT: u8 = 1;
pub const DEPOSIT_ACCESS_VERSION: u8 = 2;
pub const DEPSOIT_SETTLE_VERSION: u8 = 3;
pub const DEPSOIT_STATUS: u8 = 4;

pub const SERVER_ADDR: u8 = 0;
pub const SERVER_ENDPOINT: u8 = 1;
pub const SERVER_ACCESS_VERSION: u8 = 2;
pub const SERVER_VERIFY_KEY: u8 = 3;

pub fn create_game_cursor_type() -> CT {
    CT::mk_struct(vec![
        CT::Bool,                  // is_initialized
        CT::String,                // version
        CT::String,                // title
        CT::Pubkey,                // bundle_addr
        CT::Pubkey,                // stake_account
        CT::Pubkey,                // owner
        CT::Pubkey,                // token_mint
        CT::mk_option(CT::Pubkey), // transactor_addr
        CT::U64,                   // access_version
        CT::U64,                   // settle_version
        CT::U16,                   // max_players
        CT::mk_vec(CT::mk_struct(vec![
            CT::Pubkey, // addr
            CT::U16,    // position
            CT::U64,    // access_version
            CT::String, // verify_key
        ])), // players
        CT::mk_vec(CT::mk_struct(vec![
            CT::Pubkey, // addr
            CT::U64,    // amount
            CT::U64,    // access_version
            CT::U64,    // settle_version
            CT::mk_enum(vec![
                CT::Empty, // Pending
                CT::Empty, // Rejected
                CT::Empty, // Refunded
                CT::Empty, // Accepted
            ]), // deposit status
        ])), // deposits
        CT::mk_vec(CT::mk_struct(vec![
            CT::Pubkey, // addr
            CT::String, // endpoint
            CT::U64,    // access_version
            CT::String, // verify_key
        ])), // servers
        CT::U32,                   // data_len
        CT::StaticVec,             // data
        CT::mk_vec(CT::mk_struct(vec![
            CT::Pubkey, // voter
            CT::Pubkey, // votee
            CT::U8,     // vote_type, assuming it is an enum represented by a U8
        ])), // votes
        CT::mk_option(CT::U64),    // unlock_time
        CT::Enum(vec![
            CT::Struct(vec![
                CT::U64, // min_deposit
                CT::U64, // max_deposit
            ]),
            CT::Struct(vec![CT::U64]),    // amount
            CT::Struct(vec![CT::String]), // collection
        ]), // entry_type
        CT::Pubkey,    // recipient_addr
        CT::StaticVec, // checkpoint
        CT::Enum(vec![
            CT::Empty, // Open
            CT::Empty, // JoinOnly
            CT::Empty, // DepositOnly
            CT::Empty, // Closed
        ]), // entry_lock
        CT::mk_vec(CT::Struct(vec![
            CT::String, // identifier
            CT::Pubkey, // stake_addr
            CT::Pubkey, // token_addr
            CT::U64,    // amount
        ])), // bonuses
        CT::mk_vec(CT::Struct(vec![
            CT::U64, // player_id
            CT::U64, // balance
        ])), // balances
    ])
}

#[cfg(test)]
mod tests {
    use borsh::BorshDeserialize;
    use crate::ser::Cursor;
    use super::*;
    use crate::state::game::{
        DepositStatus, EntryLock, EntryType, GameState, PlayerDeposit, PlayerJoin, ServerJoin,
    };
    use crate::types::VoteType;
    use solana_program::pubkey::Pubkey;

    fn create_example_game_state() -> GameState {
        let owner = Pubkey::default();
        let stake_account = Pubkey::default();
        let token_mint = Pubkey::default();

        // Create players
        let mut players: Vec<PlayerJoin> = (0..1).map(|i| PlayerJoin {
            addr: Pubkey::default(),
            position: i as u16,
            access_version: 999,
            verify_key: "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEhbxg8+aPcfBCXVv2sT9BEXaX8DAAoiyRqu/nVz/+quEll6XlW8qjl+jcHM3mrtPzwSl2tpEz2kZJkNTORSTlSA==}".to_string(),
        }).collect();

        // Create servers
        let mut servers: Vec<ServerJoin> = (0..3)
            .map(|i| ServerJoin {
                addr: Pubkey::default(),
                endpoint: format!("endpoint_{}", i),
                access_version: 0,
                verify_key: format!("verify_key_server_{}", i),
            })
            .collect();

        // Create deposits
        let mut deposits: Vec<PlayerDeposit> = (0..1)
            .map(|i| PlayerDeposit {
                addr: Pubkey::default(),
                amount: ((i + 1) * 1000) as u64,
                access_version: 0,
                settle_version: 0,
                status: DepositStatus::Pending,
            })
            .collect();

        GameState {
            is_initialized: true,
            version: "1.0".to_string(),
            title: "Example Game".to_string(),
            bundle_addr: Pubkey::default(),
            stake_account,
            owner,
            token_mint,
            transactor_addr: None,
            access_version: 0,
            settle_version: 0,
            max_players: 30,
            players,
            deposits,
            servers,
            data_len: 0,
            data: vec![0;10000],
            votes: vec![],
            unlock_time: None,
            entry_type: EntryType::Cash {
                min_deposit: 1,
                max_deposit: 9999,
            },
            recipient_addr: Pubkey::default(),
            checkpoint: vec![0u8; 10000],
            entry_lock: EntryLock::Open,
            bonuses: vec![],
            balances: vec![],
        }
    }

    #[test]
    fn test_deser() -> anyhow::Result<()> {
        let game_state = create_example_game_state();
        let src = borsh::to_vec(&game_state)?;
        println!("src len: {}", src.len());
        let game_cursor_type = create_game_cursor_type();
        println!(
            "cursor_type size: {}",
            borsh::to_vec(&game_cursor_type)?.len()
        );
        let (mut game_cursor, _) = Cursor::new(&game_cursor_type, &src, 0);
        println!("new cursor size: {}", borsh::to_vec(&game_cursor)?.len());

        let Cursor::Struct(ref mut sc) = game_cursor else {
            panic!("wrong cursor type");
        };
        let Cursor::Option(oc) = sc.get_cursor(TRANSACTOR_ADDR) else {
            panic!("wrong cursor type");
        };
        assert!(oc.get_inner().is_none());
        let Cursor::StaticVec(vc) = sc.get_cursor_mut(CHECKPOINT) else {
            panic!("wrong cursor type");
        };
        // vc.set(vec![1, 2, 3]);



        println!("u8={}, u16={}, u32={}, u64={}, usize={}, bool={}",
            std::mem::size_of::<u8>(),
            std::mem::size_of::<u16>(),
            std::mem::size_of::<u32>(),
            std::mem::size_of::<u64>(),
            std::mem::size_of::<usize>(),
            std::mem::size_of::<bool>(),
        );

        let mut v = vec![0; game_cursor.size()];
        game_cursor.write(&src, &mut v, 0);
        let game_state2 = GameState::try_from_slice(&v).unwrap();
        //assert_eq!(game_state2.checkpoint, vec![1, 2, 3]);

        Ok(())
    }
}
