use crate::{
    error::ProcessError, processor::misc::pack_state_to_account, state::{GameReg, GameState, RegistryState}
};

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar
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

    let game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;
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

    msg!(
        "Registered game {} to {}",
        game_account.key.clone(),
        registry_account.key.clone()
    );

    pack_state_to_account(registry_state, &registry_account, &payer, &system_program)?;

    Ok(())
}
