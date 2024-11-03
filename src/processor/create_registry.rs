use crate::state::RegistryState;
use crate::types::CreateRegistrationParams;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar
};

#[inline(never)]
pub fn process(
    _programe_id: &Pubkey,
    accounts: &[AccountInfo],
    params: CreateRegistrationParams,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let payer = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }


    let registry_state = RegistryState {
        is_initialized: true,
        is_private: params.is_private,
        size: params.size,
        owner: payer.key.clone(),
        games: Box::new(vec![]),
    };

    msg!("Account length: {}", registry_account.data_len());
    msg!("Account lamports: {}", registry_account.lamports());
    registry_account.try_borrow_mut_data()?.copy_from_slice(&borsh::to_vec(&registry_state)?);
    msg!("Account updated");

    let rent = Rent::get()?;
    if !rent.is_exempt(registry_account.lamports(), registry_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_state_size() {
        let st = RegistryState {
            is_initialized: true,
            is_private: false,
            size: 100,
            owner: Pubkey::new_unique(),
            games: Box::new(vec![])
        };

        let bs = borsh::to_vec(&st).unwrap();
        println!("length of empty RegistryState: {}", bs.len())
    }
}
