use {
    quasar_svm::{Account, Pubkey, QuasarSvm},
    solana_program_pack::Pack,
    spl_token_interface::state::{Account as TokenAccount, AccountState, Mint},
};

// ---------------------------------------------------------------------------
// SVM factories
// ---------------------------------------------------------------------------

pub fn svm_validate() -> QuasarSvm {
    let elf = std::fs::read("../../target/deploy/quasar_test_token_validate.so").unwrap();
    QuasarSvm::new()
        .with_token_program()
        .with_token_2022_program()
        .with_associated_token_program()
        .with_program(&quasar_test_token_validate::ID, &elf)
}

pub fn svm_init() -> QuasarSvm {
    let elf = std::fs::read("../../target/deploy/quasar_test_token_init.so").unwrap();
    QuasarSvm::new()
        .with_token_program()
        .with_token_2022_program()
        .with_associated_token_program()
        .with_program(&quasar_test_token_init::ID, &elf)
}

pub fn svm_cpi() -> QuasarSvm {
    let elf = std::fs::read("../../target/deploy/quasar_test_token_cpi.so").unwrap();
    QuasarSvm::new()
        .with_token_program()
        .with_token_2022_program()
        .with_associated_token_program()
        .with_program(&quasar_test_token_cpi::ID, &elf)
}

// ---------------------------------------------------------------------------
// Program IDs
// ---------------------------------------------------------------------------

pub fn spl_token_program_id() -> Pubkey {
    quasar_svm::SPL_TOKEN_PROGRAM_ID
}

pub fn token_2022_program_id() -> Pubkey {
    quasar_svm::SPL_TOKEN_2022_PROGRAM_ID
}

pub fn ata_program_id() -> Pubkey {
    quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID
}

// ---------------------------------------------------------------------------
// Account packing helpers
// ---------------------------------------------------------------------------

pub fn pack_token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> Vec<u8> {
    let token = TokenAccount {
        mint,
        owner,
        amount,
        delegate: None.into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };
    let mut data = vec![0u8; TokenAccount::LEN];
    Pack::pack(token, &mut data).unwrap();
    data
}

pub fn pack_token_account_with_delegate(
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
    delegate: Pubkey,
    delegated_amount: u64,
) -> Vec<u8> {
    let token = TokenAccount {
        mint,
        owner,
        amount,
        delegate: Some(delegate).into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount,
        close_authority: None.into(),
    };
    let mut data = vec![0u8; TokenAccount::LEN];
    Pack::pack(token, &mut data).unwrap();
    data
}

pub fn pack_mint_account(authority: Pubkey, decimals: u8) -> Vec<u8> {
    let mint = Mint {
        mint_authority: Some(authority).into(),
        supply: 1_000_000_000,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    let mut data = vec![0u8; Mint::LEN];
    Pack::pack(mint, &mut data).unwrap();
    data
}

pub fn pack_mint_account_with_freeze(
    authority: Pubkey,
    decimals: u8,
    freeze_authority: Pubkey,
) -> Vec<u8> {
    let mint = Mint {
        mint_authority: Some(authority).into(),
        supply: 1_000_000_000,
        decimals,
        is_initialized: true,
        freeze_authority: Some(freeze_authority).into(),
    };
    let mut data = vec![0u8; Mint::LEN];
    Pack::pack(mint, &mut data).unwrap();
    data
}

pub fn pack_uninitialized_token_account() -> Vec<u8> {
    vec![0u8; TokenAccount::LEN]
}

pub fn pack_uninitialized_mint_account() -> Vec<u8> {
    vec![0u8; Mint::LEN]
}

// ---------------------------------------------------------------------------
// Account constructors
// ---------------------------------------------------------------------------

pub fn token_account(
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
    token_program: Pubkey,
) -> Account {
    Account {
        lamports: 1_000_000,
        data: pack_token_account(mint, owner, amount),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn mint_account(authority: Pubkey, decimals: u8, token_program: Pubkey) -> Account {
    Account {
        lamports: 1_000_000,
        data: pack_mint_account(authority, decimals),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn mint_account_with_freeze(
    authority: Pubkey,
    decimals: u8,
    freeze_authority: Pubkey,
    token_program: Pubkey,
) -> Account {
    Account {
        lamports: 1_000_000,
        data: pack_mint_account_with_freeze(authority, decimals, freeze_authority),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn signer_account() -> Account {
    Account::new(1_000_000, 0, &Pubkey::default())
}

pub fn rich_signer_account() -> Account {
    Account::new(100_000_000_000, 0, &Pubkey::default())
}

pub fn system_account(lamports: u64) -> Account {
    Account::new(lamports, 0, &quasar_svm::system_program::ID)
}

pub fn empty_account_for_init(space: usize) -> Account {
    Account::new(0, space, &quasar_svm::system_program::ID)
}
