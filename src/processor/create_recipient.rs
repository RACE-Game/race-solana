use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::{set_authority, AuthorityType},
    state::Account,
};

use crate::{
    constants::RECIPIENT_ACCOUNT_LEN, error::ProcessError, processor::misc::is_native_mint,
    state::RecipientState, types::CreateRecipientParams,
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

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let CreateRecipientParams { slots } = params;

    if slots.is_empty() {
        return Err(ProcessError::EmptyRecipientSlots)?;
    }

    for slot in slots.iter() {
        let slot_stake_account = next_account_info(accounts_iter)?;
        if slot.stake_addr.ne(slot_stake_account.key) {
            return Err(ProcessError::InvalidSlotStakeAccount)?;
        }

        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[recipient_account.key.as_ref(), &[slot.id]], program_id);

        if is_native_mint(&slot.token_addr) {
            if slot_stake_account.key.ne(&pda) {
                msg!("For SOL slot, must use PDA as stake account");
                return Err(ProcessError::InvalidSlotStakeAccount)?;
            }
        } else {
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
                &[
                    slot_stake_account.clone(),
                    payer.clone(),
                    token_program.clone(),
                ],
            )?;
        }
    }

    let slots = slots.into_iter().map(Into::into).collect();
    let recipient_state = RecipientState {
        is_initialized: true,
        cap_addr: Some(cap_account.key.clone()),
        slots,
    };

    if recipient_account.data_len() < RECIPIENT_ACCOUNT_LEN {
        return Err(ProgramError::AccountDataTooSmall);
    }

    RecipientState::pack(
        recipient_state,
        &mut recipient_account.try_borrow_mut_data()?,
    )?;

    msg!("Created recipient account: {:?}", recipient_account.key);

    Ok(())
}
