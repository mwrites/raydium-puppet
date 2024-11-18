import { PublicKey } from '@solana/web3.js';
import {
    Account,
    getMint,
    getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { connection, userWallet } from  "./config";
import BN from 'bn.js';


class TokenHelper {
    mint: PublicKey;

    constructor(mint: PublicKey) {
        this.mint = mint;
    }

    getMint = async (): Promise<PublicKey> => {
       return (await getMint(connection, this.mint)).address;
    }

    balance = async (): Promise<{ amount: BN; formatted: number}> => {
        const tokenBag = (await this.getOrCreateTokenBag(userWallet.publicKey)).address;
        const balanceInfo = await connection.getTokenAccountBalance(tokenBag);
        const amount = new BN(balanceInfo.value.amount);
        const decimals = balanceInfo.value.decimals || 0;
        const formatted = amount.div(new BN(10).pow(new BN(decimals))).toNumber();
        return { amount, formatted };
    }

    getOrCreateTokenBag = async (owner: PublicKey, isPDA: boolean = false): Promise<Account> => { 
        // Get or create the account for token of type mint for owner
        return await getOrCreateAssociatedTokenAccount(
            connection,
            userWallet.payer,
            this.mint,
            owner,
            isPDA,
        );
    }
}

export {
    TokenHelper
}