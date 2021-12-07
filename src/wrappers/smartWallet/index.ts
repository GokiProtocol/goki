import type { AugmentedProvider } from "@saberhq/solana-contrib";
import { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { SystemProgram } from "@solana/web3.js";
import BN from "bn.js";

import type {
  SmartWalletData,
  SmartWalletProgram,
  SmartWalletTransactionData,
} from "../../programs";
import type { GokiSDK } from "../../sdk";
import { findTransactionAddress, findWalletDerivedAddress } from "./pda";
import type {
  InitSmartWalletWrapperArgs,
  NewTransactionArgs,
  PendingSmartWalletTransaction,
} from "./types";

export * from "./pda";
export * from "./types";

export class SmartWalletWrapper {
  public readonly bump: number;
  public readonly key: PublicKey;
  public readonly program: SmartWalletProgram;
  private _data?: SmartWalletData;

  constructor(public readonly sdk: GokiSDK, args: InitSmartWalletWrapperArgs) {
    this.bump = args.bump;
    this.key = args.key;
    this._data = args.data;
    this.program = sdk.programs.SmartWallet;
  }

  get provider(): AugmentedProvider {
    return this.sdk.provider;
  }

  get data(): SmartWalletData | undefined {
    return this._data;
  }

  /**
   * reloadData
   */
  async reloadData(): Promise<SmartWalletData> {
    this._data = await this.sdk.programs.SmartWallet.account.smartWallet.fetch(
      this.key
    );
    return this._data;
  }

  /**
   * Proposes a new transaction.
   * @returns
   */
  async newTransaction({
    proposer = this.provider.wallet.publicKey,
    payer = this.provider.wallet.publicKey,
    instructions: ixs,
    eta,
  }: NewTransactionArgs): Promise<PendingSmartWalletTransaction> {
    const index = (await this.reloadData()).numTransactions.toNumber();
    const [txKey, txBump] = await findTransactionAddress(this.key, index);
    const accounts = {
      smartWallet: this.key,
      transaction: txKey,
      proposer,
      payer,
      systemProgram: SystemProgram.programId,
    };
    const instructions: TransactionInstruction[] = [];
    if (eta === undefined) {
      instructions.push(
        this.program.instruction.createTransaction(txBump, ixs, {
          accounts,
        })
      );
    } else {
      instructions.push(
        this.program.instruction.createTransactionWithTimelock(
          txBump,
          ixs,
          eta,
          {
            accounts,
          }
        )
      );
    }

    return {
      transactionKey: txKey,
      tx: new TransactionEnvelope(this.provider, instructions),
      index,
    };
  }

  /**
   * Creates a new transaction from an envelope.
   * @returns
   */
  async newTransactionFromEnvelope({
    tx,
    ...args
  }: Omit<NewTransactionArgs, "instructions"> & {
    tx: TransactionEnvelope;
  }): Promise<PendingSmartWalletTransaction> {
    return this.newTransaction({
      ...args,
      instructions: tx.instructions,
    });
  }

  /**
   * Fetches a transaction by its index.
   */
  async fetchTransactionByIndex(
    index: number
  ): Promise<SmartWalletTransactionData | null> {
    const [txKey] = await findTransactionAddress(this.key, index);
    return await this.program.account.transaction.fetchNullable(txKey);
  }

  /**
   * fetchTransaction
   */
  async fetchTransaction(key: PublicKey): Promise<SmartWalletTransactionData> {
    return await this.program.account.transaction.fetch(key);
  }

  /**
   * Approves a transaction.
   */
  approveTransaction(
    transactionKey: PublicKey,
    owner: PublicKey = this.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.provider, [
      this.program.instruction.approve({
        accounts: {
          smartWallet: this.key,
          transaction: transactionKey,
          owner,
        },
      }),
    ]);
  }

  /**
   * Executes a transaction as the Smart Wallet.
   */
  async executeTransaction({
    transactionKey,
    owner = this.provider.wallet.publicKey,
  }: {
    transactionKey: PublicKey;
    owner?: PublicKey;
  }): Promise<TransactionEnvelope> {
    const ix = this.program.instruction.executeTransaction(
      await this.fetchExecuteTransactionContext({ transactionKey, owner })
    );
    return new TransactionEnvelope(this.provider, [ix]);
  }

  /**
   * Finds the derived wallet address and bump of a given index.
   * @param index
   * @returns
   */
  async findWalletDerivedAddress(index: number): Promise<[PublicKey, number]> {
    return await findWalletDerivedAddress(this.key, index);
  }

  private async fetchExecuteTransactionContext({
    transactionKey,
    owner = this.provider.wallet.publicKey,
    walletDerivedAddress = null,
  }: {
    transactionKey: PublicKey;
    owner?: PublicKey;
    walletDerivedAddress?: PublicKey | null;
  }) {
    const data = await this.fetchTransaction(transactionKey);
    return {
      accounts: {
        smartWallet: this.key,
        transaction: transactionKey,
        owner,
      },
      remainingAccounts: data.instructions.flatMap((ix) => [
        {
          pubkey: ix.programId,
          isSigner: false,
          isWritable: false,
        },
        ...ix.keys.map((k) => {
          if (
            walletDerivedAddress &&
            k.isSigner &&
            k.pubkey.equals(walletDerivedAddress)
          ) {
            return {
              ...k,
              isSigner: false,
            };
          }
          if (k.isSigner && k.pubkey.equals(this.key)) {
            return {
              ...k,
              isSigner: false,
            };
          }
          return k;
        }),
      ]),
    };
  }

  /**
   * Executes a transaction using a wallet-derived address.
   */
  async executeTransactionDerived({
    transactionKey,
    walletIndex,
    owner = this.provider.wallet.publicKey,
  }: {
    transactionKey: PublicKey;
    walletIndex: number;
    owner?: PublicKey;
  }): Promise<TransactionEnvelope> {
    const [walletDerivedAddress, walletBump] =
      await this.findWalletDerivedAddress(walletIndex);
    const ix = this.program.instruction.executeTransactionDerived(
      new BN(walletIndex),
      walletBump,
      await this.fetchExecuteTransactionContext({
        transactionKey,
        owner,
        walletDerivedAddress,
      })
    );
    return new TransactionEnvelope(this.provider, [ix]);
  }

  /**
   * setOwners
   */
  setOwners(owners: PublicKey[]): TransactionEnvelope {
    const ix = this.program.instruction.setOwners(owners, {
      accounts: {
        smartWallet: this.key,
      },
    });
    return new TransactionEnvelope(this.provider, [ix]);
  }

  /**
   * changeThreshold
   */
  changeThreshold(threshold: number): TransactionEnvelope {
    const ix = this.program.instruction.changeThreshold(new BN(threshold), {
      accounts: {
        smartWallet: this.key,
      },
    });
    return new TransactionEnvelope(this.provider, [ix]);
  }

  /**
   * Loads a SmartWallet
   */
  static async load(sdk: GokiSDK, key: PublicKey): Promise<SmartWalletWrapper> {
    const data = await sdk.programs.SmartWallet.account.smartWallet.fetch(key);
    return new SmartWalletWrapper(sdk, {
      key,
      data,
      bump: data.bump,
      base: data.base,
    });
  }
}
