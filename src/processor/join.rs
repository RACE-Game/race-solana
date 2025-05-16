use crate::processor::misc::pack_state_to_account;
use crate::state::{DepositStatus, PlayerDeposit, RecipientState};
use crate::types::JoinParams;
use crate::{
    error::ProcessError,
    state::{EntryType, GameState, PlayerJoin},
};
use borsh::BorshDeserialize;
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
pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], params: JoinParams) -> ProgramResult {

    let account_iter = &mut accounts.into_iter();

    let payer_account = next_account_info(account_iter)?;

    let player_account = next_account_info(account_iter)?;

    let temp_account = next_account_info(account_iter)?;

    let game_account = next_account_info(account_iter)?;

    let mint_account = next_account_info(account_iter)?;

    let stake_account = next_account_info(account_iter)?;

    let recipient_account = next_account_info(account_iter)?;

    let _pda_account = next_account_info(account_iter)?;

    let token_program = next_account_info(account_iter)?;

    let system_program = next_account_info(account_iter)?;

    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let rent = Rent::default();

    if !rent.is_exempt(player_account.lamports(), player_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    if !rent.is_exempt(game_account.lamports(), game_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    msg!("Deserializing recipient state, data len: {}", recipient_account.data_len());

    let recipient_state = RecipientState::try_from_slice(&recipient_account.try_borrow_data()?)?;

    if !recipient_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    drop(recipient_state);

    msg!("Deserializing game state, data len: {}", game_account.data_len());

    let mut game_state = GameState::try_from_slice(&game_account.try_borrow_data()?)?;


    if game_state.settle_version < params.settle_version {
        return Err(ProcessError::InvalidSettleVersion)?;
    }

    if game_state.stake_account.ne(stake_account.key) {
        return Err(ProcessError::InvalidStakeAccount)?;
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

    let is_native_token = game_state.token_mint.eq(&native_mint::id());

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

    if !is_native_token {
        // For SPL tokens, use token program to transfer tokens
        let temp_state = Account::unpack(&temp_account.try_borrow_data()?)?;

        if temp_state.amount != params.amount {
            msg!("Required amount: {}, actual amount: {}", params.amount, temp_state.amount);
            return Err(ProcessError::InvalidDeposit)?;
        }

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
        // For native mint, just close the account, transfer its lamports to stake account
        if temp_account.lamports() != params.amount {
            msg!("Invalid deposit, required: {}, actual: {}", params.amount, temp_account.lamports());
            return Err(ProcessError::InvalidDeposit)?;
        }

        **(stake_account.try_borrow_mut_lamports()?) += temp_account.lamports();
        **(temp_account.try_borrow_mut_lamports()?) = 0;

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
        access_version: game_state.access_version,
        settle_version: params.settle_version,
        status: DepositStatus::Pending,
    });

    pack_state_to_account(game_state, &game_account, &player_account, &system_program)?;

    msg!(
        "Player {} joined game",
        payer_account.key,
    );

    Ok(())
}
