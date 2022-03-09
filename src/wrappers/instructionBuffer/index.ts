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
    executer: PublicKey = this.sdk.provider.wallet.publicKey,
    writer: PublicKey = this.sdk.provider.wallet.publicKey,
    bufferAccount: Keypair = Keypair.generate()
  ): Promise<PendingBuffer> {
    const tx = new TransactionEnvelope(
      this.sdk.provider,
      [
        await this.program.account.instructionBuffer.createInstruction(
          bufferAccount,
          this.program.account.transaction.size + bufferSize
        ),
        this.program.instruction.initIxBuffer(eta, executer, writer, {
          accounts: {
            smartWallet,
            buffer: bufferAccount.publicKey,
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

  /**
   * Finalize an instruction buffer.
   */
  finalizeBuffer(
    buffer: PublicKey,
    writer: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.finalizeBuffer({
        accounts: {
          buffer,
          writer,
        },
      }),
    ]);
  }

  /**
   * Executes an instruction from the buffer.
   */
  async executeInstruction(
    buffer: PublicKey,
    accountMetas: AccountMeta[],
    executor: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    const bufferData = await this.loadData(buffer);
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.executeBufferIx({
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
   * Write an instruction to the buffer.
   */
  writeInstruction(
    ix: TransactionInstruction,
    buffer: PublicKey,
    writer: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.writeBuffer(ix, {
        accounts: {
          buffer,
          writer,
        },
      }),
    ]);
  }
}
