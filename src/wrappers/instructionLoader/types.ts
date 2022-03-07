import type { PublicKey, TransactionEnvelope } from "@saberhq/solana-contrib";

/**
 * Side of a vote.
 */
export enum BufferRole {
  Admin = 0,
  Writer = 1,
  Executer = 2,
}

export type PendingBuffer = {
  tx: TransactionEnvelope;
  bufferAccount: PublicKey;
};
