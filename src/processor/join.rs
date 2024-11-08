use crate::processor::misc::pack_state_to_account;
use crate::types::JoinParams;
use crate::{
    error::ProcessError,
    state::{EntryType, GameState, PlayerJoin},
};
use borsh::BorshDeserialize;
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
};
use spl_token::{
    instruction::{close_account, transfer},
    native_mint,
    state::Account,
};

#[inline(never)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: JoinParams) -> ProgramResult {
    let account_iter = &mut accounts.into_iter();

    let payer_account = next_account_info(account_iter)?;

    let player_account = next_account_info(account_iter)?;

    let temp_account = next_account_info(account_iter)?;

    let game_account = next_account_info(account_iter)?;

    let mint_account = next_account_info(account_iter)?;

    let stake_account = next_account_info(account_iter)?;

    let pda_account = next_account_info(account_iter)?;

    let token_program = next_account_info(account_iter)?;

    let system_program = next_account_info(account_iter)?;

    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let rent = Rent::default();

    if !Rent::is_exempt(&rent, player_account.lamports(), player_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;

    if game_state.stake_account.ne(stake_account.key) {
        return Err(ProgramError::InvalidArgument);
    }

    if game_state.token_mint.ne(mint_account.key) {
        return Err(ProcessError::InvalidMint)?;
    }

    // 1. game already full?
    // 2. position within [0..=(len-1)]?
    // 3. player already joined?
    // 4. position already taken?
    if game_state.max_players as usize == game_state.players.len() {
        return Err(ProcessError::GameFullAlready)?;
    }

    if params.position >= game_state.max_players {
        return Err(ProcessError::InvalidPosition)?;
    }

    if game_state
        .players
        .iter()
        .any(|p| p.addr == *payer_account.key)
    {
        return Err(ProcessError::JoinedGameAlready)?;
    }

    let mut position = params.position;
    if game_state
        .players
        .iter()
        .any(|p| p.position == params.position)
    {
        if let Some(pos) = (0..game_state.max_players)
            .into_iter()
            .find(|&i| !game_state.players.iter().any(|p| p.position == i as u16))
        {
            position = pos;
        } else {
            return Err(ProcessError::PositionTakenAlready)?;
        }
    }

    msg!("Player position: {:?}", position);

    // TODO: Check game status?
    // if game_state.status != GameStatus::Open {
    //     return Err(DealerError::InvalidGameStatus)?;
    // }

    // Increase game access version
    game_state.access_version += 1;

    // Player joins
    game_state.players.push(PlayerJoin {
        addr: payer_account.key.clone(),
        balance: params.amount,
        position,
        access_version: game_state.access_version,
        verify_key: params.verify_key,
    });

    match &game_state.entry_type {
        EntryType::Cash {
            min_deposit,
            max_deposit,
        } => {
            // Check player's deposit
            if params.amount < *min_deposit || params.amount > *max_deposit {
                msg!(
                    "deposit: {}, min: {}, max: {}",
                    params.amount,
                    min_deposit,
                    max_deposit
                );
                return Err(ProcessError::InvalidDeposit)?;
            }
        }
        _ => unimplemented!(),
    }

    // Transfer player deposit to game stake account
    let temp_state = Account::unpack(&temp_account.try_borrow_data()?)?;

    if game_state.token_mint.ne(&native_mint::id()) {
        if temp_state.amount != params.amount {
            return Err(ProcessError::InvalidDeposit)?;
        }

        msg!("Transfer token to stake account.");
        let transfer_ix = transfer(
            token_program.key,
            temp_account.key,
            stake_account.key,
            payer_account.key,
            &[&payer_account.key],
            params.amount as u64,
        )?;

        invoke(
            &transfer_ix,
            &[
                temp_account.clone(),
                stake_account.clone(),
                payer_account.clone(),
                token_program.clone(),
            ],
        )?;

        msg!("Close temp account.");
        let close_temp_account_ix = close_account(
            token_program.key,
            temp_account.key,
            payer_account.key,
            payer_account.key,
            &[&payer_account.key],
        )?;

        invoke(
            &close_temp_account_ix,
            &[
                temp_account.clone(),
                payer_account.clone(),
                payer_account.clone(),
            ],
        )?;
    } else {
        // For native mint, just close the account
        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[game_account.key.as_ref()], program_id);

        if pda_account.key.ne(&pda) {
            return Err(ProcessError::InvalidPDA)?;
        }

        if temp_account.lamports() != params.amount {
            return Err(ProcessError::InvalidDeposit)?;
        }

        let close_temp_account_ix = close_account(
            token_program.key,
            temp_account.key,
            pda_account.key,
            payer_account.key,
            &[&payer_account.key],
        )?;

        invoke(
            &close_temp_account_ix,
            &[
                temp_account.clone(),
                pda_account.clone(),
                payer_account.clone(),
            ],
        )?;
    }

    msg!("Pack game state with {} players", game_state.players.len());

    pack_state_to_account(game_state, &game_account, &player_account, &system_program)?;

    msg!(
        "Player {} joined the game {}",
        payer_account.key,
        game_account.key
    );

    Ok(())
}
