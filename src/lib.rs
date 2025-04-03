use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};

pub mod error;
pub mod processor;
pub mod state;

// 程序入口点
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("SOL-Token 兑换合约: 处理指令...");
    
    processor::Processor::process(program_id, accounts, instruction_data)
}