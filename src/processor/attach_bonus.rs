//! Attach a bonus to a game.
//! The bonus are stored in a dedicated token account which will be given the authority of PDA.
//! Only SPL bonus is supported, SOL/WSOL are not supported.

use crate::processor::misc::{is_native_mint, pack_state_to_account};
use crate::state::Bonus;
use crate::types::AttachBonusParams;
use crate::{
    error::ProcessError,
    state::GameState,
};
use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::rent::Rent,
};
use spl_token::instruction::{set_authority, AuthorityType};
use spl_token::state::Account;

#[inline(never)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: AttachBonusParams) -> ProgramResult {

    let account_iter = &mut accounts.into_iter();

    let payer_account = next_account_info(account_iter)?;

    let game_account = next_account_info(account_iter)?;

    let token_program = next_account_info(account_iter)?;

    let system_program = next_account_info(account_iter)?;

    if params.identifiers.iter().any(|i| i.len() > 16 || i.is_empty()) {
        return Err(ProcessError::InvalidIdentifierLength)?;
    }

    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let rent = Rent::default();

    if !rent.is_exempt(game_account.lamports(), game_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;

    let (pda, _bump_seed) = Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

    msg!("Expect to attach {} bonuses", params.identifiers.len());

    for identifier in params.identifiers {

        let temp_account = next_account_info(account_iter)?;

        let temp_state = Account::unpack(&temp_account.try_borrow_data()?)?;

        if is_native_mint(&temp_state.mint) {
            return Err(ProcessError::NativeTokenNotSupported)?;
        }

        msg!("Attach bonus at {} to {}", identifier, temp_account.key);

        let bonus = Bonus {
            identifier,
            amount: temp_state.amount,
            stake_addr: temp_account.key.clone(),
            token_addr: temp_state.mint.clone(),
        };

        game_state.bonuses.push(bonus);

        let set_authority_ix = set_authority(
            token_program.key,
            temp_account.key,
            Some(&pda),
            AuthorityType::AccountOwner,
            payer_account.key,
            &[&payer_account.key],
        )?;

        invoke(
            &set_authority_ix,
            &[temp_account.clone(), payer_account.clone(), token_program.clone()],
        )?;
    }

    pack_state_to_account(game_state, game_account, payer_account, system_program)?;

    Ok(())
}
