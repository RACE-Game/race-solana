use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use borsh::BorshDeserialize;

use crate::{error::ProcessError, state::PlayerState};
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

    if profile_pubkey != *profile_account.key {
        return Err(ProcessError::InvalidAccountPubkey)?;
    }

    let pfp_pubkey = if pfp_account.key.eq(&Pubkey::default()) {
        None
    } else {
        Some(pfp_account.key.clone())
    };

    // Credentials of an existing v2 profile can't be altered. Zeroed or
    // legacy (pre-credentials) data doesn't count, so that a closed profile
    // can be recreated and a legacy profile upgraded.
    if profile_account.owner.eq(program_id) {
        if let Ok(old_profile_state) = PlayerState::try_from_slice(&profile_account.try_borrow_data()?) {
            if old_profile_state.version.eq(&PROFILE_VERSION)
                && old_profile_state.credentials.ne(&params.credentials)
            {
                return Err(ProcessError::InconsistentCredentials)?;
            }
        }
    }

    let profile_state = PlayerState {
        version: PROFILE_VERSION,
        nick: params.nick,
        pfp: pfp_pubkey,
        credentials: params.credentials,
    };

    // Recreate the profile account if it no longer exists. The account is
    // created with seed at the exact size of the serialized state, so
    // clients don't have to guess PROFILE_ACCOUNT_LEN (which is too small
    // for profiles with credentials).
    if profile_account.owner.ne(program_id) {
        let data_len = borsh::object_length(&profile_state)?;
        let lamports = Rent::get()?.minimum_balance(data_len);
        invoke(
            &system_instruction::create_account_with_seed(
                owner_account.key,
                profile_account.key,
                owner_account.key,
                PLAYER_PROFILE_SEED,
                lamports,
                data_len as u64,
                program_id,
            ),
            &[
                owner_account.clone(),
                profile_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    // Grows the account when the serialized state exceeds its current size
    // and tops up rent from the owner.
    pack_state_to_account(profile_state, &profile_account, &owner_account, &system_program)?;

    Ok(())
}
