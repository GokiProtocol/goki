import { TOKEN_PROGRAM_ID } from "@project-serum/anchor/dist/cjs/utils/token";
import { expectTXTable } from "@saberhq/chai-solana";
import {
  createMemoInstruction,
  MEMO_PROGRAM_ID,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import { MintLayout, SPLToken } from "@saberhq/token-utils";
import {
  Keypair,
  PACKET_DATA_SIZE,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { expect } from "chai";

import { makeSDK } from "./workspace";

describe("instruction loader", () => {
  const sdk = makeSDK();
  const BUFFER_SIZE = PACKET_DATA_SIZE * 30;

  let bufferAccount: PublicKey;

  beforeEach(async () => {
    const { bufferAccount: bufferAccountInner, tx } =
      await sdk.instructionLoader.initBuffer(BUFFER_SIZE);
    await expectTXTable(tx, "initialize buffer").to.be.fulfilled;

    bufferAccount = bufferAccountInner;
  });

  it("Buffer was initialized", async () => {
    const bufferData = await sdk.instructionLoader.loadBufferData(
      bufferAccount
    );
    expect(bufferData.execCount).to.eql(0);
    expect(bufferData.writer).to.eqAddress(sdk.provider.wallet.publicKey);
    expect(bufferData.stagedTxInstructions).eql([]);
    expect(bufferData.executor).to.eqAddress(PublicKey.default);
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
    const writeTx = sdk.instructionLoader.writeInstruction(ix, bufferAccount);
    await expectTXTable(writeTx, "write memo instruction to buffer").to.be
      .fulfilled;
    const execTx = sdk.instructionLoader.executeInstruction(bufferAccount, [
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
    ]);
    execTx.addSigners(newAccountKP);
    await expectTXTable(execTx, "execute memo instruction off buffer").to.be
      .fulfilled;
    const newAccountInfo = await sdk.provider.getAccountInfo(
      newAccountKP.publicKey
    );
    expect(newAccountInfo?.accountInfo.owner).to.eqAddress(TOKEN_PROGRAM_ID);

    const bufferData = await sdk.instructionLoader.loadBufferData(
      bufferAccount
    );
    expect(bufferData.execCount).to.be.eq(1);
  });

  it("Test write and execute multiple instructions", async () => {
    const signers = new Array(3).fill(null).map(() => Keypair.generate());
    const writeTXs = signers.map((s) =>
      sdk.instructionLoader.writeInstruction(
        createMemoInstruction("test", [s.publicKey]),
        bufferAccount
      )
    );
    const writeTx = TransactionEnvelope.combineAll(...writeTXs);
    await expectTXTable(writeTx, "write memo instructions to buffer").to.be
      .fulfilled;

    const executeTXs = signers.map((s) => {
      const tx = sdk.instructionLoader.executeInstruction(bufferAccount, [
        {
          pubkey: MEMO_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: s.publicKey,
          isSigner: true,
          isWritable: false,
        },
      ]);
      tx.addSigners(s);
      return tx;
    });
    const execTx = TransactionEnvelope.combineAll(...executeTXs);
    await expectTXTable(execTx, "execute memo instructions off buffer").to.be
      .fulfilled;

    const bufferData = await sdk.instructionLoader.loadBufferData(
      bufferAccount
    );
    expect(bufferData.execCount).to.be.eq(3);
  });

  it("Close instruction buffer", async () => {
    const tx = sdk.instructionLoader.closeBuffer(bufferAccount);
    await expectTXTable(tx, "close buffer").to.be.fulfilled;

    const bufferAccountInfo = await sdk.provider.getAccountInfo(bufferAccount);
    expect(bufferAccountInfo).to.be.null;
  });
});
