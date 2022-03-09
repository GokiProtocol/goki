import type { PublicKey, TransactionEnvelope } from "@saberhq/solana-contrib";

/**
 * BufferRoles.
 */
export enum BufferRoles {
  Admin = 1,
  Writer = 2,
  Executer = 3,
}

export type PendingBuffer = {
  tx: TransactionEnvelope;
  bufferAccount: PublicKey;
};
