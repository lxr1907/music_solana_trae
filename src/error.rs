use solana_program::{program_error::ProgramError, decode_error::DecodeError};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum ExchangeError {
    #[error("无效的指令")]
    InvalidInstruction,
    
    #[error("非管理员账户")]
    NotAdmin,
    
    #[error("兑换比例必须大于零")]
    InvalidExchangeRate,
    
    #[error("SOL 余额不足")]
    InsufficientSolBalance,
    
    #[error("Token 余额不足")]
    InsufficientTokenBalance,

    #[error("算术运算溢出")]
    ArithmeticOverflow,
}

impl From<ExchangeError> for ProgramError {
    fn from(e: ExchangeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for ExchangeError {
    fn type_of() -> &'static str {
        "ExchangeError"
    }
}