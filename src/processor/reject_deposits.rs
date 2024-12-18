use crate::state::DepositStatus;
use crate::types::RejectDepositsParams;
use crate::{error::ProcessError, state::GameState};
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
};

use super::misc::{general_transfer, pack_state_to_account, validate_receiver};

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: RejectDepositsParams,
) -> ProgramResult {
    let RejectDepositsParams { reject_deposits } = params;

    let mut account_iter = accounts.iter();

    let transactor_account = next_account_info(&mut account_iter)?;

    let game_account = next_account_info(&mut account_iter)?;

    let stake_account = next_account_info(&mut account_iter)?;

    let pda_account = next_account_info(&mut account_iter)?;

    let token_program = next_account_info(&mut account_iter)?;

    let system_program = next_account_info(&mut account_iter)?;

    if !transactor_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;

    for reject_deposit in reject_deposits {
        let Some(deposit) = game_state
            .deposits
            .iter_mut()
            .find(|d| d.access_version == reject_deposit)
        else {
            msg!(
                "The deposit is not found: {}",
                reject_deposit
            );
            return Err(ProcessError::InvalidRejectDeposit)?;
        };

        if deposit.status != DepositStatus::Pending {
            return Err(ProcessError::DuplicatedDepositRejection)?;
        }

        deposit.status = DepositStatus::Rejected;

        let receiver_account = next_account_info(&mut account_iter)?;

        if validate_receiver(&deposit.addr, &game_state.token_mint, &receiver_account.key).is_ok() {
            let (_, bump_seed) =
                Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

            general_transfer(
                stake_account,
                receiver_account,
                &game_state.token_mint,
                Some(deposit.amount),
                pda_account,
                &[&[game_account.key.as_ref(), &[bump_seed]]],
                token_program,
            )?;

            deposit.status = DepositStatus::Refunded;
        }

        // The PlayerJoin with the same access_version should be removed as well
        // So the player can later join again

        game_state.players.retain(|p| p.access_version != reject_deposit);
    }

    pack_state_to_account(
        game_state,
        &game_account,
        &transactor_account,
        &system_program,
    )?;

    Ok(())
}
