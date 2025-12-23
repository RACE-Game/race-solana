use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::BorshDeserialize;

use crate::{error::ProcessError, state::ServerState};
use crate::constants::{SERVER_PROFILE_SEED, PROFILE_VERSION};
use crate::types::RegisterServerParams;
use crate::processor::misc::pack_state_to_account;


#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: RegisterServerParams,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let owner_account = next_account_info(account_iter)?;
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let server_account = next_account_info(account_iter)?;
    if !server_account.is_writable {
        return Err(ProcessError::InvalidAccountStatus)?;
    }

    let system_program = next_account_info(account_iter)?;

    let server_pubkey =
        Pubkey::create_with_seed(owner_account.key, SERVER_PROFILE_SEED, program_id)?;
    if server_pubkey != *server_account.key {
        return Err(ProcessError::InvalidAccountPubkey)?;
    }

    // If old credentials exists, it can't be altered
    if let Ok(old_server_state) = ServerState::try_from_slice(&server_account.try_borrow_data()?) {
        if old_server_state.credentials.ne(&params.credentials) {
            return Err(ProcessError::InconsistentCredentials)?;
        }
    }

    let server_state = ServerState {
        version: PROFILE_VERSION,
        addr: server_account.key.clone(),
        owner: *owner_account.key,
        endpoint: params.endpoint,
        credentials: params.credentials,
    };

    msg!("Server state: {:?}", &server_state);

    pack_state_to_account(server_state, &server_account, &owner_account, &system_program)?;

    Ok(())
}
