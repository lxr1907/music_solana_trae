use anchor_lang::prelude::*;
use solana_program::{program::invoke, system_instruction};
// 新增引入
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("E2VamReTdVrFdHxNYnnscEfBQ3h5sg4FikPiWkSn2Cto");

#[program]
mod music_store {
    use super::*;

    // 初始化用户账户的功能已整合到buyMusic方法中

    pub fn upload_music(
        ctx: Context<UploadMusic>,
        music_id: u64,
        name: String,
        price: u64,
        beneficiary: Pubkey,
        bump: u8,
    ) -> Result<()> {
        let music = &mut ctx.accounts.music;

        // 设置Music结构体字段
        music.id = music_id;
        music.name = name.clone();
        music.price = price;
        music.owner = *ctx.accounts.signer.key;
        music.royalty = Royalty {
            address: beneficiary,
            percentage: 100,
        };
        music.bump = bump;

        msg!("Music uploaded: {} ({} lamports)", music.name, music.price);
        Ok(())
    }

    // 查询购买记录
    pub fn has_purchased(ctx: Context<HasPurchased>, music_id: u64) -> Result<bool> {
        let buyer = &ctx.accounts.buyer;
        Ok(buyer.purchased_music_ids.contains(&music_id))
    }
    // 新增 buy_music_token 方法
    pub fn buy_music_token(
        ctx: Context<BuyMusicToken>,
        music_id: u64,
    ) -> Result<()> {
        let music = &ctx.accounts.music;
        let buyer = &mut ctx.accounts.buyer;

        // 如果是新账户，初始化购买记录数组
        if buyer.purchased_music_ids.is_empty() {
            buyer.purchased_music_ids = vec![];
            msg!("Buyer initialized with PDA");
        }

        require!(
            !buyer.purchased_music_ids.contains(&music_id),
            ErrorCode::AlreadyPurchased
        );

        // 执行 token 转账：买家 token 账户 -> 音乐所有者 token 账户
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // 假设价格为音乐的原始价格，可根据需求调整
        token::transfer(cpi_ctx, music.price)?;

        buyer.purchased_music_ids.push(music_id);
        msg!("Purchased music {} for {} token units", music_id, music.price);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(music_id: u64)]
pub struct BuyMusic<'info> {
    #[account(mut, seeds = [b"music", music_id.to_be_bytes().as_ref()], bump = music.bump)]
    pub music: Account<'info, Music>,
    #[account(init_if_needed, payer = payer, space = 8 + 1024, seeds = [b"buyer", payer.key().as_ref()], bump)]
    pub buyer: Account<'info, Buyer>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 数据结构调整
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Royalty {
    pub address: Pubkey,
    pub percentage: u8,
}

#[account]
pub struct Music {
    pub id: u64,
    pub name: String,
    pub price: u64,
    pub owner: Pubkey,
    pub royalty: Royalty, // 修改为单个受益人
    pub bump: u8,         // 存储bump值
}

#[account]
pub struct Buyer {
    pub purchased_music_ids: Vec<u64>,
}

// InitializeBuyer结构体已移除，功能整合到BuyMusic结构体中

#[derive(Accounts)]
#[instruction(music_id: u64, name: String, price: u64, beneficiary: Pubkey, bump: u8)]
pub struct UploadMusic<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        seeds = [b"music", music_id.to_be_bytes().as_ref()],
        bump,
        payer = signer,
        space = 8 + 8 + (name.len() + 4) + 8 + 32 + 32 + 1 + 1 // discriminator, id, name, price, owner, royalty address, percentage, bump
    )]
    pub music: Account<'info, Music>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct HasPurchased<'info> {
    #[account(seeds = [b"buyer", signer.key().as_ref()], bump)]
    pub buyer: Account<'info, Buyer>,
    pub signer: Signer<'info>,
}

// 错误代码调整
#[error_code]
pub enum ErrorCode {
    #[msg("Music not found.")]
    MusicNotFound,
    #[msg("User has already purchased this music.")]
    AlreadyPurchased,
    #[msg("Invalid royalty accounts.")]
    InvalidRoyaltyAccounts,
    #[msg("Invalid royalty account address.")]
    InvalidRoyaltyAccount,
    #[msg("Royalty account is not writable.")]
    AccountNotWritable,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}

// 新增 BuyMusicToken 结构体
#[derive(Accounts)]
pub struct BuyMusicToken<'info> {
    #[account(mut, seeds = [b"music", music_id.to_be_bytes().as_ref()], bump = music.bump)]
    pub music: Account<'info, Music>,
    #[account(init_if_needed, payer = payer, space = 8 + 1024, seeds = [b"buyer", payer.key().as_ref()], bump)]
    pub buyer: Account<'info, Buyer>,
    #[account(mut)]
    pub payer: Signer<'info>,
    // 买家的 token 账户
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    // 音乐所有者的 token 账户
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// 删除 BuyMusic 结构体
// #[derive(Accounts)]
// #[instruction(music_id: u64)]
// pub struct BuyMusic<'info> {
//     #[account(mut, seeds = [b"music", music_id.to_be_bytes().as_ref()], bump = music.bump)]
//     pub music: Account<'info, Music>,
//     #[account(init_if_needed, payer = payer, space = 8 + 1024, seeds = [b"buyer", payer.key().as_ref()], bump)]
//     pub buyer: Account<'info, Buyer>,
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }
