use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, pubkey::Pubkey, program_pack::Pack, program_error::ProgramError};

use crate::{types::AssignRecipientParams, state::{RecipientState, RecipientSlotOwner}, error::ProcessError};

#[inline(never)]
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: AssignRecipientParams,
) -> ProgramResult {

    let AssignRecipientParams { identifier } = params;

    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let recipient_account = next_account_info(accounts_iter)?;
    let assign_account = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut recipient_state = RecipientState::unpack(&recipient_account.try_borrow_data()?)?;

    if recipient_state.cap_addr.ne(payer.key) {
        return Err(ProcessError::NoRecipientUpdateCap)?;
    }

    for slot in recipient_state.slots.iter_mut() {
        for share in slot.shares.iter_mut() {
            match &share.owner {
                RecipientSlotOwner::Unassigned { identifier: target_identifier } => {
                    if target_identifier.eq(&identifier) {
                        share.owner = RecipientSlotOwner::Assigned {
                            addr: assign_account.key.clone(),
                        }
                    }
                }
                _ => (),
            }
        }
    }

    RecipientState::pack(recipient_state, &mut recipient_account.try_borrow_mut_data()?)?;

    Ok(())
}
