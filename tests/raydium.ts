import { assert } from "chai";
import BN from 'bn.js';

import { PublicKey } from "@solana/web3.js";
import { Percent } from '@raydium-io/raydium-sdk-v2'

import { fetchPoolInfo } from "./helpers/config";
import { TokenHelper } from "./helpers/token_helper";
import { createMarket } from "./helpers/create_market";
import { createAmmPool } from "./helpers/create_pool";
import { addLiquidity } from "./add_liquidity";
import { removeLiquidity } from "./remove_liquidity";


describe("raydium-puppet", () => {
    let ammPoolId: PublicKey | undefined;

    before(async () => {
        // already done! shouldn't do it again unless it's localnet
        // await createAndAirdropMint();
        const marketId = await createMarket()
        assert(marketId, "Failed to create market. Exiting setup.");
        console.log("marketId", marketId.toBase58());
        ammPoolId = await createAmmPool(marketId);
    });

    it('It deposits liquidity', async () => {
        assert(ammPoolId, "Test setup failed. No poolId.");

        // Fetch pool info before adding liquidity
        let { poolInfo, poolKeys } = await fetchPoolInfo(ammPoolId);
        const initialMintAmountA = poolInfo.mintAmountA;
        const initialMintAmountB = poolInfo.mintAmountB;
 
        const lpMint = new TokenHelper(new PublicKey(poolInfo.lpMint.address))
        let { amount, formatted } = await lpMint.balance();
        const initLpAmount = amount
        const initLpAmountFormatted = formatted
        console.log('initial lp amount', formatted)

        // the function to test
        await addLiquidity(ammPoolId, new Percent(1, 100));

        // Fetch pool info after adding liquidity
        ({ poolInfo } = await fetchPoolInfo(ammPoolId));
        // logObject('Pool Info After', poolInfo)
        const finalMintAmountA = poolInfo.mintAmountA;
        const finalMintAmountB = poolInfo.mintAmountB;
        ({ amount, formatted } = await lpMint.balance())
        const finalLpAmount = amount
        const finalLpAmountFormatted = formatted
        console.log('after lp amount', formatted.toString())
        
        assert.strictEqual(finalMintAmountA, initialMintAmountA + 1, "Mint Amount A should have increased by 1 after adding liquidity");
        assert.strictEqual(finalMintAmountB, initialMintAmountB + 1, "Mint Amount B should have increased by 1 after adding liquidity");
        assert.isTrue(finalLpAmount.eq(initLpAmount.add(new BN(10).pow(new BN(poolInfo.lpMint.decimals)))), "Should have received 1 lp");

        console.log(`User LP amount \x1b[31m${initLpAmountFormatted}\x1b[0m -> \x1b[32m${finalLpAmountFormatted}\x1b[0m`);
        console.log(`Coin vault balance \x1b[31m${initialMintAmountA}\x1b[0m -> \x1b[32m${finalMintAmountA}\x1b[0m`);
        console.log(`PC vault balance \x1b[31m${initialMintAmountB}\x1b[0m -> \x1b[32m${finalMintAmountB}\x1b[0m`);
    })

    it('It removes liquidity', async () => {
        assert(ammPoolId, "Test setup failed. No poolId.");

        // Fetch pool info before removing liquidity
        let { poolInfo, poolKeys } = await fetchPoolInfo(ammPoolId);
        const initialMintAmountA = poolInfo.mintAmountA;
        const initialMintAmountB = poolInfo.mintAmountB;
        const lpMint = new TokenHelper(new PublicKey(poolInfo.lpMint.address))
        let { amount, formatted } = await lpMint.balance();
        const initLpAmount = amount
        const initLpAmountFormatted = formatted
        console.log('initial lp amount', formatted)

        // probably want to test under different condition, like low slippage etc
        // to make this test stable we will keep a high slippage threshold
        const highSlippage = new Percent(4, 100)
        await removeLiquidity(ammPoolId, highSlippage);

        // Fetch pool info after removing liquidity
        ({ poolInfo } = await fetchPoolInfo(ammPoolId));
        // logObject('Pool Info After', poolInfo)
        const finalMintAmountA = poolInfo.mintAmountA;
        const finalMintAmountB = poolInfo.mintAmountB;
        ({ amount, formatted } = await lpMint.balance())
        const finalLpAmount = amount
        const finalLpAmountFormatted = formatted
        console.log('after lp amount', formatted.toString())

        // /!\ ideally poolInfo.mintAmountX should be a BN not a number, so that we can compare at the decimals level
        // but the SDK return poolInfo.mintAmountX as a number
        assert.strictEqual(finalMintAmountA, initialMintAmountA - 1, "Mint Amount A should have decreased by 1 after removing liquidity");
        assert.strictEqual(finalMintAmountB, initialMintAmountB - 1, "Mint Amount B should have decreased by 1 after removing liquidity");
        assert.isTrue(finalLpAmount.eq(initLpAmount.sub(new BN(10).pow(new BN(poolInfo.lpMint.decimals)))), "Should have burned 1 lp");

         console.log(`User LP amount \x1b[31m${initLpAmountFormatted}\x1b[0m -> \x1b[32m${finalLpAmountFormatted}\x1b[0m`);
         console.log(`Coin vault balance \x1b[31m${initialMintAmountA}\x1b[0m -> \x1b[32m${finalMintAmountA}\x1b[0m`);
         console.log(`PC vault balance \x1b[31m${initialMintAmountB}\x1b[0m -> \x1b[32m${finalMintAmountB}\x1b[0m`);
    })
});
