use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Music not found.")]
    MusicNotFound,
    #[msg("User has already purchased this music.")]
    AlreadyPurchased,
    #[msg("Invalid royalties, total must be 100.")]
    InvalidRoyalties,
    #[msg("Invalid royalty accounts.")]
    InvalidRoyaltyAccounts,
    #[msg("Invalid royalty account address.")]
    InvalidRoyaltyAccount,
    #[msg("Royalty account is not writable.")]
    AccountNotWritable,
    #[msg("Insufficient PLAY token balance")]
    InsufficientTokenBalance,
    #[msg("Insufficient funds, need 0.1 SOL")]
    InsufficientFunds,
}