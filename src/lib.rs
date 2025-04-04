use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::token::MintTo; // 明确引入MintTo结构体

use solana_program::{
    program::{invoke},
    system_instruction,
};

declare_id!("83eMBGtHrS4oR6VjptrJdwVjidDjoAdokVxCi6dZQeZP");

#[program]
mod music_store {
    use super::*;
    // 初始化代币合约（仅创建Mint）
    pub fn initialize_token(ctx: Context<InitializeToken>) -> Result<()> {
        msg!("Token mint initialized");
        Ok(())
    }

    // 修改后的初始化用户账户（添加代币账户验证）
    pub fn initialize_buyer(ctx: Context<InitializeBuyer>) -> Result<()> {
        let buyer = &mut ctx.accounts.buyer;
        buyer.purchased_music_ids = vec![];

        // 验证用户代币账户余额
        let token_account = &ctx.accounts.user_token_account;
        require!(
            token_account.amount >= 1_000_000 * 10u64.pow(ctx.accounts.mint.decimals as u32),
            ErrorCode::InsufficientTokenBalance
        );

        msg!("Buyer initialized with sufficient PLAY tokens.");
        Ok(())
    }
    // 上传音乐（添加分成比例参数）
    pub fn upload_music(
        ctx: Context<UploadMusic>,
        music_id: u64,
        name: String,
        price: u64,
        royalties: Vec<(Pubkey, u8)>,
    ) -> Result<()> {
        let music = &mut ctx.accounts.music;

        // 验证分成比例总和为100%
        let total_percent: u8 = royalties.iter().map(|(_, p)| *p).sum();
        require!(total_percent == 100, ErrorCode::InvalidRoyalties);

        music.id = music_id;
        music.name = name;
        music.price = price;
        music.owner = *ctx.accounts.signer.key;
        music.royalties = royalties;

        msg!("Music uploaded: ID={}, Name={}, Price={}", music_id, music.name, music.price);
        Ok(())
    }

    // 购买音乐（添加分成转账逻辑）
    pub fn buy_music<'info>(ctx: Context<'_, '_, '_, 'info, BuyMusic<'info>>, music_id: u64) -> Result<()> {
        let music = &ctx.accounts.music;
        let buyer = &mut ctx.accounts.buyer;

        // 基础验证
        require!(music.id == music_id, ErrorCode::MusicNotFound);
        require!(!buyer.purchased_music_ids.contains(&music_id), ErrorCode::AlreadyPurchased);

        // 验证分账账户参数
        let remaining_accounts = ctx.remaining_accounts;
        require!(
            remaining_accounts.len() == music.royalties.len(),
            ErrorCode::InvalidRoyaltyAccounts
        );

        // 验证每个分账账户信息
        for (i, (royalty_address, _)) in music.royalties.iter().enumerate() {
            let account = &remaining_accounts[i];
            require!(
                account.key() == *royalty_address,
                ErrorCode::InvalidRoyaltyAccount
            );
            require!(account.is_writable, ErrorCode::AccountNotWritable);
        }

        let total_price = music.price;
        let mut total_transferred = 0;

        // 执行分账转账
        for (i, (royalty_address, percent)) in music.royalties.iter().enumerate() {
            let percent = *percent as u64;
            let mut share = (total_price * percent) / 100;

            // 处理余数（最后一个分账人获得剩余金额）
            if i == music.royalties.len() - 1 {
                share = total_price - total_transferred;
            } else {
                total_transferred += share;
            }

            // 创建转账指令
            let transfer_instruction = system_instruction::transfer(
                &ctx.accounts.signer.key(),
                royalty_address,
                share,
            );

            // 执行转账
            invoke(
                &transfer_instruction,
                &[
                    ctx.accounts.signer.to_account_info(),
                    remaining_accounts[i].clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        // 更新购买记录
        buyer.purchased_music_ids.push(music_id);
        msg!("Purchase successful. Transferred {} lamports", total_price);
        Ok(())
    }

    // 查询购买记录
    pub fn has_purchased(ctx: Context<HasPurchased>, music_id: u64) -> Result<bool> {
        let buyer = &ctx.accounts.buyer;
        Ok(buyer.purchased_music_ids.contains(&music_id))
    }

    // 添加代币购买方法
    pub fn buy_play_tokens(ctx: Context<BuyPlayTokens>) -> Result<()> {
        let amount_lamports = 100_000_000; // 0.1 SOL
        let tokens_to_mint = 100_000 * 10u64.pow(ctx.accounts.mint.decimals as u32);

        // 验证支付金额
        require!(
            ctx.accounts.payer.lamports() >= amount_lamports,
            ErrorCode::InsufficientFunds
        );

        // 执行SOL转账
        invoke(
            &system_instruction::transfer(
                ctx.accounts.payer.key,
                ctx.accounts.vault.key,
                amount_lamports,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 铸造代币给购买者
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts
        );

        token::mint_to(cpi_ctx, tokens_to_mint)?;

        msg!("Purchased {} PLAY tokens with 0.1 SOL", tokens_to_mint);
        Ok(())
    }

}
// 购买代币上下文
#[derive(Accounts)]
pub struct BuyPlayTokens<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub vault: SystemAccount<'info>, // 接收SOL的账户

    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = buyer_token_account.mint == mint.key()
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    // 使用PDA作为铸币权限
    #[account(
        seeds = [b"mint-auth"],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// 代币初始化上下文修改
#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = mint_authority, // 设置PDA为铸币权限
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    // PDA作为铸币权限
    #[account(
        seeds = [b"mint-auth"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
// 上传音乐的上下文
#[derive(Accounts)]
pub struct UploadMusic<'info> {
    #[account(init, payer = signer, space = 8 + 8 + 64 + 8 + 32)]
    pub music: Account<'info, Music>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 音乐账户（添加分成比例字段）
#[account]
pub struct Music {
    pub id: u64,
    pub name: String,
    pub price: u64,
    pub owner: Pubkey,
    pub royalties: Vec<(Pubkey, u8)>, // 分成人列表（地址，分成比例）
}

// 修改后的购买上下文（添加system_program）
#[derive(Accounts)]
pub struct BuyMusic<'info> {
    #[account(mut)]
    pub music: Account<'info, Music>,
    #[account(mut)]
    pub buyer: Account<'info, Buyer>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 查询购买记录的上下文
#[derive(Accounts)]
pub struct HasPurchased<'info> {
    pub buyer: Account<'info, Buyer>,
}


// 修改后的初始化用户上下文（添加代币验证）
#[derive(Accounts)]
pub struct InitializeBuyer<'info> {
    #[account(init, payer = signer, space = 8 + 1024)]
    pub buyer: Account<'info, Buyer>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>, // 用户的PLAY代币账户
    pub mint: Account<'info, Mint>,                       // PLAY代币铸币账户
    pub system_program: Program<'info, System>,
}


// 用户账户
#[account]
pub struct Buyer {
    pub purchased_music_ids: Vec<u64>, // 用户购买的音乐 ID 列表
}

// 新增错误类型
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