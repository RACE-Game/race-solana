/// Helper fuctions for reading and mutating players information of a game.  Each game has one
/// players registration account(players_reg_account), which contains a list of all [[PlayerJoin]]s.
/// Each PlayerJoin is stored as 170 bytes.  Since the verify_key isn't really useful in the
/// program logic, we usually skip it by deserilaizing the PlayerJoin with PlayerJoinWithoutKey
/// struct, which is 43 bytes, completely on stack.
///
/// The account structure:
///
/// [u64][u64][usize][128byte][4byte][PlayerJoin*]
/// |    |     |      |       |      |___ The array of PlayerJoins, each uses 170 bytes.
/// |    |     |      |       |___ The total number of slots, empty slots included.
/// |    |     |      |___ The position flags, 0 stands for empty, 1 stands for occupied.
/// |    |     |___ The number of players. It is legal to have some empty slots in the middle, those are not counted.
/// |    | __ The settle_version. Updated every time a settlement is procced.
/// |___ The access_version. Updated every time a new player joined.
///
use crate::error::ProcessError;
use crate::state::PlayerJoin;
use borsh::BorshDeserialize;
use solana_program::{program_error::ProgramError, pubkey::Pubkey, msg};

// lens for each fields
const VERSION_LEN: usize = 8;
const COUNT_LEN: usize = 8;
const PUBKEY_LEN: usize = 32;
const POSITION_FLAGS_LEN: usize = 128;
const PLAYER_INFO_LEN: usize = 170;
const PLAYER_INFO_WITHOUT_KEY_LEN: usize = 42;
const SLOTS_COUNT_LEN: usize = 4;

// lens for fields of PlayerJoin
const POSITION_LEN: usize = 2;
const POSITION_OFFSET: usize = PUBKEY_LEN;
const ID_OFFSET: usize = PUBKEY_LEN + POSITION_LEN;
const ID_LEN: usize = 8;

// offsets for each fields
const ACCESS_VERSION_OFFSET: usize = 0;
const SETTLE_VERSION_OFFSET: usize = ACCESS_VERSION_OFFSET + VERSION_LEN;
const COUNT_OFFSET: usize = SETTLE_VERSION_OFFSET + VERSION_LEN;
#[allow(unused)]
const POSITION_FLAGS_OFFSET: usize = COUNT_OFFSET + COUNT_LEN;
const SLOTS_COUNT_OFFSET: usize = POSITION_FLAGS_OFFSET + POSITION_FLAGS_LEN;

pub const HEAD_LEN: usize = SLOTS_COUNT_OFFSET + SLOTS_COUNT_LEN;

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, BorshDeserialize)]
pub struct PlayerJoinWithoutKey {
    pub addr: Pubkey,
    #[allow(unused)]
    pub position: u16,
    #[allow(unused)]
    pub access_version: u64,
}

pub fn validate_account_data(data: &[u8]) -> Result<(), ProgramError> {
    if data.len() != HEAD_LEN {
        return Err(ProcessError::InvalidPlayersRegAccountForInit)?;
    }

    Ok(())
}

pub fn set_versions(data: &mut [u8], access_version: u64, settle_version: u64) -> Result<(), ProgramError> {
    msg!("Set versions, access version = {}, settle version = {}", access_version, settle_version);
    borsh::to_writer(&mut data[ACCESS_VERSION_OFFSET..(ACCESS_VERSION_OFFSET+VERSION_LEN)], &access_version)?;
    borsh::to_writer(&mut data[SETTLE_VERSION_OFFSET..(SETTLE_VERSION_OFFSET+VERSION_LEN)], &settle_version)?;
    Ok(())
}

pub fn get_players_count(data: &[u8]) -> Result<usize, ProgramError> {
    Ok(usize::try_from_slice(&data[COUNT_OFFSET..(COUNT_OFFSET+COUNT_LEN)])?)
}

pub fn increase_players_count(data: &mut [u8]) -> Result<usize, ProgramError> {
    let size = get_players_count(&data)?;
    borsh::to_writer(&mut data[COUNT_OFFSET..(COUNT_OFFSET+COUNT_LEN)], &(size + 1))?;
    Ok(size + 1)
}

pub fn decrease_players_count(data: &mut [u8]) -> Result<usize, ProgramError> {
    let size = get_players_count(&data)?;
    if size == 0 {
        return Err(ProcessError::CantDecreasePlayersRegAccountSize)?;
    }
    borsh::to_writer(&mut data[COUNT_OFFSET..(COUNT_OFFSET+COUNT_LEN)], &(size - 1))?;
    Ok(size - 1)
}

#[allow(unused)]
pub fn get_player_by_index(
    data: &[u8],
    index: usize,
) -> Result<Option<PlayerJoinWithoutKey>, ProgramError> {
    let slots_count = get_slots_count(data)?;
    if index >= slots_count {
        return Ok(None);
    }
    let start = index * PLAYER_INFO_LEN + HEAD_LEN;
    let addr_end = start + PUBKEY_LEN;
    let end = start + PLAYER_INFO_WITHOUT_KEY_LEN;
    if data[start..addr_end].iter().any(|n| *n != 0) {
        let data = &data[start..end];
        Ok(Some(PlayerJoinWithoutKey::try_from_slice(data)?))
    } else {
        Ok(None)
    }
}

pub fn get_player_by_id(
    data: &[u8],
    id: u64,
) -> Result<Option<(usize, PlayerJoinWithoutKey)>, ProgramError> {
    let mut id_v = [0u8; 8];
    borsh::to_writer(&mut id_v[..], &id)?;
    let mut i = 0;
    while HEAD_LEN + PLAYER_INFO_LEN * i < data.len() {
        let start = HEAD_LEN + PLAYER_INFO_LEN * i;
        let id_start = start + ID_OFFSET;
        let id_end = id_start + ID_LEN;
        if &id_v == &data[id_start..id_end] {
            return Ok(Some((
                i,
                PlayerJoinWithoutKey::try_from_slice(
                    &data[start..(start + PLAYER_INFO_WITHOUT_KEY_LEN)],
                )?,
            )));
        }
        i += 1;
    }
    return Ok(None);
}

#[allow(unused)]
pub fn get_player_by_addr(
    data: &[u8],
    addr: &Pubkey,
) -> Result<Option<(usize, PlayerJoinWithoutKey)>, ProgramError> {
    let mut i = 0;
    while HEAD_LEN + PLAYER_INFO_LEN * i < data.len() {
        let start = HEAD_LEN + PLAYER_INFO_LEN * i;
        let addr_end = start + PUBKEY_LEN;
        if addr.as_ref() == &data[start..addr_end] {
            return Ok(Some((
                i,
                PlayerJoinWithoutKey::try_from_slice(
                    &data[start..(start + PLAYER_INFO_WITHOUT_KEY_LEN)],
                )?,
            )));
        }
        i += 1;
    }
    return Ok(None);
}

pub fn is_player_joined(data: &[u8], addr: &Pubkey) -> Result<bool, ProgramError> {
    let slots_count = get_slots_count(data)?;
    // Find a slot
    for i in 0..slots_count {
        let start = i * PLAYER_INFO_LEN + HEAD_LEN;
        let addr_end = start + PUBKEY_LEN;
        if addr.as_ref() == &data[start..addr_end] {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn is_position_occupied(data: &[u8], position: u16) -> Result<bool, ProgramError> {
    if data.len() < HEAD_LEN {
        return Err(ProcessError::MalformedPlayersRegAccount)?;
    }
    // We support at most 1024 players
    if position > 1024 {
        return Ok(true);
    }

    let i = position / 8;
    let o = position % 8;
    let f = 1 << o as u8;

    if f & (&data[POSITION_OFFSET + i as usize]) != 0 {
        return Ok(true);
    }

    Ok(false)
}

pub fn set_position_flag(data: &mut [u8], position: u16, flag: bool) -> Result<(), ProgramError> {
    if data.len() < HEAD_LEN {
        return Err(ProcessError::MalformedPlayersRegAccount)?;
    }

    let i = position / 8;
    let o = position % 8;
    let f = 1 << o as u8;

    if flag {
        data[POSITION_OFFSET + i as usize] |= f;
    } else {
        data[POSITION_OFFSET + i as usize] &= !f;
    }
    return Ok(());
}

pub fn get_available_position(data: &[u8], max_players: u16) -> Result<u16, ProgramError> {
    for position in 0u16..max_players {
        let i = position / 8;
        let o = position % 8;
        let f = 1 << o as u8;
        if data[POSITION_OFFSET + i as usize] & f == 0 {
            return Ok(i * 8 + o);
        }
    }
    return Err(ProcessError::GameFullAlready)?;
}

pub fn increase_size_set_position_flag(data: &mut [u8], position: u16) -> Result<(), ProgramError> {
    increase_players_count(data)?;
    set_position_flag(data, position, true)?;
    return Ok(());
}

pub fn get_slots_count(data: &[u8]) -> Result<usize, ProgramError> {
    let slots_count = u32::try_from_slice(&data[SLOTS_COUNT_OFFSET..(SLOTS_COUNT_OFFSET+SLOTS_COUNT_LEN)])?;
    Ok(slots_count as usize)
}

pub fn increase_slots_count(data: &mut [u8]) -> Result<(), ProgramError> {
    let slots_count = get_slots_count(&data)?;
    borsh::to_writer(&mut data[SLOTS_COUNT_OFFSET..(SLOTS_COUNT_OFFSET+SLOTS_COUNT_LEN)], &(slots_count as u32+1))?;
    Ok(())
}

/// Add new player to the account. Return Some(index_of_the_player) if success.  If the player can't
/// be added, the caller should realloc the account and retry.
pub fn add_player(data: &mut [u8], player: &PlayerJoin) -> Result<Option<usize>, ProgramError> {
    let slots_count = get_slots_count(&data)?;
    // Find a slot
    for i in 0..slots_count {
        let start = i * PLAYER_INFO_LEN + HEAD_LEN;
        let addr_end = start + PUBKEY_LEN;
        if data[start..addr_end].iter().all(|&n| n == 0) {
            // Found an empty slot, increase the player acount and insert player info.
            increase_players_count(data)?;
            set_position_flag(data, player.position, true)?;
            borsh::to_writer(&mut data[start..(start + PLAYER_INFO_LEN)], player)?;
            return Ok(Some(i));
        }
    }
    Ok(None) // Failed to insert
}

pub fn remove_player_by_index(data: &mut [u8], index: usize) -> Result<(), ProgramError> {
    let start = index * PLAYER_INFO_LEN + HEAD_LEN;
    let end = start + PLAYER_INFO_LEN;
    if &[0; 32] != &data[start..(start + PUBKEY_LEN)] {
        let pos_start = start + POSITION_OFFSET;
        let pos_end = pos_start + POSITION_LEN;
        let pos = u16::try_from_slice(&data[pos_start..pos_end])?;
        data[start..end].fill(0);
        set_position_flag(data, pos, false)?;
        decrease_players_count(data)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_player(
        addr: Pubkey,
        position: u16,
        access_version: u64,
        verify_key: &str,
    ) -> PlayerJoin {
        PlayerJoin {
            addr,
            position,
            access_version,
            verify_key: verify_key.to_string(),
        }
    }

    fn setup_data(players: Vec<PlayerJoin>) -> Vec<u8> {
        let mut data = vec![0; HEAD_LEN + players.len() * PLAYER_INFO_LEN];
        borsh::to_writer(&mut data[COUNT_OFFSET..(COUNT_OFFSET+COUNT_LEN)], &players.len()).unwrap();
        for (i, player) in players.iter().enumerate() {
            let start = i * PLAYER_INFO_LEN + HEAD_LEN;
            borsh::to_writer(&mut data[start..(start + PLAYER_INFO_LEN)], &player).unwrap();
            increase_slots_count(&mut data).unwrap();
        }
        data
    }

    #[test]
    fn test_init_account_data() {
        let mut v = vec![0; HEAD_LEN];
        validate_account_data(&mut v).unwrap();
        assert_eq!(v, vec![0; HEAD_LEN]);
    }

    #[test]
    #[should_panic]
    fn test_init_account_data_fail() {
        let mut v = vec![0; 3];
        validate_account_data(&mut v).unwrap();
    }

    #[test]
    fn test_get_players_count() {
        let players = vec![
            create_player(Pubkey::default(), 1, 1, "key1"),
            create_player(Pubkey::default(), 2, 2, "key2"),
        ];
        let data = setup_data(players);
        assert_eq!(get_players_count(&data).unwrap(), 2);
    }

    #[test]
    fn test_get_player_by_index() {
        let players = vec![
            create_player(Pubkey::new_unique(), 1, 1, "key1"),
            create_player(Pubkey::new_unique(), 2, 2, "key2"),
        ];
        let data = setup_data(players.clone());
        let second_player = get_player_by_index(&data, 1).unwrap().unwrap();
        assert_eq!(second_player.position, players[1].position);
    }

    #[test]
    fn test_get_player_by_addr() {
        let player1 = create_player(Pubkey::new_unique(), 1, 1, "key1");
        let player2 = create_player(Pubkey::new_unique(), 2, 2, "key2");
        let players = vec![player1.clone(), player2.clone()];
        let data = setup_data(players.clone());

        let (index, found_player) = get_player_by_addr(&data, &player2.addr).unwrap().unwrap();
        assert_eq!(index, 1);
        assert_eq!(found_player.position, player2.position);
    }

    #[test]
    fn test_add_player() {
        let player = create_player(Pubkey::new_unique(), 1, 1, "key");
        let mut data = setup_data(vec![player.clone()]);
        data.resize(data.len() + 171, 0);
        increase_slots_count(&mut data).unwrap();
        let result = add_player(&mut data, &player).unwrap();
        assert_eq!(result, Some(1));
        let added_player = get_player_by_index(&data, 0).unwrap().unwrap();
        assert_eq!(added_player.position, player.position);
    }

    #[test]
    fn test_remove_player_by_index() {
        let player = create_player(Pubkey::new_unique(), 1, 1, "key");
        let mut data = setup_data(vec![player.clone()]);
        remove_player_by_index(&mut data, 0).unwrap();
        let removed_player = get_player_by_index(&data, 0).unwrap();
        assert!(removed_player.is_none());
        assert_eq!(get_players_count(&data).unwrap(), 0);
        remove_player_by_index(&mut data, 0).unwrap();
        assert_eq!(is_position_occupied(&data, 1).unwrap(), false);
    }

    #[test]
    fn test_set_position_flag() {
        let mut data = vec![0; HEAD_LEN];
        assert_eq!(is_position_occupied(&data, 0).unwrap(), false);

        set_position_flag(&mut data, 0, true).unwrap();
        println!("data: {:?}", &data[POSITION_FLAGS_OFFSET..(POSITION_FLAGS_OFFSET+POSITION_FLAGS_LEN)]);
        assert_eq!(is_position_occupied(&data, 0).unwrap(), true);

        set_position_flag(&mut data, 0, false).unwrap();
        println!("data: {:?}", &data[POSITION_FLAGS_OFFSET..(POSITION_FLAGS_OFFSET+POSITION_FLAGS_LEN)]);
        assert_eq!(is_position_occupied(&data, 0).unwrap(), false);

        assert_eq!(is_position_occupied(&data, 1).unwrap(), false);

        set_position_flag(&mut data, 1, true).unwrap();
        println!("data: {:?}", &data[POSITION_FLAGS_OFFSET..(POSITION_FLAGS_OFFSET+POSITION_FLAGS_LEN)]);
        assert_eq!(is_position_occupied(&data, 1).unwrap(), true);

        set_position_flag(&mut data, 1, false).unwrap();
        println!("data: {:?}", &data[POSITION_FLAGS_OFFSET..(POSITION_FLAGS_OFFSET+POSITION_FLAGS_LEN)]);
        assert_eq!(is_position_occupied(&data, 1).unwrap(), false);
    }

    #[test]
    fn test_get_available_position() {
        let mut data = vec![0; HEAD_LEN + PLAYER_INFO_LEN * 2]; // Assume two players for simplicity
        borsh::to_writer(&mut data[..HEAD_LEN], &2usize).unwrap();

        assert_eq!(get_available_position(&data, 2).unwrap(), 0);
        set_position_flag(&mut data, 0, true).unwrap();
        assert_eq!(get_available_position(&data, 2).unwrap(), 1);

        assert_eq!(is_position_occupied(&data, 1).unwrap(), false);
        set_position_flag(&mut data, 1, true).unwrap();
        assert_eq!(is_position_occupied(&data, 1).unwrap(), true);
        assert!(get_available_position(&data, 2).is_err());
    }

    #[test]
    fn test_get_player_by_id() {
        let player = create_player(Pubkey::new_unique(), 1, 1, "key");
        let data = setup_data(vec![player.clone()]);
        let (index, found_player) = get_player_by_id(&data, 1).unwrap().unwrap();
        assert_eq!(index, 0);
        assert_eq!(found_player.position, player.position);
    }
}
