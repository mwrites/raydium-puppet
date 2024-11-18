import { mintTo }  from "@solana/spl-token";
import { mintAKeypair, mintBKeypair, connection, userWallet, findProgramAddress } from "./config";
import { TokenHelper } from "./token_helper";
import { User } from "./user_helper";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";


const createAndAirdropMint = async () => {
    const mintAddress = await _createMint(mintAKeypair, mintAKeypair.publicKey)
    const mintBddress = await _createMint(mintBKeypair, mintBKeypair.publicKey)
    console.log(`Mint 🍏'${mintAddress.toString()}'`);
    console.log(`Mint 🍌'${mintBddress.toString()}'` );
    
    const user = new User()
    await Promise.all([
        user.getOrCreateMintATokenBag(),
        user.getOrCreateMintBTokenBag()
    ]);
    await Promise.all([
        mintTokens(mintAKeypair, user.mintATokenBag),
        mintTokens(mintBKeypair, user.mintBTokenBag)
    ]);

    let balance = await (new TokenHelper(mintAddress)).balance();
    console.log(`Token Account 🍏'${user.mintATokenBag.toString()}' balance: ${balance}`);

    balance = await (new TokenHelper(mintBddress)).balance();
    console.log(`Token Account 🍌'${user.mintBTokenBag.toString()}' balance: ${balance}`);
}



const _createMint = async (keypairToAssign: Keypair, authorityToAssign: PublicKey): Promise<PublicKey> => {
    try {
        return await createMint(
            connection,
            userWallet.payer,
            authorityToAssign, // mint authority
            null, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
            6, // decimals
            keypairToAssign // address of the mint
        );
    } catch (err) {
        const logs = await err.getLogs();
        // Check if the error is "Create Account: account ... already in use"
        if (logs.some(log => log.includes("Create Account: account") && log.includes("already in use"))) {
            console.warn("Mints already created, continuing...");
            return keypairToAssign.publicKey; // Return the existing public key
        } else {
            throw err; // Re-raise the error if it's not the specific one
        }
    }
}

const mintTokens = async (mintKeypair: Keypair, tokenBag: PublicKey) => {
    await mintTo(
        connection,
        userWallet.payer,
        mintKeypair.publicKey,
        tokenBag,
        mintKeypair,
        1_000_000_000_000_000_000,
        []
    );
};


export {
    createAndAirdropMint
}

