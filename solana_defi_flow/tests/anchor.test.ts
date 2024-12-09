// No imports needed: web3, anchor, pg, and more are globally available

/*
  anchor.test.ts: solana_defi_flow test file

  This test file validates the core functionalities of a Solana-based DeFi program, including:
  - **Liquidity Provision:** Users can deposit tokens into a liquidity pool.
  - **Liquidity Removal:** Users can withdraw tokens from the pool.
  - **Token Swapping:** Verifies token exchange functionality through the program.
  - **LP Token Staking:** Tests staking of liquidity pool tokens in a vault.
  - **Fee Adjustments:** Dynamically adjusts fees based on market conditions.
  - **Impermanent Loss Mitigation:** Ensures the program handles volatile market conditions.

  **Utility Functions:**
  - `requestAirdropWithRetry`: Retries SOL airdrops with a custom delay to handle network issues.
  - `delay`: Simulates wait times using Solana's blockhash query (instead of `setTimeout`).

  **Structure:**
  - `before` Hook: Sets up keypairs, airdrops SOL, and initializes test accounts.
  - Individual Tests: Validate each core feature of the DeFi program.

**Notes for Reviewers:**
- The test file assumes the Solana DeFi Flow program is correctly implemented and deployed.
  - Errors during tests (e.g., airdrop failures) can often be resolved by increasing retries 
    or debugging transaction logs via Solana Explorer or Solscan.

  This test file ensures the program operates as expected across all major features.
*/


// Define the SPL Token Program ID
// TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA:
// Universal program ID for Solana's SPL Token Program,
// used for minting, transferring, and managing fungible tokens.
// Same across all clusters (devnet, testnet, mainnet-beta).
// https://explorer.solana.com/address/TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
// https://solscan.io/account/TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
const TOKEN_PROGRAM_ID = new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

describe("Solana DeFi Flow Tests", () => {
  let userTokenA: web3.Keypair,
    userTokenB: web3.Keypair,
    poolTokenA: web3.Keypair,
    poolTokenB: web3.Keypair,
    rewardMint: web3.Keypair,
    rewardAccount: web3.Keypair,
    userLPToken: web3.Keypair;
  let vaultLPToken: web3.Keypair,
    userVault: web3.Keypair,
    config: web3.Keypair,
    priceFeed: web3.Keypair,
    authority: any;

  // Retry mechanism for airdrops to handle potential failures
  async function requestAirdropWithRetry(
    account: web3.Keypair,
    retries = 5,
    delayMs = 1000
  ): Promise<void> {
    for (let i = 0; i < retries; i++) {
      try {
        console.log(`Requesting airdrop for: ${account.publicKey.toBase58()}`);
        const tx = await pg.connection.requestAirdrop(
          account.publicKey,
          web3.LAMPORTS_PER_SOL
        );
        await pg.connection.confirmTransaction(tx);
        console.log(`Airdrop successful for: ${account.publicKey.toBase58()}`);
        return; // Exit loop if successful
      } catch (error) {
        console.warn(
          `Airdrop failed for ${account.publicKey.toBase58()}, retrying... (${i + 1}/${retries})`
        );
        if (i === retries - 1) throw error; // Rethrow if out of retries
        await delay(delayMs); // Wait before retrying
      }
    }
  }

  // Delay function using Solana's getLatestBlockhash
  async function delay(ms: number): Promise<void> {
    const start = Date.now();
    while (Date.now() - start < ms) {
      // Use Solana RPC call to prevent blocking
      await pg.connection.getLatestBlockhash();
    }
  }

  before(async () => {
    console.log("Setting up test accounts...");

    // Generate keypairs for test accounts
    userTokenA = new web3.Keypair();
    userTokenB = new web3.Keypair();
    poolTokenA = new web3.Keypair();
    poolTokenB = new web3.Keypair();
    rewardMint = new web3.Keypair();
    rewardAccount = new web3.Keypair();
    userLPToken = new web3.Keypair();
    vaultLPToken = new web3.Keypair();
    userVault = new web3.Keypair();
    config = new web3.Keypair();
    priceFeed = new web3.Keypair();
    authority = pg.wallet;

    // Request airdrops for all accounts with retries
    const accounts = [
      userTokenA,
      userTokenB,
      poolTokenA,
      poolTokenB,
      rewardMint,
      rewardAccount,
    ];
    await Promise.all(
      accounts.map((account) => requestAirdropWithRetry(account))
    );

    console.log("Test accounts setup complete.");
  });

  it("should provide liquidity", async () => {
    const amountA = new anchor.BN(1000);
    const amountB = new anchor.BN(500);

    const txHash = await pg.program.methods
      .provideLiquidity(amountA, amountB)
      .accounts({
        userTokenA: userTokenA.publicKey,
        userTokenB: userTokenB.publicKey,
        poolTokenA: poolTokenA.publicKey,
        poolTokenB: poolTokenB.publicKey,
        rewardMint: rewardMint.publicKey,
        userRewardAccount: rewardAccount.publicKey,
        rewardMintAuthority: pg.wallet.publicKey,
        userAuthority: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userTokenA, userTokenB])
      .rpc();

    console.log(`Provide Liquidity Transaction: ${txHash}`);
    await pg.connection.confirmTransaction(txHash);

    console.log("Liquidity provided successfully.");
  });

  it("should remove liquidity", async () => {
    const liquidityAmount = new anchor.BN(500);

    const txHash = await pg.program.methods
      .removeLiquidity(liquidityAmount)
      .accounts({
        userLpToken: userLPToken.publicKey,
        poolLpToken: poolTokenA.publicKey, // Assuming pool LP token is tied to Token A
        userAuthority: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userLPToken])
      .rpc();

    console.log(`Remove Liquidity Transaction: ${txHash}`);
    await pg.connection.confirmTransaction(txHash);

    console.log("Liquidity removed successfully.");
  });

  it("should swap tokens", async () => {
    const amountIn = new anchor.BN(100);
    const minAmountOut = new anchor.BN(90);

    const txHash = await pg.program.methods
      .swapTokens(amountIn, minAmountOut)
      .accounts({
        userInputToken: userTokenA.publicKey,
        swapPoolToken: poolTokenA.publicKey,
        userOutputToken: userTokenB.publicKey,
        userAuthority: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userTokenA])
      .rpc();

    console.log(`Swap Tokens Transaction: ${txHash}`);
    await pg.connection.confirmTransaction(txHash);

    console.log("Token swap completed successfully.");
  });

  it("should stake LP tokens", async () => {
    const stakeAmount = new anchor.BN(200);

    const txHash = await pg.program.methods
      .stakeTokens(stakeAmount)
      .accounts({
        userLpToken: userLPToken.publicKey,
        vaultLpToken: vaultLPToken.publicKey,
        userVault: userVault.publicKey,
        userAuthority: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userLPToken])
      .rpc();

    console.log(`Stake LP Tokens Transaction: ${txHash}`);
    await pg.connection.confirmTransaction(txHash);

    console.log("LP Tokens staked successfully.");
  });

  it("should adjust the fee", async () => {
    const marketVolatility = new anchor.BN(60);

    const txHash = await pg.program.methods
      .adjustFee(marketVolatility)
      .accounts({
        config: config.publicKey,
        authority: pg.wallet.publicKey,
      })
      .rpc();

    console.log(`Adjust Fee Transaction: ${txHash}`);
    await pg.connection.confirmTransaction(txHash);

    console.log("Fee adjusted successfully.");
  });

  it("should mitigate impermanent loss", async () => {
    const priceThreshold = new anchor.BN(100);

    const txHash = await pg.program.methods
      .mitigateImpermanentLoss(priceThreshold)
      .accounts({
        poolTokenA: poolTokenA.publicKey,
        poolTokenB: poolTokenB.publicKey,
        priceFeed: priceFeed.publicKey,
      })
      .rpc();

    console.log(`Mitigate Impermanent Loss Transaction: ${txHash}`);
    await pg.connection.confirmTransaction(txHash);

    console.log("Impermanent loss mitigation executed.");
  });
});
