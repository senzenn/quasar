use {
    quasar_lang::prelude::*,
    quasar_spl::{InterfaceAccount, Token, TokenCpi, TokenInterface},
};

#[derive(Accounts)]
pub struct CloseTokenAccountInterface<'info> {
    pub authority: &'info Signer,
    pub account: &'info mut InterfaceAccount<Token>,
    /// CHECK: destination may equal authority when the signer is closing to
    /// themselves.
    #[account(dup)]
    pub destination: &'info mut Signer,
    pub token_program: &'info Interface<TokenInterface>,
}

impl<'info> CloseTokenAccountInterface<'info> {
    #[inline(always)]
    pub fn handler(&self) -> Result<(), ProgramError> {
        self.token_program
            .close_account(self.account, self.destination, self.authority)
            .invoke()
    }
}
