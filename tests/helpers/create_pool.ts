import BN from 'bn.js';
import { PublicKey } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  initRaydiumSdk,
  txVersion,
  logObject,
  ADDRESSES,
  saveToCache,
  loadFromCache,
  mintBAddress,
  mintAAddress,
  deleteCacheFile
} from './config';
import { MARKET_STATE_LAYOUT_V3 } from '@raydium-io/raydium-sdk-v2';

const createAmmPool = async (marketId: PublicKey): Promise<PublicKey> => {
  // Use the loadFromCache helper to read market info
  const cachedData = loadFromCache('pool.json');
  if (cachedData && cachedData.address.coinMint == mintAAddress && cachedData.address.pcMint == mintBAddress) {
    console.log('Pool info loaded from cache:', cachedData);
    return new PublicKey(cachedData.address.ammId);
  } else {
    deleteCacheFile('pool.json');
  }

  const raydium = await initRaydiumSdk()

  // if you are confirmed your market info, don't have to get market info from rpc below
  const marketBufferInfo = await raydium.connection.getAccountInfo(new PublicKey(marketId))
  const { baseMint, quoteMint } = MARKET_STATE_LAYOUT_V3.decode(marketBufferInfo!.data)

  // check mint info here: https://api-v3.raydium.io/mint/list
  // or get mint info by api: await raydium.token.getTokenInfo('mint address')

  const baseMintInfo = await raydium.token.getTokenInfo(baseMint)
  logObject('baseMintInfo', baseMintInfo)
  const quoteMintInfo = await raydium.token.getTokenInfo(quoteMint)
  logObject('quoteMintInfo', quoteMintInfo)

  const baseAmount = new BN(new BN(10 ** baseMintInfo.decimals)).pow(new BN(2))
  const quoteAmount = new BN(new BN(10 ** quoteMintInfo.decimals)).pow(new BN(2))

  if (
    baseMintInfo.programId !== TOKEN_PROGRAM_ID.toBase58() ||
    quoteMintInfo.programId !== TOKEN_PROGRAM_ID.toBase58()
  ) {
    throw new Error(
      'amm pools with openbook market only support TOKEN_PROGRAM_ID mints, if you want to create pool with token-2022, please create cpmm pool instead'
    )
  }

  if (baseAmount.mul(quoteAmount).lte(new BN(1).mul(new BN(10 ** baseMintInfo.decimals)).pow(new BN(2)))) {
    throw new Error('initial liquidity too low, try adding more baseAmount/quoteAmount')
  }

  const { execute, extInfo } = await raydium.liquidity.createPoolV4({
    programId: ADDRESSES.AmmV4,
    marketInfo: {
      marketId,
      programId: ADDRESSES.OpenBookMarket,
    },
    baseMintInfo: {
      mint: baseMint,
      decimals: baseMintInfo.decimals, // if you know mint decimals here, can pass number directly
    },
    quoteMintInfo: {
      mint: quoteMint,
      decimals: quoteMintInfo.decimals, // if you know mint decimals here, can pass number directly
    },
    baseAmount: baseAmount,
    quoteAmount: quoteAmount,

    startTime: new BN(0), // unit in seconds
    ownerInfo: {
      useSOLBalance: true,
    },
    associatedOnly: false,
    txVersion,
    feeDestinationId: ADDRESSES.FeeDestinationId,
    // optional: set up priority fee here
    // computeBudgetConfig: {
    //   units: 60000,
    //   microLamports: 10000000,
    // },
  })

  const { txId } = await execute({ sendAndConfirm: true })
  console.log('liquidity added:', { txId: `https://explorer.solana.com/tx/${txId}?cluster=devnet` })

  console.log(
    ', poolKeys:',
    Object.keys(extInfo.address).reduce(
      (acc, cur) => ({
        ...acc,
        [cur]: extInfo.address[cur as keyof typeof extInfo.address].toBase58(),
      }),
      {}
    )
  )
  logObject('extInfo', extInfo)
  saveToCache('pool.json', extInfo)
  return extInfo.address.ammId
}

export {
  createAmmPool
}