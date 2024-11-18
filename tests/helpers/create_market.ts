import { PublicKey } from '@solana/web3.js'
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

const createMarket = async (): Promise<PublicKey | undefined> => {
  // Use the loadFromCache helper to read market info
  const cachedData = loadFromCache('market.json');
  // that's right, it's quoteMin not quoteMint
  if (cachedData && cachedData.address.baseMint == mintAAddress && cachedData.address.quoteMin == mintBAddress) {
    console.log('Market info loaded from cache:', cachedData);
    return new PublicKey(cachedData.address.marketId);
  } else {
    deleteCacheFile('market.json');
  }

  const raydium = await initRaydiumSdk()

  try {
    const { execute, extInfo, transactions } = await raydium.marketV2.create({
      baseInfo: {
        mint: mintAAddress,
        decimals: 6,
      },
      quoteInfo: {
        mint: mintBAddress,
        decimals: 6,
      },
      lotSize: 1,
      tickSize: 0.01,
      dexProgramId: ADDRESSES.OpenBookMarket,
      txVersion,
    })



    console.log(
      `create market total ${transactions.length} txs, market info: `,
      Object.keys(extInfo.address).reduce(
        (acc, cur) => ({
          ...acc,
          [cur]: extInfo.address[cur as keyof typeof extInfo.address].toBase58(),
        }),
        {}
      )
    )

    const txIds = await execute({
      sequentially: true,
    });
    console.log('create market txIds:', txIds);

    logObject('extInfo', extInfo)
    saveToCache('market.json', extInfo)

    return extInfo.address.marketId;
  } catch (error) {
    logObject('Error executing transactions', error);
    return null;
  }
}

export {
  createMarket
}