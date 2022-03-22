import { expectTX, expectTXTable } from "@saberhq/chai-solana";
import {
  createMemoInstruction,
  MEMO_PROGRAM_ID,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import { MintLayout, SPLToken, TOKEN_PROGRAM_ID } from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";
import { Keypair, PACKET_DATA_SIZE, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";
import { expect } from "chai";

import type { SmartWalletWrapper } from "../src";
import { makeSDK } from "./workspace";

describe("instruction loader", () => {
  const sdk = makeSDK();
  const DEFAULT_BUNDLE_INDEX = 0;
  const BUFFER_SIZE = PACKET_DATA_SIZE * 30;

  let smartWalletW: SmartWalletWrapper;
  let bufferAccount: PublicKey;

  before(async () => {
    const { smartWalletWrapper: wrapperInner, tx } = await sdk.newSmartWallet({
      numOwners: 1,
      owners: [sdk.provider.wallet.publicKey],
      threshold: new BN(1),
    });
    await expectTX(tx, "create new smartWallet").to.be.fulfilled;

    smartWalletW = wrapperInner;
    await smartWalletW.reloadData();
  });

  beforeEach(async () => {
    const { bufferAccount: bufferAccountInner, tx } =
      await sdk.instructionBuffer.initBuffer({
        bufferSize: BUFFER_SIZE,
        smartWallet: smartWalletW.key,
      });
    await expectTXTable(tx, "initialize buffer").to.be.fulfilled;

    bufferAccount = bufferAccountInner;
  });

  it("Buffer was initialized", async () => {
    const bufferData = await sdk.instructionBuffer.loadData(bufferAccount);
    expect(bufferData.bundles.length).to.eql(0);
    expect(bufferData.ownerSetSeqno).to.eql(smartWalletW.data?.ownerSetSeqno);
    expect(bufferData.eta).to.bignumber.eql(new BN(-1));
    expect(bufferData.authority).to.eqAddress(sdk.provider.wallet.publicKey);
    expect(bufferData.executor).eqAddress(sdk.provider.wallet.publicKey);
    expect(bufferData.smartWallet).to.eqAddress(smartWalletW.key);
  });

  it("Intialize buffer with bundles", async () => {
    const expectedNumBundles = 30;
    const { bufferAccount: bufferAccountWithBunldes, tx } =
      await sdk.instructionBuffer.initBuffer({
        bufferSize: BUFFER_SIZE,
        smartWallet: smartWalletW.key,
        numBundles: expectedNumBundles,
      });
    await expectTXTable(tx, "initialize buffer with bundles").to.be.fulfilled;
    const bufferData = await sdk.instructionBuffer.loadData(
      bufferAccountWithBunldes
    );
    expect(bufferData.bundles.length).to.eql(expectedNumBundles);
    expect(bufferData.ownerSetSeqno).to.eql(smartWalletW.data?.ownerSetSeqno);
    expect(bufferData.eta).to.bignumber.eql(new BN(-1));
    expect(bufferData.authority).to.eqAddress(sdk.provider.wallet.publicKey);
    expect(bufferData.executor).eqAddress(sdk.provider.wallet.publicKey);
    expect(bufferData.smartWallet).to.eqAddress(smartWalletW.key);
  });

  it("Test write and execute instruction", async () => {
    const newAccountKP = Keypair.generate();
    const ix = SystemProgram.createAccount({
      fromPubkey: sdk.provider.wallet.publicKey,
      newAccountPubkey: newAccountKP.publicKey,
      space: MintLayout.span,
      lamports: await SPLToken.getMinBalanceRentForExemptMint(
        sdk.provider.connection
      ),
      programId: TOKEN_PROGRAM_ID,
    });
    const writeTx = sdk.instructionBuffer.appendInstruction(
      bufferAccount,
      DEFAULT_BUNDLE_INDEX,
      ix
    );
    await expectTXTable(writeTx, "write memo instruction to buffer").to.be
      .fulfilled;

    const finalizeTx = sdk.instructionBuffer.finalizeBuffer(bufferAccount);
    const execTx = await sdk.instructionBuffer.executeInstruction(
      bufferAccount,
      DEFAULT_BUNDLE_INDEX,
      [
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: sdk.provider.wallet.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: newAccountKP.publicKey,
          isSigner: true,
          isWritable: true,
        },
      ]
    );
    execTx.addSigners(newAccountKP);
    await expectTXTable(
      finalizeTx.combine(execTx),
      "execute memo instruction off buffer"
    ).to.be.fulfilled;
    const newAccountInfo = await sdk.provider.getAccountInfo(
      newAccountKP.publicKey
    );
    expect(newAccountInfo?.accountInfo.owner).to.eqAddress(TOKEN_PROGRAM_ID);

    const bufferData = await sdk.instructionBuffer.loadData(bufferAccount);
    expect(bufferData.bundles[DEFAULT_BUNDLE_INDEX]?.isExecuted).to.be.true;
  });

  it("Test write and execute multiple instructions", async () => {
    const signers = new Array(3).fill(null).map(() => Keypair.generate());
    const writeTXs = signers.map((s) =>
      sdk.instructionBuffer.appendInstruction(
        bufferAccount,
        DEFAULT_BUNDLE_INDEX,
        createMemoInstruction("test", [s.publicKey])
      )
    );
    const writeTx = TransactionEnvelope.combineAll(...writeTXs);
    await expectTXTable(writeTx, "write memo instructions to buffer").to.be
      .fulfilled;

    const signerAccountMetas = signers.map((s) => ({
      pubkey: s.publicKey,
      isSigner: true,
      isWritable: false,
    }));

    const execTx = await sdk.instructionBuffer.executeInstruction(
      bufferAccount,
      DEFAULT_BUNDLE_INDEX,
      [
        { pubkey: MEMO_PROGRAM_ID, isSigner: false, isWritable: false },
        ...signerAccountMetas,
      ]
    );
    execTx.addSigners(...signers);

    const tx = sdk.instructionBuffer
      .finalizeBuffer(bufferAccount)
      .combine(execTx);

    const receipt = await tx.confirm();
    receipt.printLogs();

    const joinedLogs = receipt.response.meta?.logMessages?.join("");
    expect(
      signers.every((s) =>
        joinedLogs?.includes(`Signed by ${s.publicKey.toString()}`)
      )
    ).to.be.true;

    const bufferData = await sdk.instructionBuffer.loadData(bufferAccount);
    expect(bufferData.bundles[DEFAULT_BUNDLE_INDEX]?.isExecuted).to.be.true;
  });

  it("Test write and execute bundles in parallel", async () => {
    const numBundles = 4;
    const signers = new Array(numBundles)
      .fill(null)
      .map(() => Keypair.generate());

    const writeTXs = signers.map((s, bundleIndex) =>
      sdk.instructionBuffer.appendInstruction(
        bufferAccount,
        bundleIndex,
        createMemoInstruction("test", [s.publicKey])
      )
    );
    await expectTXTable(
      TransactionEnvelope.combineAll(...writeTXs),
      "write memo instructions to buffer"
    ).to.be.fulfilled;

    let bufferData = await sdk.instructionBuffer.loadData(bufferAccount);
    expect(bufferData.bundles.length).to.eq(numBundles);

    const execTxs = await Promise.all(
      signers.map(async (s, bundleIndex) => {
        const tx = await sdk.instructionBuffer.executeInstruction(
          bufferAccount,
          bundleIndex,
          [
            { pubkey: MEMO_PROGRAM_ID, isSigner: false, isWritable: false },
            { pubkey: s.publicKey, isSigner: true, isWritable: false },
          ]
        );
        tx.addSigners(s);
        return tx;
      })
    );
    const finalizeTx = sdk.instructionBuffer.finalizeBuffer(bufferAccount);
    const tx = TransactionEnvelope.combineAll(finalizeTx, ...execTxs);

    const receipt = await tx.confirm();
    receipt.printLogs();

    const joinedLogs = receipt.response.meta?.logMessages?.join("");
    expect(
      signers.every((s) =>
        joinedLogs?.includes(`Signed by ${s.publicKey.toString()}`)
      )
    ).to.be.true;

    bufferData = await sdk.instructionBuffer.loadData(bufferAccount);
    expect(signers.every((_, i) => bufferData.bundles[i]?.isExecuted)).to.be
      .true;
  });

  it("Cannot execute on buffer that is not finalized", async () => {
    const writeTx = sdk.instructionBuffer.appendInstruction(
      bufferAccount,
      DEFAULT_BUNDLE_INDEX,
      createMemoInstruction("test", [sdk.provider.wallet.publicKey])
    );
    await expectTXTable(writeTx, "write memo instruction to buffer").to.be
      .fulfilled;

    const execTx = await sdk.instructionBuffer.executeInstruction(
      bufferAccount,
      DEFAULT_BUNDLE_INDEX,
      [
        {
          pubkey: MEMO_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: sdk.provider.wallet.publicKey,
          isSigner: true,
          isWritable: false,
        },
      ]
    );

    await expectTX(
      execTx,
      "cannot execute on unfinalized buffer"
    ).to.be.rejectedWith("0x177d");
  });

  it("Close the instruction buffer", async () => {
    const tx = sdk.instructionBuffer.closeBuffer(bufferAccount);
    await expectTXTable(tx, "close buffer").to.be.fulfilled;

    expect(await sdk.provider.getAccountInfo(bufferAccount)).to.be.null;
  });
});
