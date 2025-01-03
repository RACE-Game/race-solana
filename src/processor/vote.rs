use crate::{error::ProcessError, state::GameState};
use crate::{
    state::Vote,
    types::{VoteParams, VoteType},
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use super::misc::pack_state_to_account;

#[inline(never)]
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: VoteParams,
) -> ProgramResult {
    let VoteParams { vote_type } = params;
    let account_iter = &mut accounts.iter();

    let voter_account = next_account_info(account_iter)?;
    let game_account = next_account_info(account_iter)?;
    let votee_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !voter_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;

    // Validate voter identity

    let transactor_addr = game_state
        .transactor_addr
        .as_ref()
        .ok_or(ProcessError::GameNotServed)?;

    if voter_account.key.ne(transactor_addr) || votee_account.key.eq(voter_account.key) {
        return Err(ProcessError::InvalidVoteeAccount)?;
    }

    match vote_type {
        VoteType::ServerVoteTransactorDropOff => {
            if game_state
                .servers
                .iter()
                .any(|s| s.addr.eq(votee_account.key))
            {
                return Err(ProcessError::InvalidVoteeAccount)?;
            }

            game_state.votes.push(Vote {
                voter: voter_account.key.clone(),
                votee: votee_account.key.clone(),
                vote_type,
            });

            let clock = Clock::get()?.epoch;

            if game_state.votes.len() >= game_state.servers.len() / 2 {
                game_state.unlock_time = Some(clock + 10_000);
            }
        }
        VoteType::ClientVoteTransactorDropOff => return Err(ProcessError::Unimplemented)?,
    }

    pack_state_to_account(game_state, &game_account, &voter_account, &system_program)?;

    Ok(())
}
