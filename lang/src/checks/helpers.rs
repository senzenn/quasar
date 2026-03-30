//! Standalone validation helpers for manual `ParseAccounts` implementations.
//!
//! These functions wrap the same checks that `#[derive(Accounts)]` generates,
//! exposed as composable building blocks.

use {
    crate::prelude::{AccountView, Address, ProgramError},
    crate::utils::hint::unlikely,
};

/// Verify that the account is a transaction signer.
#[inline(always)]
pub fn require_signer(view: &AccountView) -> Result<(), ProgramError> {
    if unlikely(!view.is_signer()) {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Verify that the account is writable.
#[inline(always)]
pub fn require_writable(view: &AccountView) -> Result<(), ProgramError> {
    if unlikely(!view.is_writable()) {
        return Err(ProgramError::Immutable);
    }
    Ok(())
}

/// Verify that the account is executable.
#[inline(always)]
pub fn require_executable(view: &AccountView) -> Result<(), ProgramError> {
    if unlikely(!view.executable()) {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Verify that the account address matches an expected value.
#[inline(always)]
pub fn require_address(view: &AccountView, expected: &Address) -> Result<(), ProgramError> {
    if unlikely(!crate::keys_eq(view.address(), expected)) {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}
