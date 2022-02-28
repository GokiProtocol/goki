import { expectTXTable } from "@saberhq/chai-solana";
import type { PublicKey } from "@solana/web3.js";
import { PACKET_DATA_SIZE } from "@solana/web3.js";
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
});
