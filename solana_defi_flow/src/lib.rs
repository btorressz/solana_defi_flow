use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use pyth_sdk_solana::PriceFeed;

declare_id!("82oEV2b8foYx8zd25DFdXhFRayKZYgcTki7t6wRqk4s8");

/// Define constants for fee, thresholds, etc.
const FEE_BASIS_POINTS: u64 = 25; // 0.25% fee
const MAX_SLIPPAGE_BASIS_POINTS: u64 = 100; // 1% max slippage
const REWARD_MULTIPLIER: u64 = 10; // Reward multiplier for LPs
const MAX_FEE_BASIS_POINTS: u64 = 1000; // Maximum allowed fee (10%)

#[program]
pub mod solana_defi_flow {
    use super::*;

    /// Provide liquidity to a pool
    pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        // Transfer token A
        transfer_tokens(
            ctx.accounts.user_token_a.to_account_info(),
            ctx.accounts.pool_token_a.to_account_info(),
            ctx.accounts.user_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            amount_a,
        )?;

        // Transfer token B
        transfer_tokens(
            ctx.accounts.user_token_b.to_account_info(),
            ctx.accounts.pool_token_b.to_account_info(),
            ctx.accounts.user_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            amount_b,
        )?;

        // Mint reward tokens for LPs
        let reward_amount = calculate_rewards(amount_a, amount_b);
        mint_rewards(
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.user_reward_account.to_account_info(),
            ctx.accounts.reward_mint_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            reward_amount,
        )?;

        emit!(LiquidityAdded {
            user: *ctx.accounts.user_authority.key,
            token_a_amount: amount_a,
            token_b_amount: amount_b,
            reward_issued: reward_amount,
        });

        Ok(())
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, liquidity_amount: u64) -> Result<()> {
        transfer_tokens(
            ctx.accounts.user_lp_token.to_account_info(),
            ctx.accounts.pool_lp_token.to_account_info(),
            ctx.accounts.user_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            liquidity_amount,
        )?;

        emit!(LiquidityRemoved {
            user: *ctx.accounts.user_authority.key,
            lp_tokens_burned: liquidity_amount,
        });

        Ok(())
    }

    /// Execute a token swap using a DEX 
    pub fn swap_tokens(ctx: Context<SwapTokens>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    let fee = apply_fee(amount_in);
    let amount_after_fee = amount_in - fee;

    // Transfer input tokens to the swap pool
    transfer_tokens(
        ctx.accounts.user_input_token.to_account_info(),
        ctx.accounts.swap_pool_token.to_account_info(),
        ctx.accounts.user_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_after_fee,
    )?;

    emit!(SwapExecuted {
        user: ctx.accounts.user_authority.key(),
        token_in: ctx.accounts.user_input_token.key(),
        token_out: ctx.accounts.user_output_token.key(),
        amount_in: amount_in,
        amount_out: min_amount_out,
    });

    Ok(())
}

    /// Stake LP tokens in a vault
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        transfer_tokens(
            ctx.accounts.user_lp_token.to_account_info(),
            ctx.accounts.vault_lp_token.to_account_info(),
            ctx.accounts.user_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            amount,
        )?;

        let user_vault = &mut ctx.accounts.user_vault;
        user_vault.staked_amount += amount;

        emit!(TokensStaked {
            user: *ctx.accounts.user_authority.key,
            staked_amount: amount,
        });

        Ok(())
    }

    /// Adjust fees dynamically based on market conditions
    pub fn adjust_fee(ctx: Context<UpdateParameters>, market_volatility: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;

        if market_volatility > 50 {
            config.fee_basis_points = 50; // 0.5% fee
        } else {
            config.fee_basis_points = 10; // 0.1% fee
        }

        emit!(FeeAdjusted {
            new_fee_basis_points: config.fee_basis_points,
        });

        Ok(())
    }

    /// Mitigate impermanent loss based on market volatility
    pub fn mitigate_impermanent_loss(ctx: Context<RebalanceLiquidity>, price_threshold: u64) -> Result<()> {
        let current_price = get_price(&ctx.accounts.price_feed)?;
        if current_price > price_threshold {
            msg!("Mitigating impermanent loss. Price: {}", current_price);
        }

        Ok(())
    }
}

#[event]
pub struct LiquidityAdded {
    pub user: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub reward_issued: u64,
}

#[event]
pub struct LiquidityRemoved {
    pub user: Pubkey,
    pub lp_tokens_burned: u64,
}

#[event]
pub struct SwapExecuted {
    pub user: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
}

#[event]
pub struct TokensStaked {
    pub user: Pubkey,
    pub staked_amount: u64,
}

#[event]
pub struct FeeAdjusted {
    pub new_fee_basis_points: u64,
}

#[derive(Accounts)]
pub struct ProvideLiquidity<'info> {
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_reward_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub reward_mint_authority: AccountInfo<'info>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_lp_token: Account<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapTokens<'info> {
    #[account(mut)]
    pub user_input_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub swap_pool_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_output_token: Account<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_lp_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_vault: Account<'info, Vault>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateParameters<'info> {
    #[account(mut)]
    pub config: Account<'info, Config>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RebalanceLiquidity<'info> {
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    pub price_feed: AccountInfo<'info>, // Pyth price feed
}

#[account]
pub struct Vault {
    pub staked_amount: u64,
}

#[account]
pub struct Config {
    pub fee_basis_points: u64,
}

/// Helper function to transfer tokens
fn transfer_tokens<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = token::Transfer {
        from,
        to,
        authority,
    };
    let cpi_context = CpiContext::new(token_program, cpi_accounts);
    token::transfer(cpi_context, amount)
}

/// Helper function to calculate rewards
fn calculate_rewards(amount_a: u64, amount_b: u64) -> u64 {
    (amount_a + amount_b) * REWARD_MULTIPLIER / 1000
}

/// Helper function to mint rewards
fn mint_rewards<'a: 'b + 'c + 'd, 'b: 'a, 'c: 'a, 'd: 'a>(
    mint: AccountInfo<'a>,
    to: AccountInfo<'b>,
    authority: AccountInfo<'c>,
    token_program: AccountInfo<'d>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = token::MintTo {
        mint,
        to,
        authority,
    };
    let cpi_context = CpiContext::new(token_program, cpi_accounts);
    token::mint_to(cpi_context, amount)
}
/// Helper function to apply a fee
fn apply_fee(amount: u64) -> u64 {
    amount * FEE_BASIS_POINTS / 10_000
}

/// Helper function to get the current price from a Pyth price feed
fn get_price(price_feed: &AccountInfo) -> Result<u64> {
    let price_data = price_feed.try_borrow_data()?;
    let price = PriceFeed::deserialize(&mut price_data.as_ref())?.get_price_unchecked();
    Ok(price.price as u64)
}
