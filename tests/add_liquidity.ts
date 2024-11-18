import { PublicKey } from '@solana/web3.js'
import Decimal from 'decimal.js'
import {
    TokenAmount,
    toToken,
    Percent,
  } from '@raydium-io/raydium-sdk-v2'
  import {
    initRaydiumSdk,
    txVersion,
    isValidAmm,
    fetchPoolInfo,
  } from './helpers/config';
  
const addLiquidity = async (poolId: PublicKey, slippage: Percent) => {
    console.log("poolId", poolId.toBase58())

    const raydium = await initRaydiumSdk()
    const { poolInfo, poolKeys } = await fetchPoolInfo(poolId)

    if (!isValidAmm(poolInfo.programId)) throw new Error('target pool is not AMM pool')
    if (!poolInfo) throw new Error('poolInfo not found')
    if (!poolKeys) throw new Error('poolKeys not found')
    // logObject('poolInfo', poolInfo)
    // logObject('poolKeys', poolKeys)
  
    const inputAmount = '1'
  
    const r = raydium.liquidity.computePairAmount({
      poolInfo,
      amount: inputAmount,
      baseIn: true,
      slippage,
    })
  
    const { execute } = await raydium.liquidity.addLiquidity({
      poolInfo,
      poolKeys,
      amountInA: new TokenAmount(
        toToken(poolInfo.mintA),
        new Decimal(inputAmount).mul(10 ** poolInfo.mintA.decimals).toFixed(0)
      ),
      amountInB: new TokenAmount(
        toToken(poolInfo.mintB),
        new Decimal(r.maxAnotherAmount.toExact()).mul(10 ** poolInfo.mintA.decimals).toFixed(0)
      ),
      otherAmountMin: r.minAnotherAmount,
      fixedSide: 'a',
      txVersion,
    })
  
    // don't want to wait confirm, set sendAndConfirm to false or don't pass any params to execute
    const { txId } = await execute({ sendAndConfirm: true })
    console.log('liquidity added:', { txId: `https://explorer.solana.com/tx/${txId}` })
  }
  
  export {
    addLiquidity
  }


  