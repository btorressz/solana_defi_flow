# solana_defi_flow

# Solana DeFi Flow: Liquidity Pools, Token Swaps, and Staking 
(https://explorer.solana.com/address/82oEV2b8foYx8zd25DFdXhFRayKZYgcTki7t6wRqk4s8?cluster=devnet)

## Overview
Solana DeFi Flow is a decentralized finance (DeFi) program built on the Solana blockchain using the Anchor framework. It provides essential functionalities for liquidity management, token swapping, staking, and impermanent loss mitigation. The program incentivizes liquidity providers with reward tokens and supports dynamic fee adjustments based on market conditions.
This program integrates Pyth Network price feeds and SPL tokens for seamless interaction with the Solana DeFi ecosystem. I made a similar project in  solidity for eth for a client earlier this year and I wanted to translate the solidity code I had to rust  and implment a solana version to test my problem solving skills. I will explain the differences and similairtes of the eth smart contract and this solana program at the end.

## Core Functionalities

### Provide Liquidity
- Users deposit two tokens (e.g., Token A and Token B) into a liquidity pool.
- The program transfers these tokens from the user’s account into the pool.
- As a reward, the program calculates how much the user has contributed and mints reward tokens (using a reward multiplier) into the user's reward token account.
- An event is emitted to log details about the liquidity provision, including the user's public key, amounts deposited, and rewards issued.

### Remove Liquidity
- Users can withdraw liquidity by burning LP (Liquidity Provider) tokens they own.
- The program transfers the corresponding underlying assets (e.g., Token A and Token B) back to the user from the pool.
- An event is emitted to log details about the liquidity withdrawal, including the user’s public key and the number of LP tokens burned.

### Token Swaps
- Users can swap one token for another using a decentralized exchange (DEX).
- The program deducts a small fee (default: 0.25%) from the input token amount before the swap.
- It then transfers the tokens from the user to the swap pool.
- After the swap, an event is emitted to log details such as:
  - The user's public key.
  - The token being swapped (input token) and the token being received (output token).
  - The amounts involved.

### Staking
- Users can stake their LP tokens in a vault to earn additional rewards over time.
- The program:
  - Transfers the staked LP tokens into a vault account.
  - Tracks the staked amount in a user-specific vault account.
- An event is emitted to log the staked amount and the user’s public key.

### Dynamic Fee Adjustment
- The program can dynamically adjust transaction fees based on market volatility.
- It uses Pyth Network price feeds to monitor the market:
  - If volatility is high, the program increases the fee (e.g., 0.5%).
  - If volatility is low, the program reduces the fee (e.g., 0.1%).
- The updated fee percentage is stored in a configuration account.
- An event is emitted to log the new fee percentage.

### Impermanent Loss Mitigation
- The program protects liquidity providers from impermanent loss during volatile markets:
  - It monitors token prices using a Pyth price feed.
  - If a price crosses a certain threshold, the program can take protective actions (e.g., withdrawing liquidity or rebalancing the pool).

## How It Works 

### Helper Functions
- **Token Transfers** (`transfer_tokens`): Transfers SPL tokens between accounts (e.g., from the user to the pool, or from the pool back to the user).
- **Reward Calculation** (`calculate_rewards`): Calculates reward tokens for liquidity providers based on their contribution using a reward multiplier.
- **Minting Reward Tokens** (`mint_rewards`): Mints SPL reward tokens into the user’s reward token account.
- **Fee Application** (`apply_fee`): Deducts a protocol fee (e.g., 0.25%) from the input token amount during swaps or other transactions.
- **Fetching Prices** (`get_price`): Retrieves the current price of a token using Pyth Network, allowing the program to monitor the market and take actions like mitigating impermanent loss or adjusting fees dynamically.

### Accounts and Roles

#### Liquidity Provider Accounts
- **user_token_a** and **user_token_b**: The user's accounts for the two tokens being deposited into the pool.
- **pool_token_a** and **pool_token_b**: The pool's accounts for managing token reserves.

#### Reward Accounts
- **reward_mint**: The mint account for generating reward tokens.
- **user_reward_account**: The user's account for holding reward tokens.
- **reward_mint_authority**: The account authorized to mint reward tokens.

#### LP Tokens
- **user_lp_token**: The user's account for holding LP tokens.
- **pool_lp_token**: The pool's account for managing LP token reserves.

#### Vault Accounts
- **vault_lp_token**: The vault account where staked LP tokens are stored.
- **user_vault**: A user-specific account for tracking the amount of LP tokens staked.

#### Price Feeds
- **price_feed**: The account for a Pyth Network price feed, used to monitor token prices.

#### Configuration Accounts
- **config**: A configuration account used to store protocol settings, such as the fee percentage.

## Events
The program emits events for key actions, allowing clients or frontends to track user activity and protocol operations.

- **LiquidityAdded**: Logs details when a user adds liquidity (e.g., amounts deposited, rewards issued).
- **LiquidityRemoved**: Logs details when a user removes liquidity (e.g., LP tokens burned).
- **SwapExecuted**: Logs details of a token swap (e.g., tokens exchanged, amounts).
- **TokensStaked**: Logs details when a user stakes LP tokens in a vault.
- **FeeAdjusted**: Logs details of fee adjustments based on market volatility.

## Constants
- **FEE_BASIS_POINTS**: Default protocol fee for swaps (e.g., 25 = 0.25%).
- **MAX_SLIPPAGE_BASIS_POINTS**: Maximum allowable slippage for token swaps (e.g., 100 = 1%).
- **REWARD_MULTIPLIER**: Multiplier for calculating reward tokens issued to liquidity providers.
- **MAX_FEE_BASIS_POINTS**: Maximum allowable protocol fee (1000 = 10%).

## Real-World Use Case

### For Users:
- Add liquidity to earn rewards.
- Stake LP tokens to earn additional rewards over time.
- Swap tokens with a small fee.

### Possibilites For Protocols:
- Use the program to manage liquidity pools.
- Dynamically adjust fees and mitigate impermanent loss to ensure fair and sustainable operation.

  
## Solana Defi Flow vs https://github.com/btorressz/DefiFlow 

# Comparison: solana_defi_flow vs DefiFlow(eth) smart contract

## Similarities:
- **Core Logic**: Both handle liquidity management, token swaps, and impermanent loss mitigation. Users can provide/remove liquidity and swap tokens.
- **Fee Mechanism**: Both apply a fee on swaps, calculated in basis points (e.g., 0.25% in Solana).
- **Reward Distribution**: Liquidity providers (LPs) are rewarded, with Solana minting tokens for rewards.
- **Event Emissions**: Both emit events for key actions like adding/removing liquidity, performing swaps, and adjusting fees.
- **Dynamic Parameter Adjustment**: Both adjust fees dynamically, with Solana responding to market volatility.
- **Impermanent Loss Mitigation**: Both mitigate impermanent loss using price oracles (Pyth for Solana, Chainlink for Ethereum).

## Differences:
- **Language & Framework**: Solana uses Rust and the Anchor framework, while Ethereum uses Solidity and libraries like OpenZeppelin.
- **Token Standards**: Solana uses SPL tokens (via Anchor SPL), while Ethereum uses ERC-20 tokens.
- **Price Oracles**: Solana uses Pyth for real-time data; Ethereum uses Chainlink.
- **Transaction Execution Model**: Solana is faster and cheaper with Proof of History (PoH), while Ethereum relies on Proof of Stake (PoS) with higher gas fees.
- **Context & Accounts**: In Anchor, contexts are defined using structs for account management; in Solidity, state is managed internally within the contract.
- **Events Emission**: Anchor uses `emit!()` for events; Solidity uses `emit`.
- **Fee Adjustments**: Solana adjusts fees based on volatility thresholds, while Ethereum may adjust based on external factors or governance.
- **Account Management**: Solana checks permissions through user accounts and signatures; Ethereum uses modifiers like `onlyOwner`.

 ## NOTE : I enjoyed every second developing this project and would love feedback if neccessary :) 

 ## License 
 - This project is under the **MIT LICENSE**
 






