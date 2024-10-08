use crate::{constants::SERVER_ACCOUNT_LEN, error::ProcessError};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[cfg_attr(test, derive(PartialEq, Clone))]
#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct ServerState {
    pub is_initialized: bool,
    pub addr: Pubkey,
    pub owner: Pubkey,
    pub endpoint: String, // max: 50 chars
}

impl IsInitialized for ServerState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for ServerState {}

impl Pack for ServerState {
    const LEN: usize = SERVER_ACCOUNT_LEN;

    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(mut src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self::deserialize(&mut src).map_err(|_| ProcessError::ServerDeserializationFailed)?)
    }
}

#[cfg(test)]
mod tests {
    use solana_program::borsh::get_instance_packed_len;

    use super::*;

    #[test]
    fn test_server_account_len() -> anyhow::Result<()> {
        let mut server = ServerState::default();
        server.addr = Pubkey::new_unique();
        server.owner = Pubkey::new_unique();
        server.endpoint = "https------------------------------".to_string();
        let unpadded_len = get_instance_packed_len(&server)?;
        println!("Server account len {}", unpadded_len);
        assert!(unpadded_len <= SERVER_ACCOUNT_LEN);
        assert_eq!(get_instance_packed_len(&server)?, 104);
        Ok(())
    }
}
