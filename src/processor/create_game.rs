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

use crate::{state::{GameState, PlayerJoin}, constants::GAME_ACCOUNT_LEN};
use crate::types::CreateGameAccountParams;
use spl_token::{
    instruction::{set_authority, AuthorityType},
    state::Mint,
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

    let token_state = Mint::unpack_unchecked(&token_account.data.borrow())?;

    if !token_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    let (pda, _bump_seed) = Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

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

    let game_state = GameState {
        is_initialized: true,
        title: params.title,
        // TODO: invalid bundle account
        bundle_addr: *bundle_account.key,
        // TODO: use user's stake_account from client
        stake_account: *stake_account.key,
        // TODO: invalid owner
        owner: payer.key.clone(),
        min_deposit: params.min_deposit,
        max_deposit: params.max_deposit,
        transactor_addr: None,
        token_mint: *token_account.key,
        access_version: 0,
        settle_version: 0,
        max_players: params.max_players,
        // TODO: check if data exceeds max len
        data_len: params.data.len() as u32,
        data: Box::new(params.data),
        players: Box::new(Vec::<PlayerJoin>::with_capacity(
            params.max_players as usize,
        )),
        servers: Default::default(),
        unlock_time: None,
        votes: Default::default(),
    };

    if game_account.data_len() != GAME_ACCOUNT_LEN {
        return Err(ProgramError::AccountDataTooSmall);
    }

    GameState::pack(game_state, &mut game_account.try_borrow_mut_data()?)?;

    msg!("Created game account: {:?}", game_account.key);

    Ok(())
}
