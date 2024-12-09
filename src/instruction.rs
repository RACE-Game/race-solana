use crate::types::{
    AssignRecipientParams, AttachBonusParams, CreateGameAccountParams, CreatePlayerProfileParams,
    CreateRecipientParams, CreateRegistrationParams, DepositParams, JoinParams, PublishParams,
    RegisterServerParams, RejectDepositsParams, ServeParams, SettleParams, VoteParams,
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
    /// 7. `[]` The system program
    CreateGameAccount { params: CreateGameAccountParams },

    /// # [1] Close a new game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The account of game owner
    /// 1. `[writable]` The account of game account
    /// 2. `[writable]` The stake account of game
    /// 3. `[]` PDA account.
    /// 4. `[]` The account to receive tokens
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
    /// 3. `[]` The system program
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
    /// `[]` Every bonus account and the receiver account to receive bonus
    Settle { params: SettleParams },

    /// # [6] Vote
    ///
    /// Accounts expected:
    /// 0. `[signer]` The voter account, could be the wallet address of a server or a player.
    /// 1. `[writable]` The game account.
    /// 2. `[]` The votee account.
    /// 3. `[]` The system program
    Vote { params: VoteParams },

    /// # [7] Serve a game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer acount (the server itself)
    /// 1. `[writable]` The game account to be served
    /// 2. `[]` The server account
    /// 3. `[]` The system program
    ServeGame { params: ServeParams },

    /// # [8] Register a game to the registry
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer acount (game account onwer?)
    /// 1. `[writable]` The registry account
    /// 2. `[]` The game account to be registered
    /// 3. `[]` The system program
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
    /// 0. `[signer]` The payer account
    /// 1. `[]` The player account
    /// 1. `[writable]` The temp account
    /// 2. `[writable]` The game account
    /// 3. `[]` The mint account.
    /// 4. `[writable]` The stake account that holds players' buyin assets
    /// 5. `[]` The recipient account
    /// 6. `[writable]` The pda account
    /// 7. `[]` The SPL token program
    /// 8. `[]` The system program
    /// (Optional)9. `[]` Other account to receive the payment. For EntryType::Ticket
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
    CreateRecipient { params: Box<CreateRecipientParams> },

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

    /// # [15] Deposit tokens to a game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer account
    /// 1. `[]` The player account
    /// 2. `[writable]` The temp account
    /// 3. `[writable]` The game account
    /// 4. `[]` The mint account
    /// 5. `[writable]` The stake account that holds players' deposit assets
    /// 6. `[writable]` The pda account
    /// 7. `[]` The SPL token program
    /// 8. `[]` The system program
    Deposit { params: DepositParams },

    /// # [16] Attach a bonus to a game
    ///
    /// Accounts expected:
    /// 0. `[signer]` The payer account
    /// 1. `[writable]` The game account
    /// 2. `[]` The SPL token program
    /// 3. `[]` The system program
    /// Rest. `[writable]` The temp account for each bonuses
    AttachBonus { params: AttachBonusParams },

    /// #[17] Reject a deposit
    ///
    /// Accounts expected:
    /// 0. `[signer]` The transactor account
    /// 1. `[writable]` The game account
    /// 2. `[]` The stake account
    /// 3. `[]` The PDA from game account
    /// 4. `[]` The SPL token program
    /// 5. `[]` The system program
    /// Rest. `[]` The receiver for each rejected deposit
    RejectDeposits { params: RejectDepositsParams },
}

impl RaceInstruction {
    pub fn unpack(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(RaceInstruction::try_from_slice(src).unwrap())
    }
}
