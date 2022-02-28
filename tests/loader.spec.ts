import { PACKET_DATA_SIZE } from "@solana/web3.js";

import { makeSDK } from "./workspace";

describe("instruction loader", () => {
  const sdk = makeSDK();
  const BUFFER_SIZE = PACKET_DATA_SIZE * 30;

  beforeEach(async () => {
    const { bufferAccount: bufferAccountInner, tx } = await sdk.initBuffer(
      BUFFER_SIZE
    );
    await expectTXTable(tx, "initialize buffer").to.be.fulfilled;

    bufferAccount = bufferAccountInner;
  });

  describe("", () => {});
});
