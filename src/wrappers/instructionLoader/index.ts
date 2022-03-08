import { TransactionEnvelope } from "@saberhq/solana-contrib";
import type {
  AccountMeta,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";
import BN from "bn.js";

import type { SmartWalletProgram } from "../../programs";
import type { InstructionBufferData } from "../../programs/smartWallet";
import type { GokiSDK } from "../../sdk";
import { findBufferAddress } from "../smartWallet/pda";
import type { PendingBuffer } from "./types";

export class InstructionLoaderWrapper {
  readonly program: SmartWalletProgram;

  constructor(readonly sdk: GokiSDK) {
    this.program = sdk.programs.SmartWallet;
  }

  /**
   * loadBufferData
   * @returns
   */
  async loadBufferData(
    bufferAccount: PublicKey
  ): Promise<InstructionBufferData> {
    return await this.program.account.instructionBuffer.fetch(bufferAccount);
  }

  /**
   * Initialize a loader buffer.
   */
  async initBuffer(
    bufferSize: number,
    smartWallet: PublicKey,
    eta: BN = new BN(-1),
    proposer: PublicKey = this.sdk.provider.wallet.publicKey,
    writer: PublicKey = this.sdk.provider.wallet.publicKey,
    txAccount: Keypair = Keypair.generate()
  ): Promise<PendingBuffer> {
    const [buffer] = await findBufferAddress(txAccount.publicKey);
    const tx = new TransactionEnvelope(
      this.sdk.provider,
      [
        await this.program.account.instructionBuffer.createInstruction(
          txAccount,
          this.program.account.transaction.size + bufferSize
        ),
        this.program.instruction.initBuffer(eta, {
          accounts: {
            buffer,
            proposer,
            smartWallet,
            transaction: txAccount.publicKey,
            writer,
            payer: this.sdk.provider.wallet.publicKey,
            systemProgram: SystemProgram.programId,
          },
        }),
      ],
      [txAccount]
    );

    return {
      tx,
      buffer,
      txAccount: txAccount.publicKey,
    };
  }

  /**
   * Finalize an instruction buffer.
   */
  async finalizeBuffer(
    buffer: PublicKey,
    owner: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    const bufferData = await this.loadBufferData(buffer);
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.finalizeBuffer({
        accounts: {
          buffer,
          smartWallet: bufferData.smartWallet,
          transaction: bufferData.transaction,
          owner,
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
    owner: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    const bufferData = await this.loadBufferData(buffer);
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.executeFromBuffer({
        accounts: {
          buffer,
          smartWallet: bufferData.smartWallet,
          transaction: bufferData.transaction,
          owner,
        },
        remainingAccounts: accountMetas,
      }),
    ]);
  }

  /**
   * Write an instruction to the buffer.
   */
  async writeInstruction(
    buffer: PublicKey,
    ix: TransactionInstruction,
    writer: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    const bufferData = await this.loadBufferData(buffer);
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.writeToBuffer(ix, {
        accounts: {
          transaction: bufferData.transaction,
          buffer,
          writer,
        },
      }),
    ]);
  }
}
