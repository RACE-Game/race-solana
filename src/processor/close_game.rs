use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::ProcessError,
    processor::misc::{general_transfer, is_native_mint},
    state::GameState,
};
use spl_token::instruction::close_account;

#[inline(never)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_account = next_account_info(account_iter)?;
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let game_account = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let pda_account = next_account_info(account_iter)?;
    let receiver_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;

    let game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;
    // check is_initialized?

    if game_state.owner.ne(&owner_account.key) {
        return Err(ProcessError::InvalidOwner)?;
    }
    if game_state.stake_account.ne(stake_account.key) {
        return Err(ProcessError::InvalidStakeAccount)?;
    }

    let (pda, bump_seed) = Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);
    if pda.ne(pda_account.key) {
        return Err(ProcessError::InvalidPDA)?;
    }

    // We transfer the remaining balance to the owner

    general_transfer(
        stake_account,
        receiver_account,
        &game_state.token_mint,
        None,
        pda_account,
        &[&[game_account.key.as_ref(), &[bump_seed]]],
        token_program,
    )?;

    if !is_native_mint(&game_state.token_mint) {
        msg!("Close stake account");
        let close_ix = close_account(
            token_program.key,
            stake_account.key,
            owner_account.key,
            pda_account.key,
            &[pda_account.key],
        )?;

        invoke_signed(
            &close_ix,
            &[
                stake_account.clone(),
                owner_account.clone(),
                pda_account.clone(),
            ],
            &[&[game_account.key.as_ref(), &[bump_seed]]],
        )?;
    }

    // Close game account and transfer the SOL to the owner
    **owner_account.lamports.borrow_mut() = owner_account
        .lamports()
        .checked_add(game_account.lamports())
        .ok_or(ProcessError::StakeAmountOverflow)?;
    msg!("Lamports of the account returned to its owner");
    **game_account.lamports.borrow_mut() = 0;

    msg!("Successfully closed the game account: {}", game_account.key);
    Ok(())
}
