use crate::processor::misc::pack_state_to_account;
use crate::state::{PlayerDeposit, RecipientState};
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

    let recipient_account = next_account_info(account_iter)?;

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

    msg!("Deserializing recipient state, data len: {}", recipient_account.data_len());

    let recipient_state = RecipientState::unpack(&recipient_account.try_borrow_data()?)?;

    if !recipient_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    drop(recipient_state);

    msg!("Deserializing game state, data len: {}", game_account.data_len());

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

    // Increase game access version
    game_state.access_version += 1;

    // Transfer player deposit to game stake account
    let temp_state = Account::unpack(&temp_account.try_borrow_data()?)?;

    let is_native_token = game_state.token_mint.ne(&native_mint::id());

    let account_to_receive_payment;

    match &game_state.entry_type {
        EntryType::Cash {
            min_deposit, max_deposit
        } => {
            if params.amount < *min_deposit || params.amount > *max_deposit {
                msg!(
                    "Invalid deposit amount: {}, min: {}, max: {}",
                    params.amount,
                    min_deposit,
                    max_deposit
                );
                return Err(ProcessError::InvalidPaymentParams)?;
            }
        },
        EntryType::Ticket { amount } => {
            if params.amount != *amount {
                msg!("Invalid payment amount: {}, ticket: {}",
                    params.amount, amount);

                return Err(ProcessError::InvalidPaymentParams)?;
            }

        },
        _ => { unimplemented!() }
    }

    if is_native_token {
        account_to_receive_payment = stake_account;
    } else {
        account_to_receive_payment = pda_account;
    }

    msg!("Handle payment, the account to receive payment is {}", account_to_receive_payment.key.to_string());
    if is_native_token {
        if temp_state.amount != params.amount {
            return Err(ProcessError::InvalidDeposit)?;
        }

        let transfer_ix = transfer(
            token_program.key,
            temp_account.key,
            account_to_receive_payment.key,
            payer_account.key,
            &[&payer_account.key],
            params.amount as u64,
        )?;

        invoke(
            &transfer_ix,
            &[
                temp_account.clone(),
                account_to_receive_payment.clone(),
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

        if account_to_receive_payment.key.ne(&pda) {
            return Err(ProcessError::InvalidPDA)?;
        }

        if temp_account.lamports() != params.amount {
            return Err(ProcessError::InvalidDeposit)?;
        }

        let close_temp_account_ix = close_account(
            token_program.key,
            temp_account.key,
            account_to_receive_payment.key,
            payer_account.key,
            &[&payer_account.key],
        )?;

        invoke(
            &close_temp_account_ix,
            &[
                temp_account.clone(),
                account_to_receive_payment.clone(),
                payer_account.clone(),
            ],
        )?;
    }

    msg!("Add player and its deposit to game state");

    // Player joins
    game_state.players.push(PlayerJoin {
        addr: payer_account.key.clone(),
        position,
        access_version: game_state.access_version,
        verify_key: params.verify_key,
    });

    game_state.deposits.push(PlayerDeposit {
        addr: payer_account.key.clone(),
        amount: params.amount,
        settle_version: params.settle_version
    });

    pack_state_to_account(game_state, &game_account, &player_account, &system_program)?;

    msg!(
        "Player {} joined the game {}",
        payer_account.key,
        game_account.key
    );

    Ok(())
}
