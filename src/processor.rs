use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    msg,
    program_pack::Pack,
};
use spl_token::instruction as token_instruction;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    error::ExchangeError,
    state::ExchangeState,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ExchangeInstruction {
    /// 初始化兑换合约
    /// 
    /// 账户列表:
    /// 0. `[signer]` 管理员账户
    /// 1. `[writable]` 兑换状态账户
    /// 2. `[]` Token mint 账户
    /// 3. `[writable]` 合约 Token 账户
    /// 4. `[]` Token 程序 ID
    /// 5. `[]` 系统程序 ID
    /// 6. `[]` Rent sysvar
    Initialize {
        // 兑换比例: 1 SOL = rate 个自定义 token
        rate: u64,
    },
    
    /// 更新兑换比例
    /// 
    /// 账户列表:
    /// 0. `[signer]` 管理员账户
    /// 1. `[writable]` 兑换状态账户
    UpdateRate {
        // 新的兑换比例
        new_rate: u64,
    },
    
    /// SOL 兑换为 Token
    /// 
    /// 账户列表:
    /// 0. `[signer]` 用户账户
    /// 1. `[]` 兑换状态账户
    /// 2. `[writable]` 合约 Token 账户
    /// 3. `[writable]` 用户 Token 账户
    /// 4. `[]` Token 程序 ID
    ExchangeSolToToken {
        // 要兑换的 SOL 数量 (lamports)
        amount: u64,
    },
    
    /// Token 兑换为 SOL
    /// 
    /// 账户列表:
    /// 0. `[signer]` 用户账户
    /// 1. `[]` 兑换状态账户
    /// 2. `[writable]` 合约 Token 账户
    /// 3. `[writable]` 用户 Token 账户
    /// 4. `[]` Token 程序 ID
    ExchangeTokenToSol {
        // 要兑换的 Token 数量
        amount: u64,
    },
}

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = ExchangeInstruction::try_from_slice(instruction_data)
            .map_err(|_| ExchangeError::InvalidInstruction)?;
            
        match instruction {
            ExchangeInstruction::Initialize { rate } => {
                Self::process_initialize(program_id, accounts, rate)
            },
            ExchangeInstruction::UpdateRate { new_rate } => {
                Self::process_update_rate(program_id, accounts, new_rate)
            },
            ExchangeInstruction::ExchangeSolToToken { amount } => {
                Self::process_exchange_sol_to_token(program_id, accounts, amount)
            },
            ExchangeInstruction::ExchangeTokenToSol { amount } => {
                Self::process_exchange_token_to_sol(program_id, accounts, amount)
            },
        }
    }
    
    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        rate: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let admin_info = next_account_info(account_info_iter)?;
        let exchange_state_info = next_account_info(account_info_iter)?;
        let token_mint_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        
        // 验证签名
        if !admin_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        // 验证兑换比例
        if rate == 0 {
            return Err(ExchangeError::InvalidExchangeRate.into());
        }
        
        // 创建兑换状态账户
        let rent = Rent::from_account_info(rent_info)?;
        let space = ExchangeState::LEN;
        let lamports = rent.minimum_balance(space);
        
        invoke(
            &system_instruction::create_account(
                admin_info.key,
                exchange_state_info.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[admin_info.clone(), exchange_state_info.clone()],
        )?;
        
        // 初始化兑换状态
        let exchange_state = ExchangeState {
            is_initialized: true,
            admin: *admin_info.key,
            token_mint: *token_mint_info.key,
            token_account: *token_account_info.key,
            rate,
        };
        
        ExchangeState::pack(exchange_state, &mut exchange_state_info.data.borrow_mut())?;
        
        msg!("兑换合约初始化成功，兑换比例: 1 SOL = {} 个自定义 token", rate);
        
        Ok(())
    }
    
    fn process_update_rate(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        new_rate: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let admin_info = next_account_info(account_info_iter)?;
        let exchange_state_info = next_account_info(account_info_iter)?;
        
        // 验证签名
        if !admin_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        // 验证兑换比例
        if new_rate == 0 {
            return Err(ExchangeError::InvalidExchangeRate.into());
        }
        
        // 验证程序拥有状态账户
        if exchange_state_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        // 获取兑换状态
        let mut exchange_state = ExchangeState::unpack(&exchange_state_info.data.borrow())?;
        
        // 验证管理员
        if exchange_state.admin != *admin_info.key {
            return Err(ExchangeError::NotAdmin.into());
        }
        
        // 更新兑换比例
        exchange_state.rate = new_rate;
        ExchangeState::pack(exchange_state, &mut exchange_state_info.data.borrow_mut())?;
        
        msg!("兑换比例已更新: 1 SOL = {} 个自定义 token", new_rate);
        
        Ok(())
    }
    
    fn process_exchange_sol_to_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let user_info = next_account_info(account_info_iter)?;
        let exchange_state_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let user_token_account_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        
        // 验证账户
        validate_accounts(
            program_id,
            user_info,
            exchange_state_info,
            token_account_info,
            &ExchangeState::unpack(&exchange_state_info.data.borrow())?,
        )?;
        
        // 获取兑换状态
        let exchange_state = ExchangeState::unpack(&exchange_state_info.data.borrow())?;
        
        // 计算要转移的 token 数量
        let token_amount = amount.checked_mul(exchange_state.rate).ok_or(ExchangeError::ArithmeticOverflow)?;
        
        // 转移 SOL 到程序
        invoke(
            &system_instruction::transfer(user_info.key, exchange_state_info.key, amount),
            &[user_info.clone(), exchange_state_info.clone()],
        )?;
        
        // 转移 token 到用户
        invoke(
            &token_instruction::transfer(
                token_program_info.key,
                token_account_info.key,
                user_token_account_info.key,
                exchange_state_info.key,
                &[],
                token_amount,
            )?,
            &[
                token_account_info.clone(),
                user_token_account_info.clone(),
                exchange_state_info.clone(),
                token_program_info.clone(),
            ],
        )?;
        
        msg!("兑换成功: {} SOL -> {} 个自定义 token", amount, token_amount);
        
        Ok(())
    }
    
    fn process_exchange_token_to_sol(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let user_info = next_account_info(account_info_iter)?;
        let exchange_state_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let user_token_account_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        
        // 验证账户
        validate_accounts(
            program_id,
            user_info,
            exchange_state_info,
            token_account_info,
            &ExchangeState::unpack(&exchange_state_info.data.borrow())?,
        )?;
        
        // 获取兑换状态
        let exchange_state = ExchangeState::unpack(&exchange_state_info.data.borrow())?;
        
        // 计算要转移的 SOL 数量
        let sol_amount = amount.checked_div(exchange_state.rate).ok_or(ExchangeError::ArithmeticOverflow)?;
        
        // 检查合约账户是否有足够的 SOL
        if **exchange_state_info.lamports.borrow() < sol_amount {
            return Err(ExchangeError::InsufficientSolBalance.into());
        }
        
        // 转移 token 到合约
        invoke(
            &token_instruction::transfer(
                token_program_info.key,
                user_token_account_info.key,
                token_account_info.key,
                user_info.key,
                &[],
                amount,
            )?,
            &[
                user_token_account_info.clone(),
                token_account_info.clone(),
                user_info.clone(),
                token_program_info.clone(),
            ],
        )?;
        
        // 转移 SOL 到用户
        **exchange_state_info.try_borrow_mut_lamports()? -= sol_amount;
        **user_info.try_borrow_mut_lamports()? += sol_amount;
        
        msg!("兑换成功: {} token -> {} SOL (汇率: 1 SOL = {} token)", amount, sol_amount, exchange_state.rate);
        
        Ok(())
    }
}

fn validate_accounts(
    program_id: &Pubkey,
    user_info: &AccountInfo,
    exchange_state_info: &AccountInfo,
    token_account_info: &AccountInfo,
    exchange_state: &ExchangeState,
) -> ProgramResult {
    // 验证签名
    if !user_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 验证程序拥有状态账户
    if exchange_state_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // 验证状态账户已初始化
    if !exchange_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    
    // 验证Token账户
    if token_account_info.key != &exchange_state.token_account {
        return Err(ProgramError::InvalidAccountData);
    }
    
    Ok(())
}