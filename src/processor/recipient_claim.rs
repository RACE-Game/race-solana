use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::state::{RecipientSlot, RecipientSlotOwner, RecipientState};

use super::misc::{validate_receiver_account, TransferSource};

fn claim_from_slot(stake_amount: u64, slot: &mut RecipientSlot, owner: &Pubkey) -> u64 {
    let total_weights: u16 = slot.shares.iter().map(|s| s.weights).sum();
    let total_amount: u64 = slot.shares.iter().map(|s| s.claim_amount).sum::<u64>() + stake_amount;

    for share in slot.shares.iter_mut() {
        match &share.owner {
            RecipientSlotOwner::Assigned { addr } if addr.eq(owner) => {
                let claim = (total_amount * share.weights as u64 / total_weights as u64)
                    - share.claim_amount;
                share.claim_amount += claim;
                return claim;
            }
            _ => (),
        }
    }

    0
}

#[inline(never)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let recipient_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let mut recipient_state = RecipientState::unpack(&recipient_account.try_borrow_data()?)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    for slot in recipient_state.slots.iter_mut() {
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
        let total_claim = claim_from_slot(slot_stake_state.amount, slot, payer.key);

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

    RecipientState::pack(
        recipient_state,
        &mut recipient_account.try_borrow_mut_data()?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::state::{RecipientSlotShare, RecipientSlotType};

    use super::*;

    #[test]
    fn test_claim_amount() {
        let alice = Pubkey::new_unique();
        let bob = Pubkey::new_unique();
        // alice share: 1
        // bob share: 2
        let mut slot = RecipientSlot {
            id: 0,
            slot_type: RecipientSlotType::Token,
            token_addr: Pubkey::default(),
            stake_addr: Pubkey::default(),
            shares: vec![
                RecipientSlotShare {
                    owner: RecipientSlotOwner::Assigned { addr: alice },
                    weights: 1,
                    claim_amount: 0,
                },
                RecipientSlotShare {
                    owner: RecipientSlotOwner::Assigned { addr: bob },
                    weights: 2,
                    claim_amount: 0,
                },
            ],
        };
        let mut stake_amount = 150;
        // 150 in total -> alice takes 50 -> 100 left
        assert_eq!(claim_from_slot(stake_amount, &mut slot, &alice), 50);
        assert_eq!(slot.shares[0].claim_amount, 50);
        stake_amount -= 50;

        // deposit 150 -> 300 in total -> bob takes 200 -> 100 left
        stake_amount += 150;
        assert_eq!(claim_from_slot(stake_amount, &mut slot, &bob), 200);
        assert_eq!(slot.shares[1].claim_amount, 200);
        stake_amount -= 200;

        // deposit 60 -> 360 in total -> alice takes 50(reach claim cap) -> 100 left
        stake_amount += 60;
        assert_eq!(claim_from_slot(stake_amount, &mut slot, &alice), 70);
        assert_eq!(slot.shares[0].claim_amount, 120);
        stake_amount -= 70;

        println!("stake amount: {}", stake_amount);
        assert_eq!(claim_from_slot(stake_amount, &mut slot, &bob), 40);
    }
}
