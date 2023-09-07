use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey, msg,
};
use spl_token::state::Account;

use crate::state::{RecipientSlotOwner, RecipientState};

use super::misc::{TransferSource, validate_receiver_account};

#[inline(never)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let recipient_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let mut recipient_state = RecipientState::unpack(&recipient_account.try_borrow_data()?)?;

    for slot in recipient_state.slots.iter_mut() {
        let total_weights: u16 = slot.shares.iter().map(|s| s.weights).sum();

        // The slot stake account
        let slot_stake_account = next_account_info(accounts_iter)?;
        let receiver_ata = next_account_info(accounts_iter)?;

        if slot_stake_account.key.ne(&slot.stake_addr) {
            return Err(ProgramError::InvalidAccountData);
        }
        let slot_stake_state = Account::unpack(&slot_stake_account.try_borrow_data()?)?;
        if slot_stake_state.mint.ne(&slot.token_addr) {
            return Err(ProgramError::InvalidAccountData);
        }

        // The total amount for both claimed and unclaimed
        let total_amount: u64 =
            slot.shares.iter().map(|s| s.claim_amount).sum::<u64>() + slot_stake_state.amount;

        let mut total_claim = 0;

        for share in slot.shares.iter_mut() {
            let breakpoint_amount =
                share.claim_amount_cap * total_weights as u64 / share.weights as u64;
            match &share.owner {
                RecipientSlotOwner::Assigned { addr } if addr.eq(payer.key) => {
                    if breakpoint_amount < total_amount {
                        let claim = share.claim_amount_cap - share.claim_amount;
                        total_claim += claim;
                        share.claim_amount += claim;
                    } else {
                        let claim = share.weights as u64 / total_weights as u64 - share.claim_amount;
                        total_claim += total_amount * claim;
                        share.claim_amount += claim;
                    }
                }
                _ => (),
            }
        }

        if total_claim > 0 {
            let transfer_source = TransferSource::try_new(
                system_program.clone(),
                token_program.clone(),
                slot_stake_account.clone(),
                recipient_account.key.as_ref(),
                pda_account.clone(),
                program_id,
            )?;

            msg!("Pay {} to {}", total_claim, receiver_ata.key);
            validate_receiver_account(&payer.key, &slot.token_addr, receiver_ata.key)?;
            transfer_source.transfer(receiver_ata, total_claim)?;
        }
    }

    RecipientState::pack(recipient_state, &mut recipient_account.try_borrow_mut_data()?)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_claim_amount_calc() {}
}
