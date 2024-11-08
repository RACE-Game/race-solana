use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

use crate::{error::ProcessError, state::GameState};
use spl_token::{instruction::{close_account, transfer}, state::Account};

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
    let ata_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;

    let game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;
    // check is_initialized?

    // Refuse to close if some players still have balance in game.
    for p in game_state.players.iter() {
        if p.balance > 0 {
            return Err(ProcessError::CantCloseGame)?;
        }
    }

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
    let token_state = Account::unpack(&stake_account.try_borrow_data()?)?;
    if token_state.amount > 0 {

        let transfer_ix = transfer(
            token_program.key,
            stake_account.key,
            ata_account.key,
            pda_account.key,
            &[&pda_account.key],
            token_state.amount
        )?;

        invoke_signed(
            &transfer_ix,
            &[
                stake_account.clone(),
                ata_account.clone(),
                pda_account.clone(),
            ],
            &[&[game_account.key.as_ref(), &[bump_seed]]]
        )?;
    }

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

    **owner_account.lamports.borrow_mut() = owner_account
        .lamports()
        .checked_add(game_account.lamports())
        .ok_or(ProcessError::StakeAmountOverflow)?;
    msg!("Lamports of the account returned to its owner");
    **game_account.lamports.borrow_mut() = 0;

    msg!("Successfully closed the game account: {}", game_account.key);
    Ok(())
}
