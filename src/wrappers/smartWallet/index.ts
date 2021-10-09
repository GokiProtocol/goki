import type { Provider } from "@saberhq/solana-contrib";
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
import { findTransactionAddress } from "./pda";
import type {
  InitSmartWalletWrapperArgs,
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

  get provider(): Provider {
    return this.sdk.provider;
  }

  get data(): SmartWalletData | undefined {
    return this._data;
  }

  /**
   * reloadData
   */
  public async reloadData(): Promise<SmartWalletData> {
    this._data = await this.sdk.programs.SmartWallet.account.smartWallet.fetch(
      this.key
    );
    return this._data;
  }

  /**
   * Loads a smartWallet
   */
  public static async load(
    sdk: GokiSDK,
    key: PublicKey
  ): Promise<SmartWalletWrapper> {
    const data = await sdk.programs.SmartWallet.account.smartWallet.fetch(key);
    return new SmartWalletWrapper(sdk, {
      key,
      data,
      bump: data.bump,
      base: data.base,
    });
  }

  public async newTransaction({
    proposer = this.provider.wallet.publicKey,
    payer = this.provider.wallet.publicKey,
    instruction,
    eta,
  }: {
    readonly proposer?: PublicKey;
    readonly payer?: PublicKey;
    readonly instruction: TransactionInstruction;
    readonly eta?: BN;
  }): Promise<PendingSmartWalletTransaction> {
    const [txKey, txBump] = await findTransactionAddress(
      this.key,
      (await this.reloadData()).numTransactions.toNumber()
    );
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
        this.program.instruction.createTransaction(txBump, instruction, {
          accounts,
        })
      );
    } else {
      instructions.push(
        this.program.instruction.createTransactionWithTimelock(
          txBump,
          instruction,
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
    };
  }

  /**
   * fetchTransaction
   */
  public async fetchTransaction(
    key: PublicKey
  ): Promise<SmartWalletTransactionData> {
    return await this.program.account.transaction.fetch(key);
  }

  /**
   * Approves a transaction.
   */
  public approveTransaction(
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
   * executeTransaction
   */
  public async executeTransaction({
    transactionKey,
    owner = this.provider.wallet.publicKey,
  }: {
    transactionKey: PublicKey;
    owner?: PublicKey;
  }): Promise<TransactionEnvelope> {
    const data = await this.fetchTransaction(transactionKey);
    const ix = this.program.instruction.executeTransaction({
      accounts: {
        smartWallet: this.key,
        transaction: transactionKey,
        owner,
      },
      remainingAccounts: [
        {
          pubkey: data.instruction.programId,
          isSigner: false,
          isWritable: false,
        },
        ...data.instruction.keys.map((k) => {
          if (k.pubkey.equals(this.key) && k.isSigner) {
            return {
              ...k,
              isSigner: false,
            };
          }
          return k;
        }),
      ],
    });

    return new TransactionEnvelope(this.provider, [ix]);
  }

  /**
   * setOwners
   */
  public setOwners(owners: PublicKey[]): TransactionEnvelope {
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
  public changeThreshold(threshold: number): TransactionEnvelope {
    const ix = this.program.instruction.changeThreshold(new BN(threshold), {
      accounts: {
        smartWallet: this.key,
      },
    });
    return new TransactionEnvelope(this.provider, [ix]);
  }
}
