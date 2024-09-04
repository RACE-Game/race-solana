use mpl_token_metadata::{instructions::{CreateMasterEditionV3Builder, CreateMetadataAccountV3Builder}, types::DataV2};
use crate::types::PublishParams;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::mint_to,
    state::Mint,
};

#[inline(never)]
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: PublishParams,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;

    let mint_account = next_account_info(accounts_iter)?;

    let ata_account = next_account_info(accounts_iter)?;

    let metadata_pda = next_account_info(accounts_iter)?;

    let edition_pda = next_account_info(accounts_iter)?;

    let token_program = next_account_info(accounts_iter)?;

    let metaplex_program = next_account_info(accounts_iter)?;

    let sys_rent = next_account_info(accounts_iter)?;

    let system_program = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // TODO: lot of necessary checkes
    let mint_state = Mint::unpack_unchecked(&mint_account.data.borrow())?;
    if !mint_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // Mint 1 token to token account
    msg!("Token mint: {}", mint_account.key);
    msg!("Minting 1 token to account: {}", ata_account.key);
    invoke(
        &mint_to(
            &token_program.key,
            &mint_account.key,
            &ata_account.key,
            &payer.key,
            &[&payer.key],
            1,
        )?,
        &[
            mint_account.clone(),
            payer.clone(),
            ata_account.clone(),
            token_program.clone(),
            sys_rent.clone(),
        ],
    )?;

    msg!("Recording creators of NFT ...");
    let creator = vec![
        mpl_token_metadata::types::Creator {
            address: payer.key.clone(),
            verified: false,
            share: 100,
        },
        mpl_token_metadata::types::Creator {
            address: mint_account.key.clone(),
            verified: false,
            share: 0,
        },
    ];

    // Create metadata account
    msg!("Creating metadata account: {}", metadata_pda.key);
    // pub name: String,
    // pub symbol: String,
    // pub uri: String,
    // pub seller_fee_basis_points: u16,
    // pub creators: Option<Vec<Creator>>,
    // pub collection: Option<Collection>,
    // pub uses: Option<Uses>,

    let create_metadata_account_ix = CreateMetadataAccountV3Builder::new()
        .payer(payer.key.clone())
        .mint(mint_account.key.clone())
        .metadata(metadata_pda.key.clone())
        .update_authority(payer.key.clone(), true)
        .mint_authority(payer.key.clone())
        .is_mutable(false)
        .update_authority(payer.key.clone(), true)
        .data(DataV2 {
            name: params.name,
            symbol: params.symbol,
            uri: params.uri,
            seller_fee_basis_points: 0,
            creators: Some(creator),
            collection: None,
            uses: None,
        })
        .instruction();

    invoke(
        &create_metadata_account_ix,
        &[
            metadata_pda.clone(),
            mint_account.clone(),
            payer.clone(),
            payer.clone(),
            metaplex_program.clone(),
            token_program.clone(),
            system_program.clone(),
            sys_rent.clone(),
        ],
    )?;

    // Create master edition account
    // mint_authority and freeze_authority will be transfer to this account
    msg!("Creating master edition account: {}", edition_pda.key);
    let create_master_edition_account_ix = CreateMasterEditionV3Builder::new()
        .edition(edition_pda.key.clone())
        .mint(mint_account.key.clone())
        .mint_authority(payer.key.clone())
        .update_authority(payer.key.clone())
        .metadata(metadata_pda.key.clone())
        .payer(payer.key.clone())
        .max_supply(0)
        .instruction();

    invoke(
        &create_master_edition_account_ix,
        &[
            edition_pda.clone(),
            mint_account.clone(),
            payer.clone(),
            payer.clone(),
            metadata_pda.clone(),
            metaplex_program.clone(),
            token_program.clone(),
            system_program.clone(),
            sys_rent.clone(),
        ],
    )?;

    msg!("Minted NFT successfully");

    Ok(())
}
