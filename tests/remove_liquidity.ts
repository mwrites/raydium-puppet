import Decimal from 'decimal.js';
import BN from 'bn.js';
import { PublicKey } from '@solana/web3.js';
import { Percent } from '@raydium-io/raydium-sdk-v2';
import { initRaydiumSdk, txVersion, isValidAmm, fetchPoolInfo } from './helpers/config';

const calculateWithdrawAmounts = (
    withdrawAmountDe: Decimal,
    baseRatio: Decimal,
    quoteRatio: Decimal,
    poolInfo: any,
    slippage: Percent
) => {
    const withdrawAmountA = withdrawAmountDe
        .mul(baseRatio)
        .mul(new Decimal(10).pow(poolInfo.mintA.decimals || 0));
    const withdrawAmountB = withdrawAmountDe
        .mul(quoteRatio)
        .mul(new Decimal(10).pow(poolInfo.mintB.decimals || 0));
    const withdrawAmountLp = withdrawAmountDe
        .mul(new Decimal(10).pow(poolInfo.lpMint.decimals));

    const lpAmount = new BN(withdrawAmountLp.toFixed(0));
    const slippageAmountA = withdrawAmountA
        .mul(slippage.numerator.toString())
        .div(slippage.denominator.toString());
    const slippageAmountB = withdrawAmountB
        .mul(slippage.numerator.toString())
        .div(slippage.denominator.toString());
    const baseAmountMin = new BN(withdrawAmountA.sub(slippageAmountA).toFixed(0));
    const quoteAmountMin = new BN(withdrawAmountB.sub(slippageAmountB).toFixed(0));

    return { lpAmount, baseAmountMin, quoteAmountMin };
};

const removeLiquidity = async (poolId: PublicKey, slippage: Percent) => {
    console.log("poolId", poolId.toBase58());

    const raydium = await initRaydiumSdk();
    const { poolInfo, poolKeys } = await fetchPoolInfo(poolId);

    if (!isValidAmm(poolInfo.programId)) throw new Error('target pool is not AMM pool');
    if (!poolInfo) throw new Error('poolInfo not found');
    if (!poolKeys) throw new Error('poolKeys not found');

    const baseRatio = poolInfo.lpAmount === 0 
        ? new Decimal(1) 
        : new Decimal(poolInfo.mintAmountA).div(poolInfo.lpAmount);
    const quoteRatio = poolInfo.lpAmount === 0 
        ? new Decimal(1) 
        : new Decimal(poolInfo.mintAmountB).div(poolInfo.lpAmount);

    const withdrawAmountDe = new Decimal('1');
    const { lpAmount, baseAmountMin, quoteAmountMin } = calculateWithdrawAmounts(
        withdrawAmountDe,
        baseRatio,
        quoteRatio,
        poolInfo,
        slippage
    );

    console.log('baseAmountMin', baseAmountMin.toString());
    console.log('quoteAmountMin', quoteAmountMin.toString());
    console.log('lpAmount', lpAmount.toString());

    if (baseAmountMin.gt(new BN(poolInfo.mintAmountA))) {
        throw new Error(`withdrawAmountA (${baseAmountMin.toString()}) exceeds available liquidity (${poolInfo.mintAmountA})`);
    }
    if (quoteAmountMin.gt(new BN(poolInfo.mintAmountB))) {
        throw new Error(`withdrawAmountB (${quoteAmountMin.toString()}) exceeds available liquidity (${poolInfo.mintAmountB})`);
    }

    const { execute } = await raydium.liquidity.removeLiquidity({
        poolInfo,
        poolKeys,
        lpAmount,
        baseAmountMin,
        quoteAmountMin,
        txVersion,
    });

    const { txId } = await execute({ sendAndConfirm: true });
    console.log('liquidity withdraw:', { txId: `https://explorer.solana.com/tx/${txId}` });
};

export { removeLiquidity };
