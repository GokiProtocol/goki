import { newProgramMap } from "@saberhq/anchor-contrib";
import type { AugmentedProvider, Provider } from "@saberhq/solana-contrib";
import {
  SolanaAugmentedProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import { u64 } from "@saberhq/token-utils";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";
import BN from "bn.js";
import mapValues from "lodash.mapvalues";

import type { Programs } from "./constants";
import { GOKI_ADDRESSES, GOKI_IDLS } from "./constants";
import { InstructionBufferWrapper } from "./wrappers/instructionBuffer";
import type { PendingSmartWallet } from "./wrappers/smartWallet";
import {
  findOwnerInvokerAddress,
  findSmartWallet,
  findSubaccountInfoAddress,
  findWalletDerivedAddress,
  SmartWalletWrapper,
} from "./wrappers/smartWallet";

/**
 * Goki SDK.
 */
export class GokiSDK {
  constructor(
    readonly provider: AugmentedProvider,
    readonly programs: Programs
  ) {}

  /**
   * Wrapper for the instruction buffer.
   */
  get instructionBuffer(): InstructionBufferWrapper {
    return new InstructionBufferWrapper(this);
  }

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  withSigner(signer: Signer): GokiSDK {
    return GokiSDK.load({
      provider: this.provider.withSigner(signer),
      addresses: mapValues(this.programs, (v) => v.programId),
    });
  }

  /**
   * loadSmartWallet
   */
  loadSmartWallet(key: PublicKey): Promise<SmartWalletWrapper> {
    return SmartWalletWrapper.load(this, key);
  }

  /**
   * Creates a subaccount info.
   * @returns
   */
  async createSubaccountInfo({
    smartWallet,
    index,
    type,
    payer = this.provider.wallet.publicKey,
  }: {
    smartWallet: PublicKey;
    index: number;
    type: "derived" | "ownerInvoker";
    payer?: PublicKey;
  }) {
    const [subaccount] =
      type === "derived"
        ? await findWalletDerivedAddress(smartWallet, index)
        : await findOwnerInvokerAddress(smartWallet, index);
    const [subaccountInfo, bump] = await findSubaccountInfoAddress(subaccount);
    return this.provider.newTX([
      this.programs.SmartWallet.instruction.createSubaccountInfo(
        bump,
        subaccount,
        smartWallet,
        new u64(index),
        {
          [type]: {},
        },
        {
          accounts: {
            subaccountInfo,
            payer,
            systemProgram: SystemProgram.programId,
          },
        }
      ),
    ]);
  }

  /**
   * Create a new multisig account
   */
  async newSmartWallet({
    owners,
    threshold,
    numOwners,
    base = Keypair.generate(),
    delay = new BN(0),
  }: {
    owners: PublicKey[];
    threshold: BN;
    /**
     * Number of owners in the smart wallet.
     */
    numOwners: number;
    base?: Signer;
    /**
     * Timelock delay in seconds
     */
    delay?: BN;
  }): Promise<PendingSmartWallet> {
    const [smartWallet, bump] = await findSmartWallet(base.publicKey);

    const ix = this.programs.SmartWallet.instruction.createSmartWallet(
      bump,
      numOwners,
      owners,
      threshold,
      delay,
      {
        accounts: {
          base: base.publicKey,
          smartWallet,
          payer: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        },
      }
    );

    return {
      smartWalletWrapper: new SmartWalletWrapper(this, {
        bump,
        key: smartWallet,
        base: base.publicKey,
      }),
      tx: new TransactionEnvelope(this.provider, [ix], [base]),
    };
  }

  /**
   * Loads the SDK.
   * @returns
   */
  static load({
    provider,
    addresses = GOKI_ADDRESSES,
  }: {
    // Provider
    provider: Provider;
    // Addresses of each program.
    addresses?: { [K in keyof Programs]?: PublicKey };
  }): GokiSDK {
    const allAddresses = { ...GOKI_ADDRESSES, ...addresses };
    const programs = newProgramMap<Programs>(provider, GOKI_IDLS, allAddresses);
    return new GokiSDK(new SolanaAugmentedProvider(provider), programs);
  }
}
