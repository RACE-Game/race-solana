#![allow(dead_code)]
use std::str::FromStr;

use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_associated_token_account::get_associated_token_address;
use spl_token::{instruction::transfer, state::Account};

use crate::error::ProcessError;

const NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";

pub fn is_native_mint(mint: &Pubkey) -> bool {
    mint.eq(&Pubkey::from_str(NATIVE_MINT).unwrap())
}

/// Validate if the receiver is owned by account.
/// For SPL token, the receiver must be an ATA of account for mint.
/// For SOL, the receiver must be account.
#[inline(never)]
pub fn validate_receiver(
    account_key: &Pubkey,
    mint: &Pubkey,
    receiver_key: &Pubkey,
) -> ProgramResult {
    if is_native_mint(mint) {
        if receiver_key.ne(&account_key) {
            msg!(
                "Invalid receiver, expected: {:?}, actual: {:?}",
                account_key,
                receiver_key
            );
        }
    } else {
        let ata = get_associated_token_address(account_key, mint);
        if receiver_key.ne(&ata) {
            msg!(
                "Invalid receiver, expected: {:?}, actual: {:?}",
                ata,
                receiver_key
            );
            return Err(ProcessError::InvalidReceiverAddress)?;
        }
    }
    Ok(())
}

#[inline(never)]
pub fn transfer_spl<'a>(
    source_account: AccountInfo<'a>,
    dest_account: AccountInfo<'a>,
    pda: AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    amount: Option<u64>,
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    if let Ok(source_state) = Account::unpack(&dest_account.try_borrow_data()?) {
        let amount = amount.unwrap_or_else(|| source_state.amount);
        let ix = transfer(
            token_program.key,
            source_account.key,
            dest_account.key,
            &pda.key,
            &[pda.key],
            amount,
        )?;

        msg!("Transfer {} SPL to {}", amount, dest_account.key);

        invoke_signed(
            &ix,
            &[source_account.clone(), dest_account.clone(), pda.clone()],
            signer_seeds,
        )?;
    } else {
        msg!("Receiver account {:?} not available", dest_account.key);
    }

    Ok(())
}

#[inline(never)]
pub fn transfer_sol<'a>(
    source_account: AccountInfo<'a>,
    dest_account: AccountInfo<'a>,
    amount: Option<u64>,
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let amount = amount.unwrap_or_else(|| source_account.lamports());
    let ix =
        solana_program::system_instruction::transfer(source_account.key, dest_account.key, amount);

    msg!("Transfer {} Lamports to {}", amount, dest_account.key);

    invoke_signed(
        &ix,
        &[source_account.clone(), dest_account.clone()],
        signer_seeds,
    )?;

    Ok(())
}

#[inline(never)]
pub fn general_transfer<'a>(
    source_account: &AccountInfo<'a>,
    dest_account: &AccountInfo<'a>,
    mint: &Pubkey,
    amount: Option<u64>,
    pda: &AccountInfo<'a>,
    signer_seeds: &[&[&[u8]]],
    token_program: &AccountInfo<'a>,
) -> ProgramResult {
    if is_native_mint(mint) {
        transfer_sol(source_account.to_owned(), dest_account.to_owned(), amount, signer_seeds)?;
    } else {
        transfer_spl(
            source_account.to_owned(),
            dest_account.to_owned(),
            pda.to_owned(),
            token_program,
            amount,
            signer_seeds,
        )?;
    }
    Ok(())
}

#[inline(never)]
pub fn pack_state_to_account<'a, T: BorshSerialize>(
    state: T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let new_data = borsh::to_vec(&state)?;
    let new_data_len = new_data.len();
    let old_data_len = account.data_len();

    msg!("Current data len: {}", old_data_len);
    msg!("New data len: {}", new_data_len);

    if new_data_len != account.data_len() {
        msg!(
            "Realloc account data, old size: {}, new size: {}",
            account.data_len(),
            new_data_len
        );
        account.realloc(new_data_len, false)?;

        // When the new data is bigger than the old data, we do realloc.
        // And check if more lamports are required for rent-exempt.
        if new_data_len > old_data_len {
            let rent = Rent::get()?;
            let new_minimum_balance = rent.minimum_balance(new_data_len);
            let lamports_diff = new_minimum_balance.saturating_sub(account.lamports());

            msg!(
                "Transfer {} lamports to make account rent-exempt({}).",
                lamports_diff,
                new_minimum_balance
            );
            if lamports_diff > 0 {
                invoke(
                    &system_instruction::transfer(payer.key, account.key, lamports_diff),
                    &[payer.clone(), account.clone(), system_program.clone()],
                )?;
            }
        }
    }

    account.try_borrow_mut_data()?.copy_from_slice(&new_data);

    Ok(())
}
