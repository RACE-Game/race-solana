use crate::types::DepositParams;
///! Player joins a game (cash, sng or tourney)
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

#[inline(never)]
pub fn process(_program_id: &Pubkey, _accounts: &[AccountInfo], _params: DepositParams) -> ProgramResult {
    Ok(())
}
