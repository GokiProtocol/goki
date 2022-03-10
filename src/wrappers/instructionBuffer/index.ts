import { TransactionEnvelope } from "@saberhq/solana-contrib";
import type {
  AccountMeta,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import BN from "bn.js";

import type { SmartWalletProgram } from "../../programs";
import type { InstructionBufferData } from "../../programs/smartWallet";
import type { GokiSDK } from "../../sdk";
import type { PendingBuffer } from "./types";

export class InstructionBufferWrapper {
  readonly program: SmartWalletProgram;

  constructor(readonly sdk: GokiSDK) {
    this.program = sdk.programs.SmartWallet;
  }

  /**
   * loadData
   * @returns
   */
  async loadData(loaderAccount: PublicKey): Promise<InstructionBufferData> {
    return await this.program.account.instructionBuffer.fetch(loaderAccount);
  }

  /**
   * Initialize a loader buffer.
   */
  async initBuffer(
    bufferSize: number,
    smartWallet: PublicKey,
    eta: BN = new BN(-1),
    authority: PublicKey = this.sdk.provider.wallet.publicKey,
    executor: PublicKey = this.sdk.provider.wallet.publicKey,
    bufferAccount: Keypair = Keypair.generate()
  ): Promise<PendingBuffer> {
    const tx = new TransactionEnvelope(
      this.sdk.provider,
      [
        await this.program.account.instructionBuffer.createInstruction(
          bufferAccount,
          this.program.account.transaction.size + bufferSize
        ),
        this.program.instruction.initIxBuffer(eta, {
          accounts: {
            buffer: bufferAccount.publicKey,
            authority,
            executor,
            smartWallet,
          },
        }),
      ],
      [bufferAccount]
    );

    return {
      tx,
      bufferAccount: bufferAccount.publicKey,
    };
  }

  closeBuffer(
    buffer: PublicKey,
    authorityOrExecutor: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.closeIxBuffer({
        accounts: {
          buffer,
          authorityOrExecutor,
        },
      }),
    ]);
  }

  /**
   * Finalize an instruction buffer.
   */
  finalizeBuffer(
    buffer: PublicKey,
    authority: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.finalizeBuffer({
        accounts: {
          buffer,
          authority,
        },
      }),
    ]);
  }

  /**
   * Executes an instruction from the buffer at the specified bundle index.
   */
  async executeInstruction(
    buffer: PublicKey,
    bundleIndex: number,
    accountMetas: AccountMeta[],
    executor: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    const bufferData = await this.loadData(buffer);
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.executeBufferIx(bundleIndex, {
        accounts: {
          buffer,
          executor,
          smartWallet: bufferData.smartWallet,
        },
        remainingAccounts: accountMetas,
      }),
    ]);
  }

  /**
   * Append an instruction to the buffer at the bundle specified by the bundle index.
   */
  appendInstruction(
    ix: TransactionInstruction,
    bundleIndex: number,
    buffer: PublicKey,
    authority: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.appendBufferIx(bundleIndex, ix, {
        accounts: {
          buffer,
          authority,
        },
      }),
    ]);
  }
}
