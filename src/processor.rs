use crate::instruction::RaceInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

mod close_game;
mod create_game;
mod create_profile;
mod create_registry;
mod join;
mod misc;
mod publish_game;
mod register_game;
mod register_server;
mod serve;
mod settle;
mod unregister_game;
mod vote;
mod create_recipient;
mod assign_recipient;
mod recipient_claim;
mod deposit;
mod attach_bonus;
mod reject_deposits;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let instruction = RaceInstruction::unpack(instruction_data)?;

    let result = match instruction {

        RaceInstruction::CreateGameAccount { params } => {
            msg!("Create a game");
            create_game::process(program_id, accounts, params)
        }
        RaceInstruction::JoinGame { params } => {
            msg!("Player joins game");
            join::process(program_id, accounts, params)
        }
        RaceInstruction::CreateRegistry { params } => {
            msg!("Create a game center for registering games");
            create_registry::process(program_id, accounts, params)
        }
        RaceInstruction::CloseGameAccount => {
            msg!("Close a game account on chain");
            close_game::process(program_id, accounts)
        }
        RaceInstruction::CreatePlayerProfile { params } => {
            msg!("Create a player profile on chain");
            create_profile::process(program_id, accounts, params)
        }
        RaceInstruction::RegisterServer { params } => {
            msg!("Create a server account on chain");
            register_server::process(program_id, accounts, params)
        }
        RaceInstruction::Settle { params } => {
            msg!("Settle game");
            settle::process(program_id, accounts, params)
        }
        RaceInstruction::Vote { params } => {
            msg!("Vote");
            vote::process(program_id, accounts, params)
        }
        RaceInstruction::ServeGame { params } => {
            msg!("Server joins a game");
            serve::process(program_id, accounts, params)
        }
        RaceInstruction::RegisterGame => {
            msg!("Register a game");
            register_game::process(program_id, accounts)
        }
        RaceInstruction::UnregisterGame => {
            msg!("Unregister a game");
            unregister_game::process(program_id, accounts)
        }
        RaceInstruction::PublishGame { params } => {
            msg!("Publish a game as NFT");
            publish_game::process(program_id, accounts, params)
        }
        RaceInstruction::CreateRecipient { params }=> {
            msg!("Create recipient");
            create_recipient::process(program_id, accounts, *params)
        }
        RaceInstruction::AssignRecipient { params }=> {
            msg!("Assign recipient");
            assign_recipient::process(program_id, accounts, params)
        }
        RaceInstruction::RecipientClaim => {
            msg!("Recipient claim");
            recipient_claim::process(program_id, accounts)
        }
        RaceInstruction::Deposit { params } => {
            msg!("Deposit");
            deposit::process(program_id, accounts, params)
        }
        RaceInstruction::AttachBonus { params } => {
            msg!("Attach bonus");
            attach_bonus::process(program_id, accounts, params)
        }
        RaceInstruction::RejectDeposits { params } => {
            msg!("Reject Deposits");
            reject_deposits::process(program_id, accounts, params)
        }
    };

    if let Err(ref e) = result {
        msg!("Error in contract: {:?}", e);
    }

    result
}
