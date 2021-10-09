import type { AnchorDefined, AnchorTypes } from "@saberhq/anchor-contrib";
import type { AccountMeta } from "@solana/web3.js";

import type { SmartWalletIDL } from "../idls/smart_wallet";

export * from "../idls/smart_wallet";

export type SmartWalletTypes = AnchorTypes<
  SmartWalletIDL,
  {
    smartWallet: SmartWalletData;
    transaction: SmartWalletTransactionData;
  },
  {
    Instruction: SmartWalletInstruction;
    AccountMeta: AccountMeta;
  }
>;

type Accounts = SmartWalletTypes["Accounts"];
export type SmartWalletData = Accounts["SmartWallet"];
export type SmartWalletTransactionData = Accounts["Transaction"];

export type SmartWalletInstruction = Omit<
  AnchorDefined<SmartWalletIDL>["Instruction"],
  "keys"
> & {
  keys: AccountMeta[];
};

export type SmartWalletError = SmartWalletTypes["Error"];
export type SmartWalletEvents = SmartWalletTypes["Events"];
export type SmartWalletProgram = SmartWalletTypes["Program"];

export type WalletCreateEvent = SmartWalletEvents["WalletCreateEvent"];
export type WalletSetOwnersEvent = SmartWalletEvents["WalletSetOwnersEvent"];
export type WalletChangeThresholdEvent =
  SmartWalletEvents["WalletChangeThresholdEvent"];
export type TransactionCreateEvent =
  SmartWalletEvents["TransactionCreateEvent"];
export type TransactionApproveEvent =
  SmartWalletEvents["TransactionApproveEvent"];
export type TransactionExecuteEvent =
  SmartWalletEvents["TransactionExecuteEvent"];
