import type { PublicKey, TransactionEnvelope } from "@saberhq/solana-contrib";

export type PendingBuffer = {
  tx: TransactionEnvelope;
  buffer: PublicKey;
  txAccount: PublicKey;
};
