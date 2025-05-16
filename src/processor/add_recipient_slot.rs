use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey
};
use spl_token::{
    instruction::{set_authority, AuthorityType},
    state::Account,
};

use crate::{
    error::ProcessError, processor::misc::is_native_mint,
    state::{RecipientSlot, RecipientState}, types::RecipientSlotInit,
};

use super::misc::pack_state_to_account;

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: RecipientSlotInit,
) -> ProgramResult {
    let RecipientSlotInit { id, slot_type, token_addr, stake_addr, init_shares } = params;

    if init_shares.is_empty() {
        return Err(ProcessError::EmptyRecipientSlotShares)?;
    }

    let accounts_iter = &mut accounts.iter();

    let payer_account = next_account_info(accounts_iter)?;
    let recipient_account = next_account_info(accounts_iter)?;
    let stake_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut recipient_state = RecipientState::try_from_slice(&recipient_account.try_borrow_mut_data()?)?;

    if !recipient_state.is_initialized {
        return Err(ProcessError::InvalidRecipientAddress)?;
    }

    if recipient_state.cap_addr.is_some_and(|ca| ca.ne(&payer_account.key)) {
        return Err(ProcessError::NoRecipientUpdateCap)?;
    }

    if recipient_state.slots.iter().find(|slot| slot.token_addr.eq(&token_addr)).is_some() {
        return Err(ProcessError::DuplicatedRecipientSlotToken)?;
    }

    if recipient_state.slots.iter().find(|slot| slot.id.eq(&id)).is_some() {
        return Err(ProcessError::InvalidSlotId)?;
    }

    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[recipient_account.key.as_ref(), &[id]], program_id);

    if is_native_mint(&token_addr) {
        if stake_account.key.ne(&pda) {
            msg!("For SOL slot, must use PDA as stake account");
            return Err(ProcessError::InvalidSlotStakeAccount)?;
        }
    } else {
        let stake_account_state = Account::unpack(&stake_account.try_borrow_data()?)?;
        if stake_account_state.mint.ne(&token_addr) {
            return Err(ProgramError::InvalidArgument);
        }
        // Transfer the authority to PDA account
        let set_authority_ix = set_authority(
            token_program.key,
            stake_account.key,
            Some(&pda),
            AuthorityType::AccountOwner,
            payer_account.key,
            &[&payer_account.key],
        )?;

        invoke(
            &set_authority_ix,
            &[
                stake_account.clone(),
                payer_account.clone(),
                token_program.clone(),
            ],
        )?;
    }

    for share in init_shares.iter() {
        match &share.owner {
            crate::state::RecipientSlotOwner::Unassigned { identifier } => {
                if identifier.is_empty() || identifier.len() > 16 {
                    return Err(ProcessError::InvalidIdentifierLength)?;
                }
            }
            crate::state::RecipientSlotOwner::Assigned { .. } => (),
        }
    }

    let slot_to_add = RecipientSlot {
        id,
        slot_type,
        token_addr,
        stake_addr,
        shares: init_shares.into_iter().map(Into::into).collect(),
    };

    recipient_state.slots.push(slot_to_add);

    pack_state_to_account(recipient_state, &recipient_account, &payer_account, &system_program)?;

    Ok(())
}
