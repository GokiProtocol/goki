import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";

import type { SmartWalletData } from "../../programs";
import type { SmartWalletWrapper } from ".";

export type InitSmartWalletWrapperArgs = {
  readonly bump: number;
  readonly base: PublicKey;
  readonly key: PublicKey;
  readonly data?: SmartWalletData;
};

export type PendingSmartWallet = {
  readonly smartWalletWrapper: SmartWalletWrapper;
  readonly tx: TransactionEnvelope;
};

export type PendingSmartWalletTransaction = {
  readonly transactionKey: PublicKey;
  readonly tx: TransactionEnvelope;
};
