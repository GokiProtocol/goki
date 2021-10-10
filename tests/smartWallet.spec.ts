import "chai-bn";

import * as anchor from "@project-serum/anchor";
import { expectTX } from "@saberhq/chai-solana";
import { sleep } from "@saberhq/token-utils";
import { TransactionInstruction } from "@solana/web3.js";
import { expect } from "chai";
import invariant from "tiny-invariant";

import { SmartWalletErrors } from "../src/idls/smart_wallet";
import type { SmartWalletWrapper } from "../src/wrappers/smartWallet";
import { findSmartWallet } from "../src/wrappers/smartWallet";
import { makeSDK } from "./workspace";

describe("smartWallet", () => {
  const { BN, web3 } = anchor;
  const sdk = makeSDK();
  const program = sdk.programs.SmartWallet;

  describe("Tests the smartWallet program", () => {
    const smartWalletBase = web3.Keypair.generate();
    const numOwners = 10; // Big enough.

    const ownerA = web3.Keypair.generate();
    const ownerB = web3.Keypair.generate();
    const ownerC = web3.Keypair.generate();
    const ownerD = web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerC.publicKey];

    const threshold = new anchor.BN(2);

    let smartWalletWrapper: SmartWalletWrapper;

    before(async () => {
      const { smartWalletWrapper: wrapperInner, tx } = await sdk.newSmartWallet(
        {
          numOwners,
          owners,
          threshold,
          base: smartWalletBase,
        }
      );
      await expectTX(tx, "create new smartWallet").to.be.fulfilled;
      smartWalletWrapper = wrapperInner;
    });

    it("happy path", async () => {
      await smartWalletWrapper.reloadData();
      invariant(smartWalletWrapper.data, "smartWallet was not created");
      expect(smartWalletWrapper.data.threshold).to.be.bignumber.equal(
        new anchor.BN(2)
      );
      expect(smartWalletWrapper.data.owners).to.deep.equal(owners);
      const [smartWalletKey, bump] = await findSmartWallet(
        smartWalletWrapper.data.base
      );
      expect(smartWalletWrapper.data.bump).to.be.equal(bump);

      const newOwners = [ownerA.publicKey, ownerB.publicKey, ownerD.publicKey];
      const data = program.coder.instruction.encode("set_owners", {
        owners: newOwners,
      });

      const instruction = new TransactionInstruction({
        programId: program.programId,
        keys: [
          {
            pubkey: smartWalletKey,
            isWritable: true,
            isSigner: true,
          },
        ],
        data,
      });

      const { transactionKey, tx: proposeTx } =
        await smartWalletWrapper.newTransaction({
          proposer: ownerA.publicKey,
          instruction,
        });
      proposeTx.signers.push(ownerA);
      await expectTX(
        proposeTx,
        "create a tx to be processed by the smartWallet"
      ).to.be.fulfilled;

      const txAccount = await smartWalletWrapper.fetchTransaction(
        transactionKey
      );
      expect(txAccount.executedAt.toNumber()).to.equal(-1);
      expect(txAccount.ownerSetSeqno).to.equal(0);
      expect(txAccount.instruction.programId, "program id").to.eqAddress(
        program.programId
      );
      expect(txAccount.instruction.data, "data").to.deep.equal(data);
      expect(txAccount.instruction.keys, "keys").to.deep.equal(
        instruction.keys
      );
      expect(txAccount.smartWallet).to.eqAddress(smartWalletKey);

      // Other owner approves transaction.
      await expectTX(
        smartWalletWrapper
          .approveTransaction(transactionKey, ownerB.publicKey)
          .addSigners(ownerB)
      ).to.be.fulfilled;

      // Now that we've reached the threshold, send the transaction.
      await expectTX(
        (
          await smartWalletWrapper.executeTransaction({
            transactionKey,
            owner: ownerA.publicKey,
          })
        ).addSigners(ownerA)
      ).to.be.fulfilled;

      await smartWalletWrapper.reloadData();
      expect(smartWalletWrapper.bump).to.be.equal(bump);
      expect(smartWalletWrapper.data.ownerSetSeqno).to.equal(1);
      expect(smartWalletWrapper.data.threshold).to.bignumber.equal(new BN(2));
      expect(smartWalletWrapper.data.owners).to.deep.equal(newOwners);
    });
  });

  describe("Tests the smartWallet program with timelock", () => {
    const numOwners = 10; // Big enough.
    const smartWalletBase = web3.Keypair.generate();

    const ownerA = web3.Keypair.generate();
    const ownerB = web3.Keypair.generate();
    const ownerC = web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerC.publicKey];

    const threshold = new anchor.BN(1);
    const delay = new anchor.BN(10);

    let smartWalletWrapper: SmartWalletWrapper;

    before(async () => {
      const { smartWalletWrapper: wrapperInner, tx } = await sdk.newSmartWallet(
        {
          numOwners,
          owners,
          threshold,
          base: smartWalletBase,
          delay,
        }
      );
      await expectTX(tx, "create new smartWallet").to.be.fulfilled;
      smartWalletWrapper = wrapperInner;
    });

    it("invalid eta", async () => {
      await smartWalletWrapper.reloadData();
      invariant(smartWalletWrapper.data, "smartWallet was not created");
      const [smartWalletKey] = await findSmartWallet(
        smartWalletWrapper.data.base
      );

      const newOwners = [ownerA.publicKey, ownerB.publicKey];
      const data = program.coder.instruction.encode("set_owners", {
        owners: newOwners,
      });
      const instruction = new TransactionInstruction({
        programId: program.programId,
        keys: [
          {
            pubkey: smartWalletKey,
            isWritable: true,
            isSigner: true,
          },
        ],
        data,
      });

      const { tx } = await smartWalletWrapper.newTransaction({
        proposer: ownerB.publicKey,
        instruction,
      });
      tx.signers.push(ownerB);

      try {
        await tx.confirm();
      } catch (e) {
        const err = e as Error;
        expect(err.message).to.include(
          `0x${SmartWalletErrors.InvalidETA.code.toString(16)}`
        );
      }
    });

    it("execute tx", async () => {
      await smartWalletWrapper.reloadData();
      invariant(smartWalletWrapper.data, "smartWallet was not created");
      const [smartWalletKey] = await findSmartWallet(
        smartWalletWrapper.data.base
      );

      const newOwners = [ownerA.publicKey, ownerB.publicKey];
      const data = program.coder.instruction.encode("set_owners", {
        owners: newOwners,
      });
      const instruction = new TransactionInstruction({
        programId: program.programId,
        keys: [
          {
            pubkey: smartWalletKey,
            isWritable: true,
            isSigner: true,
          },
        ],
        data,
      });

      const eta = smartWalletWrapper.data.minimumDelay.add(
        new BN(Date.now() / 1000)
      );
      const { transactionKey, tx } = await smartWalletWrapper.newTransaction({
        proposer: ownerB.publicKey,
        instruction,
        eta,
      });
      tx.signers.push(ownerB);
      await expectTX(tx, "create a tx to be processed by the smartWallet").to.be
        .fulfilled;

      const falseStartTx = await smartWalletWrapper.executeTransaction({
        transactionKey,
        owner: ownerA.publicKey,
      });
      falseStartTx.addSigners(ownerA);
      try {
        await falseStartTx.confirm();
      } catch (e) {
        const err = e as Error;
        expect(err.message).to.include(
          `0x${SmartWalletErrors.TransactionNotReady.code.toString(16)}`
        );
      }

      const sleepTime = eta.sub(new BN(Date.now() / 1000)).add(new BN(1));
      await sleep(sleepTime.toNumber() * 1000);

      await expectTX(
        (
          await smartWalletWrapper.executeTransaction({
            transactionKey,
            owner: ownerC.publicKey,
          })
        ).addSigners(ownerC),
        "execute transaction"
      ).to.be.fulfilled;

      await smartWalletWrapper.reloadData();
      expect(smartWalletWrapper.data.ownerSetSeqno).to.equal(1);
      expect(smartWalletWrapper.data.threshold).to.bignumber.equal(threshold);
      expect(smartWalletWrapper.data.owners).to.deep.equal(newOwners);
    });
  });
});
