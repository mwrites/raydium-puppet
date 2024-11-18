import {  PublicKey } from '@solana/web3.js';
import { userWallet,  mintAAddress, mintBAddress } from "./config"
import { TokenHelper } from "./token_helper";
import { Wallet } from "@coral-xyz/anchor";


class User {
    mintAToken: TokenHelper;
    mintATokenBag: PublicKey;
    mintBToken: TokenHelper;
    mintBTokenBag: PublicKey;
    wallet: Wallet;

    constructor(wallet = userWallet) {
        this.mintAToken = new TokenHelper(mintAAddress);
        this.mintBToken = new TokenHelper(mintBAddress);
        this.wallet = wallet;
    }

    getOrCreateMintATokenBag = async () => {
       this.mintATokenBag = (await this.mintAToken.getOrCreateTokenBag(this.wallet.publicKey)).address;
    }

    getOrCreateMintBTokenBag = async () => {
        this.mintBTokenBag = (await this.mintBToken.getOrCreateTokenBag(this.wallet.publicKey)).address;
    }

    mintABalance = async () => {
        // call getOrCreateBeefTokenBag first
        return await this.mintAToken.balance();
    }

    mintBBalance = async () => {
        // call getOrCreateStakeTokenbag first
        return await this.mintBToken.balance();
    }
}


export {
    User
}