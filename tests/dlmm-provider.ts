// FILE: tests/dloom_flow.ts

import * as anchor from "@coral-xyz/anchor";
import { Program, BN, AnchorError } from "@coral-xyz/anchor";
import { DloomFlow } from "../target/types/dloom_flow";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress, // <<< FIX: Import the correct function
  mintTo,
  getAccount,
  createMint,
} from "@solana/spl-token";
import { assert } from "chai";

const METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const sortMints = (
  mintA: PublicKey,
  mintB: PublicKey
): [PublicKey, PublicKey] => {
  return mintA.toBase58() < mintB.toBase58() ? [mintA, mintB] : [mintB, mintA];
};

describe("dloom_flow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DloomFlow as Program<DloomFlow>;
  const payer = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  let splMintA: PublicKey;
  let splMintB: PublicKey;
  let t22MintA: PublicKey;
  let t22MintB: PublicKey;

  before(async () => {
    const balance = await connection.getBalance(payer.publicKey);
    if (balance < 2 * LAMPORTS_PER_SOL) {
      await connection.requestAirdrop(payer.publicKey, 2 * LAMPORTS_PER_SOL);
    }

    console.log("Creating test mints...");
    splMintA = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      null,
      6,
      Keypair.generate(),
      undefined,
      TOKEN_PROGRAM_ID
    );
    splMintB = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      null,
      6,
      Keypair.generate(),
      undefined,
      TOKEN_PROGRAM_ID
    );
    t22MintA = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      null,
      6,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    t22MintB = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      null,
      6,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    console.log("Test mints created.");
  });

  const testPoolFunctionality = (
    description: string,
    mint1: () => PublicKey,
    mint1Program: PublicKey,
    mint2: () => PublicKey,
    mint2Program: PublicKey
  ) => {
    describe(description, () => {
      const binStep = 20;
      const feeRate = 50;
      const initialBinId = 10;

      let tokenAMint: PublicKey, tokenBMint: PublicKey;
      let tokenAProgram: PublicKey, tokenBProgram: PublicKey;
      let poolPda: PublicKey;
      let tokenAVault: PublicKey, tokenBVault: PublicKey;
      let userTokenAAccount: PublicKey, userTokenBAccount: PublicKey;

      const positionMint = Keypair.generate();
      let positionPda: PublicKey;

      before(async () => {
        const sorted = sortMints(mint1(), mint2());
        tokenAMint = sorted[0];
        tokenBMint = sorted[1];
        tokenAProgram = tokenAMint.equals(mint1())
          ? mint1Program
          : mint2Program;
        tokenBProgram = tokenBMint.equals(mint1())
          ? mint1Program
          : mint2Program;

        [poolPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("pool"),
            tokenAMint.toBuffer(),
            tokenBMint.toBuffer(),
            new BN(binStep).toBuffer("le", 2),
          ],
          program.programId
        );

        [tokenAVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), poolPda.toBuffer(), tokenAMint.toBuffer()],
          program.programId
        );
        [tokenBVault] = PublicKey.findProgramAddressSync(
          [Buffer.from("vault"), poolPda.toBuffer(), tokenBMint.toBuffer()],
          program.programId
        );

        [positionPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("position"), positionMint.publicKey.toBuffer()],
          program.programId
        );

        const userTokenAInfo = await getOrCreateAssociatedTokenAccount(
          connection,
          payer.payer,
          tokenAMint,
          payer.publicKey,
          false,
          "confirmed",
          undefined,
          tokenAProgram
        );
        const userTokenBInfo = await getOrCreateAssociatedTokenAccount(
          connection,
          payer.payer,
          tokenBMint,
          payer.publicKey,
          false,
          "confirmed",
          undefined,
          tokenBProgram
        );
        userTokenAAccount = userTokenAInfo.address;
        userTokenBAccount = userTokenBInfo.address;

        await mintTo(
          connection,
          payer.payer,
          tokenAMint,
          userTokenAAccount,
          payer.payer,
          1_000_000_000,
          [],
          undefined,
          tokenAProgram
        );
        await mintTo(
          connection,
          payer.payer,
          tokenBMint,
          userTokenBAccount,
          payer.payer,
          1_000_000_000,
          [],
          undefined,
          tokenBProgram
        );
      });

      it("Should initialize the pool", async () => {
        try {
          await program.methods
            .initializePool(binStep, feeRate, initialBinId)
            .accounts({
              tokenAMint,
              tokenBMint,
              payer: payer.publicKey,
              pool: poolPda,
              tokenAProgram,
              tokenBProgram,
              tokenAVault,
              tokenBVault,
            })
            .rpc({ commitment: "confirmed" });

          const poolAccount = await program.account.pool.fetch(poolPda);
          assert.ok(poolAccount.tokenAMint.equals(tokenAMint));
          assert.equal(poolAccount.binStep, binStep);
        } catch (error) {
          const err = error as AnchorError;
          if (err.message.includes("already in use")) {
            console.log(
              `Pool ${description} already exists, skipping creation.`
            );
            return;
          }
          throw error;
        }
      });

      it("Should add liquidity", async () => {
        const lowerBinId = initialBinId - 2 * binStep;
        const upperBinId = initialBinId + 2 * binStep;
        const amountToDeposit = new BN(100_000);

        const binPubkeys: PublicKey[] = [];
        for (let id = lowerBinId; id <= upperBinId; id += binStep) {
          const [binPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("bin"),
              poolPda.toBuffer(),
              new BN(id).toBuffer("le", 4),
            ],
            program.programId
          );
          binPubkeys.push(binPda);
          try {
            await program.methods
              .initializeBin(id)
              .accounts({ bin: binPda, pool: poolPda, payer: payer.publicKey })
              .rpc();
          } catch (e) {
            /* ignore */
          }
        }

        const remainingAccounts = binPubkeys.map((pubkey) => ({
          pubkey,
          isWritable: true,
          isSigner: false,
        }));

        // <<< FIX: Use getAssociatedTokenAddress to just get the address, not create it.
        const userPositionNftAccount = await getAssociatedTokenAddress(
          positionMint.publicKey,
          payer.publicKey
        );

        const [metadataPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            METADATA_PROGRAM_ID.toBuffer(),
            positionMint.publicKey.toBuffer(),
          ],
          METADATA_PROGRAM_ID
        );
        const [masterEditionPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            METADATA_PROGRAM_ID.toBuffer(),
            positionMint.publicKey.toBuffer(),
            Buffer.from("edition"),
          ],
          METADATA_PROGRAM_ID
        );

        await program.methods
          .addLiquidity(lowerBinId, upperBinId, amountToDeposit)
          .accounts({
            pool: poolPda,
            position: positionPda,
            owner: payer.publicKey,
            payer: payer.publicKey,
            tokenAMint,
            tokenBMint,
            tokenAVault,
            tokenBVault,
            positionMint: positionMint.publicKey,
            userPositionNftAccount,
            metadataAccount: metadataPda,
            masterEditionAccount: masterEditionPda,
            userTokenAAccount,
            userTokenBAccount,
            tokenAProgram,
            tokenBProgram,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METADATA_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY, // Add rent sysvar
          })
          .remainingAccounts(remainingAccounts)
          .signers([positionMint])
          .rpc({ commitment: "confirmed" });

        const positionAccount = await program.account.position.fetch(
          positionPda
        );
        assert.equal(
          positionAccount.liquidity.toString(),
          amountToDeposit.toString()
        );
      });

      it("Should remove liquidity", async () => {
        const positionAccountBefore = await program.account.position.fetch(
          positionPda
        );
        const liquidityToRemove = positionAccountBefore.liquidity;
        const lowerBinId = positionAccountBefore.lowerBinId;
        const upperBinId = positionAccountBefore.upperBinId;

        const binPubkeys: PublicKey[] = [];
        for (let id = lowerBinId; id <= upperBinId; id += binStep) {
          const [binPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("bin"),
              poolPda.toBuffer(),
              new BN(id).toBuffer("le", 4),
            ],
            program.programId
          );
          binPubkeys.push(binPda);
        }

        const remainingAccounts = binPubkeys.map((pubkey) => ({
          pubkey,
          isWritable: true,
          isSigner: false,
        }));

        await program.methods
          .removeLiquidity(liquidityToRemove as BN, new BN(0), new BN(0))
          .accounts({
            owner: payer.publicKey,
            pool: poolPda,
            position: positionPda,
            tokenAMint,
            tokenBMint,
            tokenAVault,
            tokenBVault,
            userTokenAAccount,
            userTokenBAccount,
            tokenAProgram,
            tokenBProgram,
          })
          .remainingAccounts(remainingAccounts)
          .rpc({ commitment: "confirmed" });

        const positionAccountAfter = await program.account.position.fetch(
          positionPda
        );
        assert.equal(positionAccountAfter.liquidity.toString(), "0");
      });

      it("Should swap tokens", async () => {
        const lowerBinId = initialBinId - 5 * binStep;
        const upperBinId = initialBinId + 5 * binStep;
        const amountToDeposit = new BN(500_000);
        const tempPositionMint = Keypair.generate();
        const [tempPositionPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("position"), tempPositionMint.publicKey.toBuffer()],
          program.programId
        );

        // <<< FIX: Use getAssociatedTokenAddress here as well.
        const tempUserPositionNftAccount = await getAssociatedTokenAddress(
          tempPositionMint.publicKey,
          payer.publicKey
        );

        const [tempMetadataPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            METADATA_PROGRAM_ID.toBuffer(),
            tempPositionMint.publicKey.toBuffer(),
          ],
          METADATA_PROGRAM_ID
        );
        const [tempMasterEditionPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            METADATA_PROGRAM_ID.toBuffer(),
            tempPositionMint.publicKey.toBuffer(),
            Buffer.from("edition"),
          ],
          METADATA_PROGRAM_ID
        );

        const binPubkeys: PublicKey[] = [];
        for (let id = lowerBinId; id <= upperBinId; id += binStep) {
          const [binPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("bin"),
              poolPda.toBuffer(),
              new BN(id).toBuffer("le", 4),
            ],
            program.programId
          );
          binPubkeys.push(binPda);
          try {
            await program.methods
              .initializeBin(id)
              .accounts({ bin: binPda, pool: poolPda, payer: payer.publicKey })
              .rpc();
          } catch (e) {
            /* ignore */
          }
        }
        await program.methods
          .addLiquidity(lowerBinId, upperBinId, amountToDeposit)
          .accounts({
            pool: poolPda,
            position: tempPositionPda,
            owner: payer.publicKey,
            payer: payer.publicKey,
            tokenAMint,
            tokenBMint,
            tokenAVault,
            tokenBVault,
            positionMint: tempPositionMint.publicKey,
            userPositionNftAccount: tempUserPositionNftAccount,
            metadataAccount: tempMetadataPda,
            masterEditionAccount: tempMasterEditionPda,
            userTokenAAccount,
            userTokenBAccount,
            tokenAProgram,
            tokenBProgram,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METADATA_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .remainingAccounts(
            binPubkeys.map((p) => ({
              pubkey: p,
              isWritable: true,
              isSigner: false,
            }))
          )
          .signers([tempPositionMint])
          .rpc({ commitment: "confirmed" });

        const amountIn = new BN(10000);
        const minAmountOut = new BN(1);
        const balanceBefore = await getAccount(
          connection,
          userTokenBAccount,
          "confirmed",
          tokenBProgram
        );

        const activeId = (await program.account.pool.fetch(poolPda))
          .activeBinId;
        const swapBinPubkeys: PublicKey[] = [];
        for (let i = -5; i <= 5; i++) {
          const id = activeId + i * binStep;
          const [binPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("bin"),
              poolPda.toBuffer(),
              new BN(id).toBuffer("le", 4),
            ],
            program.programId
          );
          swapBinPubkeys.push(binPda);
        }

        await program.methods
          .swap(amountIn, minAmountOut)
          .accounts({
            trader: payer.publicKey,
            pool: poolPda,
            tokenAMint,
            tokenBMint,
            sourceVault: tokenAVault,
            destinationVault: tokenBVault,
            userSourceTokenAccount: userTokenAAccount,
            userDestinationTokenAccount: userTokenBAccount,
            tokenAProgram,
            tokenBProgram,
          })
          .remainingAccounts(
            swapBinPubkeys.map((pubkey) => ({
              pubkey,
              isWritable: true,
              isSigner: false,
            }))
          )
          .rpc({ commitment: "confirmed" });

        const balanceAfter = await getAccount(
          connection,
          userTokenBAccount,
          "confirmed",
          tokenBProgram
        );
        assert.ok(
          balanceAfter.amount > balanceBefore.amount,
          "Destination balance should have increased after swap"
        );
      });
    });
  };

  testPoolFunctionality(
    "SPL-SPL Pool",
    () => splMintA,
    TOKEN_PROGRAM_ID,
    () => splMintB,
    TOKEN_PROGRAM_ID
  );
  testPoolFunctionality(
    "Token2022-Token2022 Pool",
    () => t22MintA,
    TOKEN_2022_PROGRAM_ID,
    () => t22MintB,
    TOKEN_2022_PROGRAM_ID
  );
  testPoolFunctionality(
    "Mixed (SPL-Token2022) Pool",
    () => splMintA,
    TOKEN_PROGRAM_ID,
    () => t22MintA,
    TOKEN_2022_PROGRAM_ID
  );
});
