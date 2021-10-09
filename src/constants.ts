import { PublicKey } from "@solana/web3.js";

import { SmartWalletJSON } from "./idls/smart_wallet";
import { TokenSignerJSON } from "./idls/token_signer";
import type { SmartWalletProgram } from "./programs";

export interface Programs {
  SmartWallet: SmartWalletProgram;
}

export const GOKI_ADDRESSES = {
  SmartWallet: new PublicKey("GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH"),
  TokenSigner: new PublicKey("NFTUJzSHuUCsMMqMRJpB7PmbsaU7Wm51acdPk2FXMLn"),
};

export const GOKI_IDLS = {
  SmartWallet: SmartWalletJSON,
  TokenSigner: TokenSignerJSON,
};
