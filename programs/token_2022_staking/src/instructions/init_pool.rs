use {
    crate::{state::*, utils::*},
    anchor_lang::prelude::*,
    anchor_spl::token_interface,
    std::mem::size_of,
};

pub fn handler(ctx: Context<InitializePool>) -> Result<()> {
    check_token_program(ctx.accounts.token_program.key());

    // initialize pool state
    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.bump = ctx.bumps.pool_state;
    pool_state.amount = 0;
    pool_state.vault_bump = ctx.bumps.token_vault;
    pool_state.vault_auth_bump = ctx.bumps.pool_authority;
    pool_state.token_mint = ctx.accounts.token_mint.key();
    pool_state.staking_token_mint = ctx.accounts.staking_token_mint.key();
    pool_state.vault_authority = ctx.accounts.pool_authority.key();

    msg!("Staking pool created!");

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    /// CHECK: PDA, auth over all token vaults
    #[account(
        seeds = [VAULT_AUTH_SEED.as_bytes()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>, // 所有池子的owner
    // pool state account
    #[account(
        init,
        seeds = [token_mint.key().as_ref(), STAKE_POOL_STATE_SEED.as_bytes()],
        bump,
        payer = payer,
        space = 8 + size_of::<PoolState>()
    )]
    pub pool_state: Account<'info, PoolState>, // 在PDA上创建账户，存储和池子有关的状态，比如质押的代币数量、质押的用户数量等
    // Mint of token
    #[account(
        mint::token_program = token_program,
        mint::authority = payer // 增加了 authority 限制，仅允许 owner 来创建池子
    )]
    pub token_mint: InterfaceAccount<'info, token_interface::Mint>, // 预计在此池子中质押的代币数量。每个代币都有一个唯一的池子
    // pool token account for Token Mint
    #[account(
        init,
        token::mint = token_mint,
        token::authority = pool_authority,
        token::token_program = token_program,
        // use token_mint, pool auth, and constant as seeds for token a vault
        seeds = [token_mint.key().as_ref(), pool_authority.key().as_ref(), VAULT_SEED.as_bytes()],
        bump,
        payer = payer,
    )]
    pub token_vault: InterfaceAccount<'info, token_interface::TokenAccount>,
    /*
        token_vault:
        所有质押在池子里的token都要存在这个账户里。
        token_vault 是一个 Token Account，存储了与 token_mint（代币铸造标识）相同类型的代币。
        Token Account: 在区块链上（例如 Solana），代币账户是用来存储某种类型代币的地方。每种代币都有一个唯一的 mint（铸造地址）作为其标识。
    */
    // Mint of staking token
    #[account(
        mut,
        mint::token_program = token_program
    )]
    // InterfaceAccount 是 AccountInfo 的 Wrapper
    pub staking_token_mint: InterfaceAccount<'info, token_interface::Mint>, // 在vault中质押的 Mint Account
    // payer, will pay for creation of pool vault
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
