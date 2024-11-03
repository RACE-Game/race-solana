use crate::{
    error::ProcessError,
    state::{GameState, RegistryState},
};

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar
};

#[inline(never)]
pub fn process(_programe_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let payer = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let game_account = next_account_info(account_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut registry_state = RegistryState::try_from_slice(&registry_account.try_borrow_data()?)?;

    if registry_state.is_private && registry_state.owner.ne(payer.key) {
        return Err(ProcessError::InvalidOwner)?;
    }

    // An empty game account can be removed without checking
    if !game_account.data_is_empty() {
        let game_state = GameState::unpack(&game_account.try_borrow_data()?)?;
        if !game_state.is_initialized {
            return Err(ProgramError::UninitializedAccount);
        }

        if game_state.owner.ne(payer.key) {
            return Err(ProcessError::InvalidOwner)?;
        }
        drop(game_state);
    }

    let mut removed = false;
    if registry_state
        .games
        .iter()
        .find(|reg| reg.addr.eq(game_account.key))
        .is_none()
    {
        return Err(ProcessError::InvalidUnregistration)?;
    } else if !removed {
        let mut unreg_idx = 0usize;
        for (idx, game) in registry_state.games.iter().enumerate() {
            if game.addr.eq(game_account.key) {
                unreg_idx = idx;
                break;
            }
        }
        let unreg_game = registry_state.games.remove(unreg_idx);
        msg!("Unregitered game {}", unreg_game.addr);

        let new_registry_account_data = borsh::to_vec(&registry_state)?;

        // Shrink the account size
        registry_account.realloc(new_registry_account_data.len(), false)?;
        registry_account.try_borrow_mut_data()?.copy_from_slice(&new_registry_account_data);

        removed = true;
    }

    if !removed {
        return Err(ProcessError::InvalidUnregistration)?;
    }

    Ok(())
}
