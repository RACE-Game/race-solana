use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::rent::Rent,
};
use borsh::BorshDeserialize;

use crate::{constants::PROFILE_ACCOUNT_LEN, error::ProcessError, state::PlayerState};
use crate::constants::{PLAYER_PROFILE_SEED, PROFILE_VERSION};
use crate::types::CreatePlayerProfileParams;
use crate::processor::misc::pack_state_to_account;


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

    let system_program = next_account_info(account_iter)?;

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

    // If old credentials exists, it can't be altered
    if let Ok(old_profile_state) = PlayerState::try_from_slice(&profile_account.try_borrow_data()?) {
        if old_profile_state.credentials.ne(&params.credentials) {
            return Err(ProcessError::InconsistentCredentials)?;
        }
    }

    let profile_state = PlayerState {
        version: PROFILE_VERSION,
        nick: params.nick,
        pfp: pfp_pubkey,
        credentials: params.credentials,
    };

    pack_state_to_account(profile_state, &profile_account, &owner_account, &system_program)?;

    Ok(())
}
