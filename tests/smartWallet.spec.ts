import "chai-bn";

import * as anchor from "@project-serum/anchor";
import { expectTX } from "@saberhq/chai-solana";
import { TransactionInstruction } from "@solana/web3.js";
import { expect } from "chai";
import invariant from "tiny-invariant";

import { findSmartWallet } from "../src/wrappers/smartWallet";
import { makeSDK } from "./workspace";

describe("smartWallet", () => {
  const { BN, web3 } = anchor;
  const sdk = makeSDK();
  const program = sdk.programs.SmartWallet;

  it("Tests the smartWallet program", async () => {
    const smartWalletBase = web3.Keypair.generate();
    const numOwners = 10; // Big enough.

    const ownerA = web3.Keypair.generate();
    const ownerB = web3.Keypair.generate();
    const ownerC = web3.Keypair.generate();
    const ownerD = web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerC.publicKey];

    const threshold = new anchor.BN(2);
    const { smartWalletWrapper, tx } = await sdk.newSmartWallet({
      numOwners,
      owners,
      threshold,
      base: smartWalletBase,
    });
    await expectTX(tx, "create new smartWallet").to.be.fulfilled;
    // const pendingTx = await tx.send();
    // const confirmedTx = await pendingTx.wait();
    // confirmedTx.printLogs();

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

    const pendingTransaction = await smartWalletWrapper.newTransaction({
      proposer: ownerA.publicKey,
      instruction,
    });
    pendingTransaction.tx.signers.push(ownerA);
    await expectTX(
      pendingTransaction.tx,
      "create a tx to be processed by the smartWallet"
    ).to.be.fulfilled;

    const txAccount = await smartWalletWrapper.fetchTransaction(
      pendingTransaction.transactionKey
    );
    expect(txAccount.executedAt.toNumber()).to.equal(-1);
    expect(txAccount.ownerSetSeqno).to.equal(0);
    expect(txAccount.instruction.programId, "program id").to.eqAddress(
      program.programId
    );
    expect(txAccount.instruction.data, "data").to.deep.equal(data);
    expect(txAccount.instruction.keys, "keys").to.deep.equal(instruction.keys);
    expect(txAccount.smartWallet).to.eqAddress(smartWalletKey);

    // Other owner approves transaction.
    await expectTX(
      smartWalletWrapper
        .approveTransaction(pendingTransaction.transactionKey, ownerB.publicKey)
        .addSigners(ownerB)
    ).to.be.fulfilled;

    // Now that we've reached the threshold, send the transaction.
    await expectTX(
      (
        await smartWalletWrapper.executeTransaction({
          transactionKey: pendingTransaction.transactionKey,
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
