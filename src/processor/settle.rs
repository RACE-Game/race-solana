//! Settle game result
//!
//! Transfer the game assets between players, eject and pay leaving players.
//! This instruction is only available for current game transactor.
//!
//! Settles must be validated:
//! 1. All changes are sum up to zero.
//! 2. Player without assets must be ejected.

use crate::constants::MAX_SETTLE_INCREASEMENT;
use crate::state::RecipientState;
use crate::types::{SettleOp, SettleParams, Transfer};
use crate::{error::ProcessError, state::GameState};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

use super::misc::{validate_receiver_account, TransferSource};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: SettleParams,
) -> ProgramResult {
    let SettleParams {
        settles,
        transfers,
        checkpoint,
        settle_version,
        next_settle_version,
    } = params;

    let account_iter = &mut accounts.iter();

    let transactor_account = next_account_info(account_iter)?;

    let game_account = next_account_info(account_iter)?;

    let stake_account = next_account_info(account_iter)?;

    let pda_account = next_account_info(account_iter)?;

    let recipient_account = next_account_info(account_iter)?;

    let token_program = next_account_info(account_iter)?;

    let system_program = next_account_info(account_iter)?;

    if !transactor_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Collect the pays.
    let mut pays: Vec<(Pubkey, u64)> = Vec::new();

    // We should check the order of settles: add < sub < ejec
    // 0 for add, 1 for sub, 2 for eject.
    let mut op_type = 0;
    let mut game_state = GameState::unpack(&game_account.try_borrow_mut_data()?)?;

    if game_state.settle_version != settle_version {
        return Err(ProcessError::InvalidSettleVersion)?;
    }

    if next_settle_version > game_state.settle_version + MAX_SETTLE_INCREASEMENT
        || next_settle_version <= game_state.settle_version
    {
        return Err(ProcessError::InvalidNextSettleVersion)?;
    }

    if stake_account.key.ne(&game_state.stake_account) {
        msg!("Stake account expected: {:?}", game_state.stake_account);
        msg!("Stake account given: {:?}", stake_account.key);
        return Err(ProcessError::InvalidStakeAccount)?;
    }

    // Ensure changes are sum up to zero
    let mut sum = 0i128;

    for settle in settles.into_iter() {
        match settle.op {
            SettleOp::Add(amt) => {
                if op_type != 0 {
                    return Err(ProcessError::InvalidOrderOfSettles)?;
                }
                if let Some(player) = game_state
                    .players
                    .iter_mut()
                    .find(|p| p.position.eq(&settle.position))
                {
                    player.balance = player
                        .balance
                        .checked_add(amt)
                        .ok_or(ProcessError::PlayerBalanceOverflow)?;
                } else {
                    return Err(ProcessError::InvalidSettlePlayerAddress)?;
                }
                sum = sum.checked_add(i128::from(amt))
                    .ok_or(ProcessError::SettleValidationOverflow)?;
            }
            SettleOp::Sub(amt) => {
                if op_type == 2 {
                    return Err(ProcessError::InvalidOrderOfSettles)?;
                }
                if let Some(player) = game_state
                    .players
                    .iter_mut()
                    .find(|p| p.position.eq(&settle.position))
                {
                    player.balance = player
                        .balance
                        .checked_sub(amt)
                        .ok_or(ProcessError::PlayerBalanceOverflow)?;
                } else {
                    return Err(ProcessError::InvalidSettlePlayerAddress)?;
                }
                sum = sum.checked_sub(i128::from(amt))
                    .ok_or(ProcessError::SettleValidationOverflow)?;
                op_type = 1;
            }
            SettleOp::Eject => {
                let idx = game_state
                    .players
                    .iter()
                    .position(|p| p.position.eq(&settle.position));
                if let Some(idx) = idx {
                    let player = game_state.players.remove(idx);
                    pays.push((player.addr, player.balance));
                } else {
                    return Err(ProcessError::InvalidSettlePlayerAddress)?;
                }
                op_type = 2;
            }
            SettleOp::AssignSlot(_identifier) => {
                unimplemented!()
            }
        }
    }

    sum = sum.checked_add(transfers.iter().map(|t| i128::from(t.amount)).sum::<i128>())
        .ok_or(ProcessError::SettleValidationOverflow)?;

    if sum != 0 {
        return Err(ProcessError::InvalidSettleAmounts)?;
    }

    // // Ensure all players' assets are greater than zero
    // for player in game_state.players.iter() {
    //     if player.balance == 0 {
    //         return Err(ProcessError::UnhandledEliminatedPlayer)?;
    //     }
    // }

    // Transfer tokens
    let transfer_source = TransferSource::try_new(
        system_program.clone(),
        token_program.clone(),
        stake_account.clone(),
        game_account.key.as_ref(),
        pda_account.clone(),
        program_id,
    )?;

    for (addr, amount) in pays.into_iter() {
        let receiver_ata = next_account_info(account_iter)?;
        validate_receiver_account(&addr, &game_state.token_mint, receiver_ata.key)?;
        transfer_source.transfer(receiver_ata, amount)?;
    }

    let recipient_state = RecipientState::unpack(&recipient_account.try_borrow_data()?)?;

    // Handle commission transfers
    for Transfer { slot_id, amount } in transfers {
        let slot_stake_account = next_account_info(account_iter)?;
        if let Some(slot) = recipient_state.slots.iter().find(|s| s.id == slot_id) {
            if slot_stake_account.key.eq(&slot.stake_addr) {
                transfer_source.transfer(slot_stake_account, amount)?;
            } else {
                return Err(ProcessError::InvalidSlotStakeAccount)?;
            }
        } else {
            return Err(ProcessError::InvalidSlotId)?;
        }
    }

    game_state.settle_version = next_settle_version;
    game_state.checkpoint = Box::new(checkpoint);
    GameState::pack(game_state, &mut game_account.try_borrow_mut_data()?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use solana_program_test::*;
}
