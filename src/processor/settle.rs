//! Settle game result
//!
//! Transfer the game assets between players, eject and pay leaving players.
//! This instruction is only available for current game transactor.
//!
//! Settles must be validated:
//! 1. All changes are sum up to zero.
//! 2. Player without assets must be ejected.

use crate::state::players;
use crate::state::{DepositStatus, RecipientState};
use crate::types::{Award, BalanceChange, Settle, SettleParams, Transfer};
use crate::{
    error::ProcessError,
    state::{GameState, PlayerBalance},
};
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
use spl_token::state::Account;

use super::misc::{general_transfer, is_native_mint, pack_state_to_account, validate_receiver};

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: SettleParams,
) -> ProgramResult {
    let SettleParams {
        settles,
        transfer,
        awards,
        checkpoint,
        // access_version,
        settle_version,
        next_settle_version,
        entry_lock,
        accept_deposits,
        ..
    } = params;

    let mut account_iter = accounts.iter();

    let transactor_account = next_account_info(&mut account_iter)?;

    let game_account = next_account_info(&mut account_iter)?;

    let players_reg_account = next_account_info(&mut account_iter)?;

    let stake_account = next_account_info(&mut account_iter)?;

    let pda_account = next_account_info(&mut account_iter)?;

    let recipient_account = next_account_info(&mut account_iter)?;

    let token_program = next_account_info(&mut account_iter)?;

    let system_program = next_account_info(&mut account_iter)?;

    if !transactor_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;

    msg!("Game state deserialized");

    if game_state.settle_version != settle_version {
        return Err(ProcessError::InvalidSettleVersion)?;
    }

    if next_settle_version <= game_state.settle_version {
        msg!("Invalid next_settle = {}");
        return Err(ProcessError::InvalidNextSettleVersion)?;
    }

    if stake_account.key.ne(&game_state.stake_account) {
        msg!("Stake account expected: {:?}", game_state.stake_account);
        msg!("Stake account given: {:?}", stake_account.key);
        return Err(ProcessError::InvalidStakeAccount)?;
    }

    let (pda, bump_seed) = Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

    if pda.ne(&pda_account.key) {
        return Err(ProcessError::InvalidPDA)?;
    }

    // msg!("Handle settles: {:?}", settles);

    handle_settles(
        &mut game_state,
        *settles,
        game_account,
        players_reg_account,
        stake_account,
        pda_account,
        bump_seed,
        token_program,
        &mut account_iter,
    )?;

    if let Some(transfer) = transfer {
        // msg!("Handle transfer: {:?}", transfer);

        handle_transfer(
            &game_state,
            transfer,
            game_account,
            stake_account,
            recipient_account,
            pda_account,
            bump_seed,
            token_program,
            &mut account_iter,
        )?;
    }

    // msg!("Handle bonuses: {:?}", awards);
    handle_bonuses(
        &mut game_state,
        *awards,
        game_account,
        players_reg_account,
        pda_account,
        transactor_account,
        bump_seed,
        token_program,
        &mut account_iter,
    )?;

    // msg!("Handle accepted deposits: {:?}", accept_deposits);
    for accept_deposit in *accept_deposits {
        if let Some(d) = game_state
            .deposits
            .iter_mut()
            .find(|d| d.access_version == accept_deposit)
        {
            // msg!("Mark accepted deposit: {}", d.access_version);
            d.status = DepositStatus::Accepted;
        }
    }

    game_state
        .deposits
        .retain(|d| matches!(d.status, DepositStatus::Pending | DepositStatus::Rejected));

    validate_balance(&game_state, &stake_account)?;
    // msg!("Balance validation passed");

    // msg!("Bump settle version to {}", next_settle_version);
    game_state.settle_version = next_settle_version;
    game_state.checkpoint = *checkpoint;
    if let Some(entry_lock) = entry_lock {
        // msg!("Update entry lock: {:?}", entry_lock);
        game_state.entry_lock = entry_lock;
    }

    players::set_versions(&mut players_reg_account.try_borrow_mut_data()?, game_state.access_version, game_state.settle_version)?;

    pack_state_to_account(
        game_state,
        &game_account,
        &transactor_account,
        &system_program,
    )?;

    Ok(())
}

#[inline(never)]
fn validate_balance<'a, 'b>(
    game_state: &'a GameState,
    stake_account: &'a AccountInfo<'b>,
) -> ProgramResult {
    let stake_amount = if is_native_mint(&game_state.token_mint) {
        stake_account.lamports()
    } else {
        let token_state = Account::unpack(&stake_account.try_borrow_data()?)?;
        token_state.amount
    };

    let balance_sum = game_state.balances.iter().map(|b| b.balance).sum::<u64>();
    let unhandled_deposit = game_state
        .deposits
        .iter()
        .filter(|d| matches!(d.status, DepositStatus::Pending | DepositStatus::Rejected))
        .map(|d| d.amount)
        .sum::<u64>();

    if !(stake_amount == balance_sum + unhandled_deposit) {
        msg!("Stake amount = {}, balance_sum + unhandled_deposit = {}", stake_amount, balance_sum + unhandled_deposit);
        Err(ProcessError::UnbalancedGameStake)?
    }
    Ok(())
}

#[inline(never)]
fn handle_settles<'a, 'b, 'c, I: Iterator<Item = &'a AccountInfo<'b>>>(
    game_state: &'c mut GameState,
    settles: Vec<Settle>,
    game_account: &'a AccountInfo<'b>,
    players_reg_account: &'a AccountInfo<'b>,
    stake_account: &'a AccountInfo<'b>,
    pda_account: &'a AccountInfo<'b>,
    bump_seed: u8,
    token_program: &'a AccountInfo<'b>,
    account_iter: &'c mut I,
) -> ProgramResult {
    let mut pays = vec![];

    for settle in settles.into_iter() {
        if let Some(player_balance) = game_state
            .balances
            .iter_mut()
            .find(|pb| pb.player_id == settle.player_id)
        {
            match settle.change {
                Some(BalanceChange::Add(amount)) => {
                    player_balance.balance += amount;
                }
                Some(BalanceChange::Sub(amount)) => {
                    player_balance.balance = player_balance
                        .balance
                        .checked_sub(amount)
                        .ok_or(ProcessError::InvalidSettleBalance)?;
                }
                None => (),
            }
        } else {
            match settle.change {
                Some(BalanceChange::Add(amount)) => game_state.balances.push(PlayerBalance {
                    player_id: settle.player_id,
                    balance: amount,
                }),
                Some(BalanceChange::Sub(_)) => {
                    return Err(ProcessError::InvalidSettleBalance)?;
                }
                None => (),
            }
        }

        game_state.balances.retain(|b| b.balance > 0);

        let mut indices_to_remove = vec![];
        if let Some((player_idx, player)) = players::get_player_by_id(&players_reg_account.try_borrow_data()?, settle.player_id)? {
            if settle.player_id != 0 && settle.amount > 0 {
                pays.push((player.addr, settle.amount));
            }
            if settle.eject {
                indices_to_remove.push(player_idx);
            }
        }

        for idx in indices_to_remove {
            players::remove_player_by_index(&mut players_reg_account.try_borrow_mut_data()?, idx)?;
        }
    }

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

    Ok(())
}

#[inline(never)]
fn handle_bonuses<'a, 'b, 'c, I: Iterator<Item = &'a AccountInfo<'b>>>(
    game_state: &'c mut GameState,
    awards: Vec<Award>,
    game_account: &'a AccountInfo<'b>,
    players_reg_account: &'a AccountInfo<'b>,
    pda_account: &'a AccountInfo<'b>,
    transactor_account: &'a AccountInfo<'b>,
    bump_seed: u8,
    token_program: &'a AccountInfo<'b>,
    account_iter: &'c mut I,
) -> ProgramResult {
    for Award {
        bonus_identifier,
        player_id,
    } in awards
    {
        for bonus in game_state.bonuses.iter() {
            if bonus.identifier.ne(&bonus_identifier) {
                continue;
            }

            let bonus_account = next_account_info(account_iter)?;
            let receiver_account = next_account_info(account_iter)?;

            if bonus.stake_addr.ne(&bonus_account.key) {
                return Err(ProcessError::InvalidAwardIdentifier)?;
            }


            let player = match players::get_player_by_id(&players_reg_account.try_borrow_data()?, player_id)? {
                Some((_, p)) => p,
                None => return Err(ProcessError::InvalidAwardPlayerId)?,
            };

            validate_receiver(&player.addr, &bonus.token_addr, receiver_account.key)?;

            general_transfer(
                bonus_account,
                receiver_account,
                &bonus.token_addr,
                None,
                pda_account,
                &[&[game_account.key.as_ref(), &[bump_seed]]],
                token_program,
            )?;

            let close_ix = close_account(
                token_program.key,
                bonus_account.key,
                transactor_account.key,
                pda_account.key,
                &[pda_account.key],
            )?;

            invoke_signed(
                &close_ix,
                &[
                    bonus_account.clone(),
                    transactor_account.clone(),
                    pda_account.clone(),
                ],
                &[&[game_account.key.as_ref(), &[bump_seed]]],
            )?;
        }
    }
    Ok(())
}

#[inline(never)]
fn handle_transfer<'a, 'b, 'c, I: Iterator<Item = &'a AccountInfo<'b>>>(
    game_state: &'c GameState,
    transfer: Transfer,
    game_account: &'a AccountInfo<'b>,
    stake_account: &'a AccountInfo<'b>,
    recipient_account: &'a AccountInfo<'b>,
    pda_account: &'a AccountInfo<'b>,
    bump_seed: u8,
    token_program: &'a AccountInfo<'b>,
    account_iter: &'c mut I,
) -> ProgramResult {
    let recipient_state = RecipientState::try_from_slice(&recipient_account.try_borrow_data()?)?;

    // Handle commission transfers
    let slot_stake_account = next_account_info(account_iter)?;
    if let Some(slot) = recipient_state
        .slots
        .iter()
        .find(|s| s.token_addr.eq(&game_state.token_mint))
    {
        if slot_stake_account.key.eq(&slot.stake_addr) {
            general_transfer(
                stake_account,
                &slot_stake_account,
                &game_state.token_mint,
                Some(transfer.amount),
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

    Ok(())
}
