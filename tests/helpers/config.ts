import { Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";
import * as anchor from "@coral-xyz/anchor";
import { Program, Wallet } from "@coral-xyz/anchor";
import { RaydiumPuppet } from "../../target/types/raydium_puppet";
import {
  Raydium,
  TxVersion,
  ApiV3PoolInfoStandardItem,
  AmmV4Keys,
  AMM_V4,
  AMM_STABLE,
  DEVNET_PROGRAM_ID,
  OPEN_BOOK_PROGRAM,
  FEE_DESTINATION_ID
} from '@raydium-io/raydium-sdk-v2'

// **************************************************************
// ********************** Cluster Stuff **********************
// **************************************************************
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const txVersion = TxVersion.V0 // or TxVersion.LEGACY
const program = anchor.workspace.RaydiumPuppet as Program<RaydiumPuppet>;
const connection = provider.connection
const userWallet: Wallet = anchor.workspace.RaydiumPuppet.provider.wallet;
const clusterUrl = provider.connection.rpcEndpoint;
const isDevnet = clusterUrl.includes('devnet');
const raydiumCluster = isDevnet ? 'devnet' : 'mainnet';

const ADDRESSES = {
  AmmV4: isDevnet ? DEVNET_PROGRAM_ID.AmmV4 : AMM_V4,
  OpenBookMarket: isDevnet ? DEVNET_PROGRAM_ID.OPENBOOK_MARKET : OPEN_BOOK_PROGRAM,
  FeeDestinationId: isDevnet ? DEVNET_PROGRAM_ID.FEE_DESTINATION_ID : FEE_DESTINATION_ID,
};

const VALID_AMM_PROGRAM_ID = new Set([
  AMM_V4.toBase58(),
  AMM_STABLE.toBase58(),
  DEVNET_PROGRAM_ID.AmmV4.toBase58(),
  DEVNET_PROGRAM_ID.AmmStable.toBase58(),
])

// Function to load or generate a keypair from a file
const loadOrGenerateKeypair = (filePath: string): Keypair => {
  if (!fs.existsSync(filePath)) {
    const keypair = Keypair.generate();
    fs.writeFileSync(filePath, JSON.stringify(Array.from(keypair.secretKey)));
    console.log(`Keypair generated and saved to ${filePath}`);
    return keypair;
  }
  const secretKeyString = fs.readFileSync(filePath, 'utf-8');
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

// Load or generate mintA and mintB keypairs
const mintAKeypair = loadOrGenerateKeypair('testing_keys/mint_a.json');
const mintBKeypair = loadOrGenerateKeypair('testing_keys/mint_b.json');

// Derive public keys from keypairs
const mintAAddress = mintAKeypair.publicKey;
const mintBAddress = mintBKeypair.publicKey;


// **************************************************************
// ********************** Helper Functions **********************
// **************************************************************
const findProgramAddress = (seed: PublicKey): [PublicKey, number] => {
  return PublicKey.findProgramAddressSync(
    [seed.toBuffer()],
    program.programId
  );
}

let raydium: Raydium | undefined
const initRaydiumSdk = async (params?: { loadToken?: boolean }) => {
  if (raydium) return raydium
  raydium = await Raydium.load({
    owner: userWallet.payer,
    connection,
    cluster: raydiumCluster,
    disableFeatureCheck: true,
    disableLoadToken: !params?.loadToken,
    blockhashCommitment: 'finalized',
  })
  return raydium;
}

const isValidAmm = (id: string) => VALID_AMM_PROGRAM_ID.has(id)

function logObject(label: string, obj: any) {
  console.log(`${label}: ${JSON.stringify(obj, null, 2)}`);
}

const cachePrefixPath = isDevnet ? 'cache/devnet_' : 'cache/';

const saveToCache = (path: string, obj: any) => {
  fs.writeFileSync(cachePrefixPath + path, JSON.stringify(obj, null, 2));
  console.log(`${path} saved to cache`);
}

const loadFromCache = (filePath: string): any | null => {
  const cachePath = cachePrefixPath + filePath
  if (fs.existsSync(cachePath)) {
    try {
      const data = JSON.parse(fs.readFileSync(cachePath, 'utf-8'));
      console.log(`Data loaded from cache: ${cachePath}`);
      return data;
    } catch (error) {
      console.error(`Error reading from cache file ${cachePath}:`, error);
    }
  }
  return null;
}

const deleteCacheFile = (filePath: string): void => {
  const cachePath = cachePrefixPath + filePath
  if (fs.existsSync(cachePath)) {
    fs.unlinkSync(cachePath);
    console.log('Cache file deleted:', cachePath);
  }
};


const fetchPoolInfo = async (poolId: PublicKey): Promise<{ poolInfo: ApiV3PoolInfoStandardItem, poolKeys: AmmV4Keys }> => {
  const raydium = await initRaydiumSdk()
  let poolInfo;
  let poolKeys;

  if (!isDevnet) {
    const data = await raydium.api.fetchPoolById({ ids: poolId.toBase58() });
    poolInfo = data[0] as ApiV3PoolInfoStandardItem;
    // where is poolKeys?
    throw new Error('Fatal error: not sure how to deal with that in mainnet for now');
  } else {
    const data = await raydium.liquidity.getPoolInfoFromRpc({ poolId: poolId.toBase58() });
    poolInfo = data.poolInfo;
    poolKeys = data.poolKeys;
  }

  return { poolInfo, poolKeys };
}

export {    
  txVersion,
  program,
  provider,
  connection,
  userWallet, 
  mintAAddress,
  mintAKeypair,
  mintBAddress,
  mintBKeypair,
  ADDRESSES,
  logObject,
  saveToCache,
  deleteCacheFile,
  loadFromCache,
  findProgramAddress,
  initRaydiumSdk,
  isValidAmm,
  fetchPoolInfo,
}