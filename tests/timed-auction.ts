import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TimedAuction } from '../target/types/timed_auction';
import {
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction,
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
    const payer = provider.wallet as anchor.Wallet;
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

    const bidderSecretKey = Uint8Array.from([
        42, 165, 245, 159, 222, 186, 30, 59, 255, 159, 113, 47, 54, 124, 76,
        163, 43, 196, 84, 5, 49, 170, 50, 11, 138, 41, 232, 148, 12, 220, 123,
        134, 132, 67, 120, 119, 74, 89, 31, 50, 157, 152, 126, 40, 40, 244, 236,
        21, 66, 18, 70, 7, 224, 123, 171, 16, 156, 248, 229, 104, 156, 119, 255,
        160,
    ]);
    const bidderKeypair = Keypair.fromSecretKey(bidderSecretKey);

    let collectionMintKeyPair: Keypair;
    let mintKeyPair: Keypair;
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
        const sellerTokenAccount = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            payer.publicKey
        );

        const [pdaAccount, bump] = PublicKey.findProgramAddressSync(
            [Buffer.from('sale'), mintKeyPair.publicKey.toBuffer()],
            program.programId
        );

        const pdaTokenAccountAddress = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            pdaAccount,
            true
        );

        const createPdaTokenAccountIx = createAssociatedTokenAccountInstruction(
            payer.publicKey,
            pdaTokenAccountAddress,
            pdaAccount,
            mintKeyPair.publicKey
        );

        const currentTimestamp = Math.floor(Date.now() / 1000);
        startTime = currentTimestamp;
        endTime = currentTimestamp + 60;

        const startPrice = new anchor.BN(1000000000);

        const transactionSignature = await program.methods
            .startAuction(
                new anchor.BN(startTime),
                new anchor.BN(endTime),
                startPrice
            )
            .accounts({
                seller: payer.publicKey,
                sellerTokenAccount: sellerTokenAccount,
                pdaAccount,
                pdaTokenAccount: pdaTokenAccountAddress,
                mint: mintKeyPair.publicKey,
                pdaSigner: pdaAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .preInstructions([createPdaTokenAccountIx])
            .signers([])
            .rpc({ skipPreflight: true });

        console.log('Auction started');
        console.log('Transaction signature', transactionSignature);
    });

    it('Place bid', async () => {
        const [pdaAccount] = PublicKey.findProgramAddressSync(
            [Buffer.from('sale'), mintKeyPair.publicKey.toBuffer()],
            program.programId
        );

        const bidAmount = new anchor.BN(2000000000);

        const transactionSignature = await program.methods
            .placeBid(bidAmount)
            .accounts({
                bidder: bidderKeypair.publicKey,
                pdaAccount: pdaAccount,
            })
            .signers([bidderKeypair])
            .rpc({ skipPreflight: true });

        console.log('Bid placed');
        console.log('Transaction signature', transactionSignature);

        // const auctionState = await program.account.auction.fetch(pdaAccount);
        // console.log('Auction State:', auctionState);
    });

    it('End auction', async () => {
        await new Promise(resolve => setTimeout(resolve, 60000)); 

        const sellerTokenAccount = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            payer.publicKey
        );

        const [pdaAccount, bump] = PublicKey.findProgramAddressSync(
            [Buffer.from('sale'), mintKeyPair.publicKey.toBuffer()],
            program.programId
        );

        const pdaTokenAccountAddress = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            pdaAccount,
            true
        );

        const highestBidderTokenAccountAddress = getAssociatedTokenAddressSync(
            mintKeyPair.publicKey,
            bidderKeypair.publicKey
        );

        const pdaTokenAccountInfo = await getAccount(
            provider.connection,
            pdaTokenAccountAddress
        ).catch(async () => {
            const createPdaTokenAccountIx = createAssociatedTokenAccountInstruction(
                payer.publicKey,
                pdaTokenAccountAddress,
                pdaAccount,
                mintKeyPair.publicKey
            );

            const transaction = new Transaction().add(createPdaTokenAccountIx);
            await sendAndConfirmTransaction(provider.connection, transaction, [payer.payer]);
        });

        const highestBidderTokenAccountInfo = await getAccount(
            provider.connection,
            highestBidderTokenAccountAddress
        ).catch(async () => {
            const createHighestBidderTokenAccountIx = createAssociatedTokenAccountInstruction(
                payer.publicKey,
                highestBidderTokenAccountAddress,
                bidderKeypair.publicKey,
                mintKeyPair.publicKey
            );

            const transaction = new Transaction().add(createHighestBidderTokenAccountIx);
            await sendAndConfirmTransaction(provider.connection, transaction, [payer.payer]);
        });

        const transactionSignature = await program.methods
            .endAuction()
            .accounts({
                seller: payer.publicKey,
                highestBidder: bidderKeypair.publicKey,
                pdaAccount: pdaAccount,
                pdaTokenAccount: pdaTokenAccountAddress,
                pdaSigner: pdaAccount,
                highestBidderTokenAccount: highestBidderTokenAccountAddress,
                mint: mintKeyPair.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([payer.payer, bidderKeypair])
            .rpc({ skipPreflight: true });

        console.log('Auction ended');
        console.log('Transaction signature', transactionSignature);
    });
});
