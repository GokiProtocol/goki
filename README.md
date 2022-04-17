# 🔑 goki

[![License](https://img.shields.io/badge/license-AGPL%203.0-blue)](https://github.com/GokiProtocol/goki/blob/master/LICENSE)
[![Build Status](https://img.shields.io/github/workflow/status/GokiProtocol/goki/E2E/master)](https://github.com/GokiProtocol/goki/actions/workflows/programs-e2e.yml?query=branch%3Amaster)
[![Contributors](https://img.shields.io/github/contributors/GokiProtocol/goki)](https://github.com/GokiProtocol/goki/graphs/contributors)

![Banner](/images/banner.jpeg)

Goki is a suite of programs for Solana key management and security.

It currently features:

- **Goki Smart Wallet:** A wallet loosely based on the [Serum](https://anchor.projectserum.com/build/3) implementation
- **Token Signer:** Allows signing transactions by holding an NFT or tokens

## Note

- **Goki is in active development, so all APIs are subject to change.**
- **This code is unaudited. Use at your own risk.**

## Packages

| Package                | Description                                       | Version                                                                                                             | Docs                                                                                |
| :--------------------- | :------------------------------------------------ | :------------------------------------------------------------------------------------------------------------------ | :---------------------------------------------------------------------------------- |
| `smart-wallet`         | Multisig Solana wallet with Timelock capabilities | [![Crates.io](https://img.shields.io/crates/v/smart-wallet)](https://crates.io/crates/smart-wallet)                 | [![Docs.rs](https://docs.rs/smart-wallet/badge.svg)](https://docs.rs/smart-wallet)  |
| `token-signer`         | Sign transactions by owning a token               | [![crates](https://img.shields.io/crates/v/token-signer)](https://crates.io/crates/token-signer)                    | [![Docs.rs](https://docs.rs/token-signer/badge.svg)](https://docs.rs/token-signer)  |
| `@gokiprotocol/client` | TypeScript SDK for Goki                           | [![npm](https://img.shields.io/npm/v/@gokiprotocol/client.svg)](https://www.npmjs.com/package/@gokiprotocol/client) | [![Docs](https://img.shields.io/badge/docs-typedoc-blue)](https://docs.goki.so/ts/) |

## Addresses

- **Smart Wallet:** [`GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH`](https://explorer.solana.com/address/GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH)
- **Token Signer:** [`NFTUJzSHuUCsMMqMRJpB7PmbsaU7Wm51acdPk2FXMLn`](https://explorer.solana.com/address/NFTUJzSHuUCsMMqMRJpB7PmbsaU7Wm51acdPk2FXMLn)

## Ecosystem

- [Tribeca](https://tribeca.so) - DAO toolkit.
- [goki-grinder](https://github.com/mralbertchen/goki-grinder) - Grinder for Goki wallet vanity addresses.

## Philosophy

Goki embraces the concept of gradually increasing decentralization: ownership should be able to shift from centralized to increasingly more decentralized accounts.

Our intended use case is to:

1. Use a local wallet for development, using the NFT key.
2. Once the project becomes more serious, send the NFT to a Ledger or other hardware wallet.
3. On mainnet/production, use the `multisig` wallet.
4. Once sufficient traction has been established, send the NFTs to the DAO.

### Granularity

Goki also allows for granular access control: NFTs are cheap to create, so it should be possible to create an NFT for every possible ownership or role within a protocol.

NFTs also support metadata via protocols such as [Metaplex](https://www.notion.so/Metaplex-Developer-Guide-afefbc19841744c28587ab948a08cfac), so it should be much easier to manage the different roles and permissions with visual NFT names.

#### Real world example: Uniswap

Imagine you have an AMM that has the following roles:

- Upgrading the "factory" contract
- Upgrading the "router" contract
- Setting protocol fees

These three roles are all very different in risk and importance.

- A factory contract is extremely sensitive, as it would affect all swaps on the platform. This should only be upgraded by the team, but in the future the keys to the contract should be "burned".
- The router is non custodial, so it should be owned by the team's multisig-- ideally one with a low threshold of execution.
- Protocol fees should be set by the DAO.

Using an NFT here makes it easier for users and the community to track and understand the transition of power as a DAO evolves.

## License

Goki Protocol is licensed under the GNU Affero General Public License v3.0.

In short, this means that any changes to this code must be made open source and available under the AGPL-v3.0 license, even if only used privately. If you have a need to use this program and cannot respect the terms of the license, please message us our team directly at [team@goki.so](mailto:team@goki.so).
