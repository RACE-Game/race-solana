use crate::constants::REGISTRY_ACCOUNT_LEN;
use crate::error::ProcessError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Default, BorshDeserialize, BorshSerialize, Clone)]
pub struct GameReg {
    pub title: String, // max: 16 chars
    pub addr: Pubkey,
    pub bundle_addr: Pubkey,
    pub reg_time: u64,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct RegistryState {
    pub is_initialized: bool,
    pub is_private: bool,
    pub size: u16, // capacity of the registration center
    pub owner: Pubkey,
    pub games: Box<Vec<GameReg>>,
}

impl IsInitialized for RegistryState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for RegistryState {}
impl Pack for RegistryState {
    const LEN: usize = REGISTRY_ACCOUNT_LEN;

    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(mut src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self::deserialize(&mut src).map_err(|_| ProcessError::RegistryDeserializationFailed)?)
    }
}

#[cfg(test)]
mod tests {

    use solana_program::borsh::get_instance_packed_len;

    use super::*;

    fn make_registry_state() -> RegistryState {
        let state = RegistryState {
            is_initialized: true,
            is_private: false,
            size: 100,
            owner: Pubkey::new_unique(),
            games: Box::new(Vec::<GameReg>::with_capacity(100)),
        };

        state
    }
    #[test]
    fn test_registry_account_len() -> anyhow::Result<()> {
        let mut registry = make_registry_state();
        println!(
            "Registry account len {}",
            get_instance_packed_len(&registry)?
        );
        for i in 0..100 {
            let reg_game = GameReg {
                title: "gametitle_16_cha".to_string(),
                addr: Pubkey::new_unique(),
                reg_time: 1111111111111u64 + (i as u64),
                bundle_addr: Pubkey::new_unique(),
            };
            registry.games.push(reg_game);
        }
        let unpadded_len = get_instance_packed_len(&registry)?;
        println!(
            "Registry account aligned len {}",
            unpadded_len
        );
        assert!(unpadded_len <= REGISTRY_ACCOUNT_LEN);
        assert_eq!(unpadded_len, 9240);
        Ok(())
    }

    #[test]
    fn test_deser() -> anyhow::Result<()> {
        let state = make_registry_state();
        let mut buf = [0u8; RegistryState::LEN];
        RegistryState::pack(state.clone(), &mut buf)?;
        let deser = RegistryState::unpack(&buf)?;
        assert_eq!(deser, state);
        Ok(())
    }
}
