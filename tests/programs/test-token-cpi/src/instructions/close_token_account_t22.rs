use {
    quasar_lang::prelude::*,
    quasar_spl::{Token2022, TokenCpi},
};

#[derive(Accounts)]
pub struct CloseTokenAccountT22<'info> {
    pub authority: &'info Signer,
    pub account: &'info mut Account<Token2022>,
    /// CHECK: destination may equal authority when the signer is closing to
    /// themselves.
    #[account(dup)]
    pub destination: &'info mut Signer,
    pub token_program: &'info Program<Token2022>,
}

impl<'info> CloseTokenAccountT22<'info> {
    #[inline(always)]
    pub fn handler(&self) -> Result<(), ProgramError> {
        self.token_program
            .close_account(self.account, self.destination, self.authority)
            .invoke()
    }
}
