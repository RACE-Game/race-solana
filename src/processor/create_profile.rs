use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::rent::Rent,
};

use crate::{constants::PROFILE_ACCOUNT_LEN, error::ProcessError, state::PlayerState};
use crate::constants::PLAYER_PROFILE_SEED;
use crate::types::CreatePlayerProfileParams;

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: CreatePlayerProfileParams,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let owner_account = next_account_info(account_iter)?;

    let profile_account = next_account_info(account_iter)?;

    let profile_pubkey =
        Pubkey::create_with_seed(owner_account.key, PLAYER_PROFILE_SEED, program_id)?;

    let pfp_account = next_account_info(account_iter)?;

    let _system_program = next_account_info(account_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !profile_account.is_writable {
        return Err(ProcessError::InvalidAccountStatus)?;
    }

    let rent = Rent::default();
    if profile_account.lamports() < rent.minimum_balance(PROFILE_ACCOUNT_LEN) {
        return Err(ProgramError::AccountNotRentExempt)?;
    }

    if profile_pubkey != *profile_account.key {
        return Err(ProcessError::InvalidAccountPubkey)?;
    }

    let pfp_pubkey = if pfp_account.key.eq(&Pubkey::default()) {
        None
    } else {
        Some(pfp_account.key.clone())
    };

    let player_state = PlayerState {
        is_initialized: true,
        nick: params.nick,
        pfp: pfp_pubkey,
    };

    msg!("player profile state: {:?}", &player_state);

    PlayerState::pack(player_state, &mut profile_account.try_borrow_mut_data()?)?;

    msg!("Profile addr: {:?}", profile_account.key);

    Ok(())
}
