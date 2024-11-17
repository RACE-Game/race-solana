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
use crate::types::{SettleParams, Transfer};
use crate::{error::ProcessError, state::GameState};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

use super::misc::{pack_state_to_account, validate_receiver_account, TransferSource};

#[inline(never)]
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
        entry_lock,
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

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;

    if game_state.settle_version != settle_version {
        return Err(ProcessError::InvalidSettleVersion)?;
    }

    let recipient_state = RecipientState::unpack(&recipient_account.try_borrow_data()?)?;

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

    for settle in settles.into_iter() {
        let player = match game_state
            .players
            .iter_mut()
            .find(|p| p.access_version == settle.player_id)
        {
            Some(p) => p,
            None => return Err(ProcessError::InvalidSettlePlayerAddress)?,
        };

        pays.push((player.addr, settle.amount));
        game_state.players.retain(|p| p.access_version != settle.player_id);
    }

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

    game_state.deposits.retain(|d| d.settle_version <= game_state.settle_version);
    game_state.settle_version = next_settle_version;
    game_state.checkpoint = Box::new(checkpoint);
    if let Some(entry_lock) = entry_lock {
        game_state.entry_lock = entry_lock;
    }

    pack_state_to_account(game_state, &game_account, &transactor_account, &system_program)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use solana_program_test::*;
}
