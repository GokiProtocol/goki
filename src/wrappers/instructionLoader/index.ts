import { TransactionEnvelope } from "@saberhq/solana-contrib";
import type {
  AccountMeta,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";

import type { SmartWalletProgram } from "../../programs";
import type { InstructionBufferData } from "../../programs/smartWallet";
import type { GokiSDK } from "../../sdk";
import type { BufferRole, PendingBuffer } from "./types";

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
    admin: PublicKey = this.sdk.provider.wallet.publicKey,
    bufferAccount: Keypair = Keypair.generate()
  ): Promise<PendingBuffer> {
    const tx = new TransactionEnvelope(
      this.sdk.provider,
      [
        await this.program.account.instructionBuffer.createInstruction(
          bufferAccount,
          this.program.account.instructionBuffer.size + bufferSize
        ),
        this.program.instruction.initIxBuffer({
          accounts: {
            buffer: bufferAccount.publicKey,
            admin,
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
    bufferAccount: PublicKey,
    writer: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.closeIxBuffer({
        accounts: {
          buffer: bufferAccount,
          writer,
        },
      }),
    ]);
  }

  /**
   * Executes an instruction from the buffer.
   */
  executeInstruction(
    bufferAccount: PublicKey,
    accountMetas: AccountMeta[]
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.executeIx({
        accounts: {
          buffer: bufferAccount,
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
    bufferAccount: PublicKey,
    writer: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.writeIx(ix, {
        accounts: {
          buffer: bufferAccount,
          writer,
        },
      }),
    ]);
  }

  /**
   * Set the executor for the instruction buffer.
   */
  setExecutor(
    bufferAccount: PublicKey,
    role: BufferRole,
    roleKey: PublicKey,
    admin: PublicKey = this.sdk.provider.wallet.publicKey
  ): TransactionEnvelope {
    return new TransactionEnvelope(this.sdk.provider, [
      this.program.instruction.setBufferRole(role, roleKey, {
        accounts: {
          buffer: bufferAccount,
          admin,
        },
      }),
    ]);
  }
}
