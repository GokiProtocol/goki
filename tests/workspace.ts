import * as anchor from "@project-serum/anchor";
import { chaiSolana } from "@saberhq/chai-solana";
import { SolanaProvider } from "@saberhq/solana-contrib";
import * as chai from "chai";

import type { Programs } from "../src";
import { GokiSDK } from "../src";

chai.use(chaiSolana);

export type Workspace = Programs;

export const makeSDK = (): GokiSDK => {
  const anchorProvider = anchor.Provider.env();
  anchor.setProvider(anchorProvider);

  const provider = SolanaProvider.load({
    connection: anchorProvider.connection,
    wallet: anchorProvider.wallet,
    opts: anchorProvider.opts,
  });
  return GokiSDK.load({
    provider,
  });
};
