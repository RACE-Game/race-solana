use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey, program::invoke,
};
use spl_token::{state::Account, instruction::{AuthorityType, set_authority}};

use crate::{
    constants::RECIPIENT_ACCOUNT_LEN, state::RecipientState, types::CreateRecipientParams,
};

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: CreateRecipientParams,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let cap_account = next_account_info(accounts_iter)?;
    let recipient_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    let CreateRecipientParams { slots } = params;

    let (pda, _bump_seed) = Pubkey::find_program_address(&[recipient_account.key.as_ref()], program_id);

    for slot in slots.iter() {
        let slot_stake_account = next_account_info(accounts_iter)?;
        if slot.stake_addr.ne(slot_stake_account.key) {
            return Err(ProgramError::InvalidArgument);
        }
        let stake_account_state = Account::unpack(&slot_stake_account.try_borrow_data()?)?;
        if stake_account_state.mint.ne(&slot.token_addr) {
            return Err(ProgramError::InvalidArgument);
        }

        // Transfer the authority to PDA account
        let set_authority_ix = set_authority(
            token_program.key,
            slot_stake_account.key,
            Some(&pda),
            AuthorityType::AccountOwner,
            payer.key,
            &[&payer.key],
        )?;

        invoke(
            &set_authority_ix,
            &[slot_stake_account.clone(), payer.clone(), token_program.clone()],
        )?;
    }

    let recipient_state = RecipientState {
        is_initialized: true,
        cap_addr: cap_account.key.clone(),
        slots,
    };

    if recipient_account.data_len() != RECIPIENT_ACCOUNT_LEN {
        return Err(ProgramError::AccountDataTooSmall);
    }

    RecipientState::pack(
        recipient_state,
        &mut recipient_account.try_borrow_mut_data()?,
    )?;

    msg!("Created recipient account: {:?}", recipient_account.key);

    Ok(())
}
