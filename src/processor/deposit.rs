use crate::types::DepositParams;
use crate::{
    error::ProcessError,
    state::{EntryType, GameState, PlayerJoin},
};
///! Player joins a game (cash, sng or tourney)
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::rent::Rent,
    // system_instruction::transfer as system_transfer,
};
use spl_token::{
    instruction::{close_account, transfer},
    native_mint,
    state::Account,
};

#[inline(never)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: DepositParams) -> ProgramResult {
    Ok(())
}
