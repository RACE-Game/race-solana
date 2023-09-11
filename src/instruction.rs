use crate::types::{
    CreateGameAccountParams, CreatePlayerProfileParams, CreateRegistrationParams, JoinParams,
    PublishParams, RegisterServerParams, SettleParams, VoteParams, ServeParams, CreateRecipientParams, AssignRecipientParams,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum RaceInstruction {
    /// # [0] Create a new game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The account of transactor
    /// 1. `[writable]` The game account, hold all necessary info about the game
    /// 2. `[writable]` The temp stake account
    /// 3. `[]` The mint account
    /// 4. `[]` The token program
    /// 5. `[]` The bundled data account
    /// 6. `[]` The recipient account
    CreateGameAccount { params: CreateGameAccountParams },

    /// # [1] Close a new game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The account of game owner
    /// 1. `[]` The account of game account
    /// 2. `[writable]` The game reg account
    /// 3. `[writable]` The stake account of game
    /// 4. `[]` PDA account.
    /// 5. `[]` Token program.
    CloseGameAccount,

    /// # [2] Create an on-chain "lobby" for game registration
    ///
    /// Accounts expected:
    /// 0. `[signer]` The account of game owner
    /// 1. `[writable]` The registry account
    CreateRegistry { params: CreateRegistrationParams },

    /// # [3] Create a player profile
    ///
    /// Accounts expected:
    /// 0. `[signer]` The owner of the player profile
    /// 1. `[]` The player profile account to be created
    /// 2. `[]` The pfp account
    CreatePlayerProfile { params: CreatePlayerProfileParams },

    /// # [4] Register (Create) a server profile
    ///
    /// Accounts expected:
    /// 0. `[signer]` The owner of the player profile
    /// 1. `[]` The server profile account to be created
    RegisterServer { params: RegisterServerParams },

    /// # [5] Settle game result
    ///
    /// Accounts expected:
    /// 0. `[signer]` The game transactor account
    /// 1. `[writable]` The game account
    /// 2. `[writable]` The stake account, must match the one in game account
    /// 3. `[]` PDA account
    /// 4. `[]` The recipient account
    /// 5. `[]` The token program
    /// 6. `[]` The system program
    /// Following:
    /// `[]` Every leaving players account, must be in the same order with Eject settles
    /// `[]` Every recipient slot accounts to receive transfer
    Settle { params: SettleParams },

    /// # [6] Vote
    ///
    /// Accounts expected:
    /// 0. `[signer]` The voter account, could be the wallet address of a server or a player.
    /// 1. `[writable]` The game account.
    /// 2. `[]` The votee account.
    Vote { params: VoteParams },

    /// # [7] Serve a game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer acount (the server itself)
    /// 1. `[writable]` The game account to be served
    /// 2. `[]` The server account
    ServeGame{ params: ServeParams },

    /// # [8] Register a game to the registry
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer acount (game account onwer?)
    /// 1. `[writable]` The registry account
    /// 2. `[]` The game account to be registered
    RegisterGame,

    /// # [9] Unregister a game to the registry
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer acount (game account onwer?)
    /// 1. `[writable]` The registry account
    /// 2. `[]` The game account to be unregistered
    UnregisterGame,

    /// # [10] Join a game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The player to join the game
    /// 1.
    /// 1. `[writable]` The temp account.
    /// 2. `[writable]` The game account
    /// 3. `[]` The mint account.
    /// 4. `[writable]` The stake account that holds players' buyin assets
    /// 5. `[writable]` The pda account
    /// 6. `[]` The SPL token program
    JoinGame { params: JoinParams },

    /// # [11] Publish a game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer account
    /// 1. `[]` The mint account
    /// 2. `[writable]` The ata account
    /// 3. `[]` The metadata PDA
    /// 4. `[]` The edition PDA
    /// 5. `[]` The token program
    /// 6. `[]` The metaplex program
    /// 7. `[]` The sys rent program
    /// 8. `[]` The system program
    PublishGame { params: PublishParams },

    /// # [12] Create recipient
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer account
    /// 1. `[]` The cap account
    /// 2. `[]` The recipient account
    /// 3. `[]` The token program
    /// 3+n. `[]` The Nth staking account for slots
    CreateRecipient { params: CreateRecipientParams },

    /// # [13] Assign recipient
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer account, should be the cap account of recipient
    /// 1. `[writable]` The recipient account
    /// 2. `[]` The account to assigned as the owner to a slot
    AssignRecipient { params: AssignRecipientParams },

    /// # [14] Recipient claim
    ///
    /// Accounts expected:
    /// 0. `[signer]` The fee payer
    /// 1. `[writable]` The recipient account
    /// 2. `[]` The PDA account as the owner of stake accounts
    /// 3. `[]` The token program
    /// 4. `[]` The system program
    /// Rest. `[]` The stake account followed by the corresponding ATA to receive tokens
    RecipientClaim,
}

impl RaceInstruction {
    pub fn unpack(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(RaceInstruction::try_from_slice(src).unwrap())
    }
}

#[cfg(test)]
mod tests {

    use crate::state::EntryType;

    use super::*;

    #[test]
    fn test_ser_create_game_account() -> anyhow::Result<()> {
        let nodata_ix = RaceInstruction::CreateGameAccount{
            params: CreateGameAccountParams {
                title: "test game".to_string(),
                entry_type: EntryType::Cash {
                    min_deposit: 10,
                    max_deposit: 20,
                },
                max_players: 10u16,
                data: vec![]
            }
        };

        let data_ix = RaceInstruction::CreateGameAccount{
            params: CreateGameAccountParams {
                title: "test game #2".to_string(),
                entry_type: EntryType::Cash {
                    min_deposit: 10,
                    max_deposit: 20,
                },
                max_players: 10u16,
                data: vec![1, 2, 3, 4],
            }
        };

        let nodata_ix_ser = nodata_ix.try_to_vec().unwrap();
        println!("No data ix {:?}", nodata_ix_ser);
        let nodata_bytes = [0, 9, 0, 0, 0, 116, 101, 115, 116, 32, 103, 97, 109, 101, 10, 0, 30, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(nodata_ix_ser, nodata_bytes);

        let data_ix_ser = data_ix.try_to_vec().unwrap();
        println!("Data ix {:?}", data_ix_ser);
        let data_bytes = [0, 12, 0, 0, 0, 116, 101, 115, 116, 32, 103, 97, 109, 101, 32, 35, 50, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4];
        assert_eq!(data_ix_ser, data_bytes);

        Ok(())
    }

    #[test]
    fn test_ser_create_recipient() -> anyhow::Result<()> {
        let data = [12, 1, 0, 0, 0, 0, 0, 198, 250, 122, 243, 190, 219, 173, 58, 61, 101, 243, 106, 171, 201, 116, 49, 177, 187, 228, 194, 210, 246, 224, 228, 124, 166, 2, 3, 69, 47, 93, 97, 153, 159, 158, 51, 166, 94, 248, 123, 51, 139, 8, 98, 46, 160, 255, 99, 63, 79, 190, 151, 195, 156, 10, 194, 43, 231, 62, 154, 159, 247, 15, 199, 1, 0, 0, 0, 1, 188, 53, 104, 156, 38, 144, 66, 219, 138, 121, 89, 91, 126, 38, 194, 57, 148, 14, 89, 226, 149, 9, 241, 111, 37, 230, 13, 67, 247, 153, 215, 162, 1, 0, 0, 228, 11, 84, 2, 0, 0, 0];
        let _ = RaceInstruction::unpack(&data).unwrap();
        Ok(())
    }

    #[test]
    fn test_deser_settle() -> anyhow::Result<()> {
        let data = [5, 1, 0, 0, 0, 188, 53, 104, 156, 38, 144, 66, 219, 138, 121, 89, 91, 126, 38, 194, 57, 148, 14, 89,
                    226, 149, 9, 241, 111, 37, 230, 13, 67, 247, 153, 215, 162, 2, 0, 0, 0, 0];
        let _ = RaceInstruction::unpack(&data).unwrap();
        Ok(())
    }

    #[test]
    fn test_ser_join() -> anyhow::Result<()> {
        let join_ix = RaceInstruction::JoinGame{
            params: JoinParams {
                amount: 1000u64,
                access_version: 0u64,
                position: 2u16,
                verify_key: "key0".into(),
            }
        };

        let join_ix_ser = join_ix.try_to_vec().unwrap();
        println!("join ix serialized {:?}", join_ix);
        let join_bytes = [10, 232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 4, 0, 0, 0, 107, 101, 121, 48];
        assert_eq!(join_ix_ser, join_bytes);

        Ok(())
    }
}
