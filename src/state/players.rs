/// Helper fuctions for reading and mutating players information of a game.
/// Each game has one players account, which contains a list of all PlayerJoin data.
/// Each PlayerJoin is stored with 171 bytes.
/// Since the verify_key isn't really useful in the contract logic, we usually skip it by deserilaizing the PlayerJoin with
/// PlayerJoinWithoutKey struct, which is 43 bytes, completely on stack.
///
/// The account structure:
///
/// [u16][PlayerJoin]*
/// |    |
/// |    |___ The array of PlayerJoins
/// |___ The number of players
///
use borsh::BorshDeserialize;
use std::io;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use crate::state::PlayerJoin;

const HEAD_LEN: usize = 2;
const PUBKEY_LEN: usize = 32;
const PLAYER_INFO_LEN: usize = 171;
const PLAYER_INFO_WITHOUT_KEY_LEN: usize = 42;

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, BorshDeserialize)]
pub struct PlayerJoinWithoutKey {
    pub addr: Pubkey,
    pub position: u16,
    pub access_version: u64,
}

pub fn get_players_count(data: &[u8]) -> Result<u16, ProgramError> {
    Ok(u16::try_from_slice(&data[..HEAD_LEN])?)
}

pub fn get_player_by_index(
    data: &[u8],
    index: usize,
) -> Result<Option<PlayerJoinWithoutKey>, ProgramError> {
    let size = get_players_count(data)? as usize;
    if index >= size as usize {
        return Ok(None);
    }
    let start = index * PLAYER_INFO_LEN + HEAD_LEN;
    let end = start + PLAYER_INFO_WITHOUT_KEY_LEN;
    let data = &data[start..end];
    Ok(Some(PlayerJoinWithoutKey::try_from_slice(data)?))
}

pub fn get_player_by_addr(
    data: &[u8],
    addr: &Pubkey,
) -> Result<Option<(usize, PlayerJoinWithoutKey)>, ProgramError> {
    let size = get_players_count(data)? as usize;
    for i in 0..size {
        let addr_start = i * PLAYER_INFO_LEN + HEAD_LEN;
        let addr_end = addr_start + PUBKEY_LEN;
        if addr.as_ref() == &data[addr_start..addr_end] {
            return Ok(Some((i, PlayerJoinWithoutKey::try_from_slice(data)?)));
        }
    }
    return Ok(None)
}

/// Add new player to the account. Return Some(index_of_the_player) if success.  If the player can't
/// be added, the caller should realloc the account and retry.
pub fn add_player(
    data: &mut [u8],
    player: &PlayerJoin,
) -> Result<Option<usize>, ProgramError> {
    let size = get_players_count(data)? as usize;
    println!("Size: {}", size);
    // Find a slot
    for i in 0..size {
        let start = i * PLAYER_INFO_LEN + HEAD_LEN;
        let addr_end = start + PUBKEY_LEN;
        println!("data should be all zeros: {:?}", &data[start..addr_end]);
        if data[start..addr_end].iter().all(|&n| n == 0) {
            // Found an empty slot, insert and return
            borsh::to_writer(&mut data[start..(start+PLAYER_INFO_LEN)], player)?;
            return Ok(Some(i));
        }
    }
    Ok(None) // Failed to insert
}

pub fn remove_player_by_index(
    data: &mut [u8],
    index: usize,
) -> Result<(), ProgramError> {
    let start = index * PLAYER_INFO_LEN + HEAD_LEN;
    let end = start + PLAYER_INFO_LEN;
    data[start..end].fill(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    fn create_player(addr: Pubkey, position: u16, access_version: u64, verify_key: &str) -> PlayerJoin {
        PlayerJoin {
            addr,
            position,
            access_version,
            verify_key: verify_key.to_string(),
        }
    }

    fn setup_data(players: Vec<PlayerJoin>) -> Vec<u8> {
        let mut data = vec![0; HEAD_LEN + players.len() * PLAYER_INFO_LEN];
        borsh::to_writer(&mut data[..HEAD_LEN], &(players.len() as u16)).unwrap();
        for (i, player) in players.iter().enumerate() {
            let start = i * PLAYER_INFO_LEN + HEAD_LEN;
            let mut w = io::Cursor::new(&mut data[start..(start + PLAYER_INFO_LEN)]);
            player.serialize(&mut w).unwrap();
        }
        data
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
        let mut data = vec![0; HEAD_LEN + 2 * PLAYER_INFO_LEN];
        borsh::to_writer(&mut data[..HEAD_LEN], &2u16).unwrap();
        let result = add_player(&mut data, &player).unwrap();
        assert_eq!(result, Some(0));
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
    }
}
