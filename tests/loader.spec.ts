import { TOKEN_PROGRAM_ID } from "@project-serum/anchor/dist/cjs/utils/token";
import { expectTXTable } from "@saberhq/chai-solana";
import { MintLayout, SPLToken } from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";
import { Keypair, PACKET_DATA_SIZE, SystemProgram } from "@solana/web3.js";
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
  });
});
