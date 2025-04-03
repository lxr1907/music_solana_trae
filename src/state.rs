use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ExchangeState {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    // 兑换比例: 1 SOL = rate 个自定义 token
    pub rate: u64,
}

impl IsInitialized for ExchangeState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for ExchangeState {}

impl Pack for ExchangeState {
    const LEN: usize = 1 + 32 + 32 + 32 + 8; // bool + pubkey + pubkey + pubkey + u64
    
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut src_mut = src;
        ExchangeState::deserialize(&mut src_mut).map_err(|_| ProgramError::InvalidAccountData)
    }
    
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut dst_mut = dst;
        self.serialize(&mut dst_mut).unwrap();
    }
}