use crate::{
    error::ProcessError,
    state::{GameReg, GameState, RegistryState},
};

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar
};


#[inline(never)]
pub fn process(_programe_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let payer = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let game_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    msg!("payer pubkey {}", payer.key.clone());
    msg!("reg account {}", registry_account.key.clone());

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut registry_state = RegistryState::try_from_slice(&registry_account.try_borrow_data()?)?;
    msg!("owner pubkey {}", registry_state.owner.clone());

    if !registry_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if registry_state.is_private && registry_state.owner.ne(payer.key) {
        return Err(ProcessError::InvalidOwner)?;
    }

    // TODO: Check on transport side?
    if registry_state.games.len() as u16 == registry_state.size {
        return Err(ProcessError::RegistrationIsFull)?;
    }

    let game_state = GameState::unpack(&game_account.try_borrow_data()?)?;
    if !game_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if game_state.owner.ne(payer.key) {
        return Err(ProcessError::InvalidOwner)?;
    }

    if registry_state.size as usize == registry_state.games.len() {
        return Err(ProcessError::RegistrationIsFull)?;
    }

    if registry_state.games.len() > 0 {
        if registry_state
            .games
            .iter()
            .any(|reg| reg.addr.eq(game_account.key))
        {
            return Err(ProcessError::GameAlreadyRegistered)?;
        }
    }

    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp as u64;
    let reg_game = GameReg {
        title: game_state.title.clone(),
        addr: game_account.key.clone(),
        reg_time: timestamp,
        bundle_addr: game_state.bundle_addr.clone(),
    };

    registry_state.games.push(reg_game);

    let new_registry_account_data = borsh::to_vec(&registry_state)?;
    msg!("Realloc registry account to {}", new_registry_account_data.len());
    registry_account.realloc(new_registry_account_data.len(), false)?;

    msg!(
        "Registered game {} to {}",
        game_account.key.clone(),
        registry_account.key.clone()
    );
    registry_account.try_borrow_mut_data()?.copy_from_slice(&new_registry_account_data);

    msg!("Registry updated");
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(registry_account.data_len());
    let lamports_diff = new_minimum_balance.saturating_sub(registry_account.lamports());
    if lamports_diff > 0 {
        invoke(
            &system_instruction::transfer(payer.key, registry_account.key, lamports_diff),
            &[
                payer.clone(),
                registry_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    Ok(())
}
