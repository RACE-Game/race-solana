//! Settle game result
//!
//! Transfer the game assets between players, eject and pay leaving players.
//! This instruction is only available for current game transactor.
//!
//! Settles must be validated:
//! 1. All changes are sum up to zero.
//! 2. Player without assets must be ejected.

use crate::state::{Bonus, RecipientState};
use crate::types::{Award, SettleParams, Transfer};
use crate::{error::ProcessError, state::GameState};
use borsh::BorshDeserialize;
use solana_program::program::invoke_signed;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::instruction::close_account;

use super::misc::{general_transfer, pack_state_to_account, validate_receiver};

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: SettleParams,
) -> ProgramResult {
    let SettleParams {
        settles,
        transfers,
        awards,
        checkpoint,
        settle_version,
        next_settle_version,
        entry_lock,
        reset,
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

    if next_settle_version <= game_state.settle_version {
        msg!("Invalid next_settle = {}");
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
            None => return Err(ProcessError::InvalidSettlePlayerId)?,
        };

        pays.push((player.addr, settle.amount));
        if settle.eject {
            game_state
                .players
                .retain(|p| p.access_version != settle.player_id);
        }
    }

    let (_, bump_seed) = Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

    for (addr, amount) in pays.into_iter() {
        let receiver = next_account_info(account_iter)?;
        validate_receiver(&addr, &game_state.token_mint, &receiver.key)?;
        general_transfer(
            stake_account,
            &receiver,
            &game_state.token_mint,
            Some(amount),
            pda_account,
            &[&[game_account.key.as_ref(), &[bump_seed]]],
            token_program,
        )?;
    }

    // Handle commission transfers
    for Transfer { slot_id, amount } in *transfers {
        let slot_stake_account = next_account_info(account_iter)?;
        if let Some(slot) = recipient_state.slots.iter().find(|s| s.id == slot_id) {
            if slot_stake_account.key.eq(&slot.stake_addr) {
                general_transfer(
                    stake_account,
                    &slot_stake_account,
                    &game_state.token_mint,
                    Some(amount),
                    pda_account,
                    &[&[game_account.key.as_ref(), &[bump_seed]]],
                    token_program,
                )?;
            } else {
                return Err(ProcessError::InvalidSlotStakeAccount)?;
            }
        } else {
            return Err(ProcessError::InvalidSlotId)?;
        }
    }

    for Award {
        bonus_identifier,
        player_id,
    } in *awards
    {
        let bonuses: Vec<&Bonus> = game_state
            .bonuses
            .iter()
            .filter(|b| b.identifier.eq(&bonus_identifier))
            .collect();

        for bonus in bonuses.iter() {
            let bonus_account = next_account_info(account_iter)?;

            if bonus.stake_addr.ne(&bonus_account.key) {
                return Err(ProcessError::InvalidAwardIdentifier)?;
            }

            let receiver_account = next_account_info(account_iter)?;
            let player = match game_state
                .players
                .iter_mut()
                .find(|p| p.access_version == player_id)
            {
                Some(p) => p,
                None => return Err(ProcessError::InvalidAwardPlayerId)?,
            };

            validate_receiver(&player.addr, &bonus.token_addr, receiver_account.key)?;

            general_transfer(
                bonus_account,
                receiver_account,
                &bonus.token_addr,
                None, // Always transfer whole amount
                pda_account,
                &[&[game_account.key.as_ref()]],
                token_program,
            )?;

            let close_ix = close_account(token_program.key, bonus_account.key, transactor_account.key, pda_account.key, &[])?;

            invoke_signed(
                &close_ix,
                &[
                    bonus_account.clone(),
                    transactor_account.clone(),
                    pda_account.clone(),
                ],
                &[&[game_account.key.as_ref(), &[bump_seed]]]
            )?;
        }
    }

    game_state
        .deposits
        .retain(|d| d.settle_version >= game_state.settle_version);
    game_state.settle_version = next_settle_version;
    game_state.checkpoint = checkpoint;
    if let Some(entry_lock) = entry_lock {
        game_state.entry_lock = entry_lock;
    }

    if reset {
        game_state.players.clear();
        game_state.deposits.clear();
    }

    pack_state_to_account(
        game_state,
        &game_account,
        &transactor_account,
        &system_program,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use solana_program_test::*;
}
