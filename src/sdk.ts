import type { Address } from "@project-serum/anchor";
import { BN, Program, Provider as AnchorProvider } from "@project-serum/anchor";
import type { Provider } from "@saberhq/solana-contrib";
import {
  DEFAULT_PROVIDER_OPTIONS,
  SignerWallet,
  SolanaProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import type { ConfirmOptions, PublicKey, Signer } from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";
import mapValues from "lodash.mapvalues";
import invariant from "tiny-invariant";

import type { Programs } from "./constants";
import { GOKI_ADDRESSES, GOKI_IDLS } from "./constants";
import type { PendingSmartWallet } from "./wrappers/smartWallet";
import { findSmartWallet, SmartWalletWrapper } from "./wrappers/smartWallet";

/**
 * Goki SDK.
 */
export class GokiSDK {
  constructor(
    public readonly provider: Provider,
    public readonly programs: Programs
  ) {}

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  public withSigner(signer: Signer): GokiSDK {
    const provider = new SolanaProvider(
      this.provider.connection,
      this.provider.broadcaster,
      new SignerWallet(signer),
      this.provider.opts
    );
    return GokiSDK.load({
      provider,
      addresses: mapValues(this.programs, (v) => v.programId),
    });
  }

  get programList(): Program[] {
    return Object.values(this.programs) as Program[];
  }

  /**
   * loadSmartWallet
   */
  public loadSmartWallet(key: PublicKey): Promise<SmartWalletWrapper> {
    return SmartWalletWrapper.load(this, key);
  }

  /**
   * Create a new multisig account
   */
  public async newSmartWallet({
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
    base?: Keypair;
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
  public static load({
    provider,
    addresses = GOKI_ADDRESSES,
    confirmOptions,
  }: {
    // Provider
    provider: Provider;
    // Addresses of each program.
    addresses?: { [K in keyof Programs]?: Address };
    confirmOptions?: ConfirmOptions;
  }): GokiSDK {
    const allAddresses = { ...GOKI_ADDRESSES, ...addresses };
    const programs: Programs = mapValues(
      GOKI_ADDRESSES,
      (_: Address, programName: keyof Programs): Program => {
        const address = allAddresses[programName];
        const idl = GOKI_IDLS[programName];
        invariant(idl, `Unknown IDL: ${programName}`);
        const anchorProvider = new AnchorProvider(
          provider.connection,
          provider.wallet,
          confirmOptions ?? DEFAULT_PROVIDER_OPTIONS
        );
        return new Program(idl, address, anchorProvider) as unknown as Program;
      }
    ) as unknown as Programs;
    return new GokiSDK(provider, programs);
  }
}
