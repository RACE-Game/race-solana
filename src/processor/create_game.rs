// use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

use crate::state::GameState;
use crate::{
    error::ProcessError,
    processor::misc::{is_native_mint, pack_state_to_account},
    state::EntryLock,
    types::CreateGameAccountParams,
};
use spl_token::{
    instruction::{set_authority, AuthorityType},
    state::{Account, Mint},
};

#[inline(never)]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: CreateGameAccountParams,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let game_account = next_account_info(accounts_iter)?;

    let stake_account = next_account_info(accounts_iter)?;

    let token_account = next_account_info(accounts_iter)?;

    let token_program = next_account_info(accounts_iter)?;

    let bundle_account = next_account_info(accounts_iter)?;

    let recipient_account = next_account_info(accounts_iter)?;

    let system_program = next_account_info(accounts_iter)?;

    if recipient_account.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }
    let recipient_addr = recipient_account.key.to_owned();

    if is_native_mint(&token_account.key) {
        // For SOL, use PDA as stake account.
        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

        if pda.ne(&stake_account.key) {
            msg!("For game account with native token, the stake account must be a PDA.");
            return Err(ProcessError::InvalidStakeAccount)?;
        }
    } else {
        // For SPL, use dedicated stake account.
        let token_state = Mint::unpack_unchecked(&token_account.data.borrow())?;
        if !token_state.is_initialized {
            return Err(ProcessError::InvalidTokenMint)?;
        }

        let stake_state = Account::unpack(&stake_account.data.borrow())?;
        if stake_state.mint.ne(&token_account.key) {
            return Err(ProcessError::InvalidStakeAccount)?;
        }

        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

        let set_authority_ix = set_authority(
            token_program.key,
            stake_account.key,
            Some(&pda),
            AuthorityType::AccountOwner,
            payer.key,
            &[&payer.key],
        )?;

        invoke(
            &set_authority_ix,
            &[stake_account.clone(), payer.clone(), token_program.clone()],
        )?;
    }

    let game_state = GameState {
        is_initialized: true,
        version: "0.2.6".into(),
        title: params.title,
        bundle_addr: *bundle_account.key,
        stake_account: *stake_account.key,
        owner: payer.key.clone(),
        transactor_addr: None,
        token_mint: *token_account.key,
        access_version: 0,
        settle_version: 0,
        max_players: params.max_players,
        data_len: params.data.len() as u32,
        data: Box::new(params.data),
        players: Default::default(),
        deposits: Default::default(),
        servers: Default::default(),
        unlock_time: None,
        votes: Default::default(),
        entry_type: params.entry_type,
        recipient_addr,
        checkpoint: Default::default(),
        entry_lock: EntryLock::Open,
        bonuses: Default::default(),
    };

    msg!("Created game account: {:?}", game_account.key);

    pack_state_to_account(game_state, &game_account, &payer, &system_program)?;

    Ok(())
}
