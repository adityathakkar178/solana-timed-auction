import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TimedAuction } from '../target/types/timed_auction';
import {
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
} from '@solana/web3.js';
import {
    TOKEN_PROGRAM_ID,
    createAssociatedTokenAccountInstruction,
    getAccount,
    getAssociatedTokenAddressSync,
} from '@solana/spl-token';

describe('timed-auction', () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const payer = provider.wallet;
    const program = anchor.workspace.TimedAuction;

    const collectionMetadata = {
        name: 'Collection1',
        symbol: 'CXYZ',
        uri: 'collectionxyz',
    };

    const metadata = {
        name: 'XYZ',
        symbol: 'ABC',
        uri: 'abcxyz',
    };

    let collectionMintKeyPair: Keypair;
    let mintKeyPair: Keypair;
    let auctionKeyPair: Keypair;
    let startTime: number;
    let endTime: number;

    it('Mint Collection', async () => {
        collectionMintKeyPair = Keypair.generate();

        const collectionAssociatedTokenAccountAddress =
            getAssociatedTokenAddressSync(
                collectionMintKeyPair.publicKey,
                payer.publicKey
            );

        const collectionTransactionSignature = await program.methods
            .mintCollection(
                collectionMetadata.name,
                collectionMetadata.symbol,
                collectionMetadata.uri
            )
            .accounts({
                payer: payer.publicKey,
                collectionMintAccount: collectionMintKeyPair.publicKey,
                collectionAssociatedTokenAccount:
                    collectionAssociatedTokenAccountAddress,
            })
            .signers([collectionMintKeyPair])
            .rpc({ skipPreflight: true });

        console.log('Collection created');
        console.log('Transaction signature', collectionTransactionSignature);
    });

    it('Mint Nft with collections', async () => {
        mintKeyPair = Keypair.generate();

        const associatedTokenAccountAddress = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            payer.publicKey
        );

        const transactionSignature = await program.methods
            .mintNft(
                metadata.name,
                metadata.symbol,
                metadata.uri,
                collectionMintKeyPair.publicKey
            )
            .accounts({
                payer: payer.publicKey,
                mintAccount: mintKeyPair.publicKey,
                associatedTokenAccount: associatedTokenAccountAddress,
                collectionMetadata: collectionMintKeyPair.publicKey,
            })
            .signers([mintKeyPair])
            .rpc({ skipPreflight: true });

        console.log('NFT minted');
        console.log('Transaction signature', transactionSignature);
    });

    it('Start auction', async () => {
        auctionKeyPair = Keypair.generate();

        const sellerNftAccountAddress = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            payer.publicKey
        );

        const auctionNftAccountAddress = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            auctionKeyPair.publicKey
        );

        const currentTimestamp = Math.floor(Date.now() / 1000);
        startTime = currentTimestamp;
        endTime = currentTimestamp + 3600;

        const startPrice = new anchor.BN(1000000000);

        const transactionSignature = await program.methods
            .startAuction(
                new anchor.BN(startTime),
                new anchor.BN(endTime),
                startPrice
            )
            .accounts({
                auction: auctionKeyPair.publicKey,
                seller: payer.publicKey,
                sellerNftAccount: sellerNftAccountAddress,
                auctionNftAccount: auctionNftAccountAddress,
                mint: mintKeyPair.publicKey,
            })
            .signers([auctionKeyPair])
            .rpc({ skipPreflight: true });

        // const auctionAccount = await program.account.auction.fetch(auctionKeyPair.publicKey);
        // console.log('Auction Account:', auctionAccount);

        console.log('Auction started');
        console.log('Transaction signature', transactionSignature);
    });
});
