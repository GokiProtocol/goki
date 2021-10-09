# token-signer

Allows the holder of an NFT to sign transactions as an address derived from the NFT.

As these keys must perform a CPI call, it is recommended to not use these for instructions that may require deep CPI calls or large numbers of BPF instructions. This is especially true if they are used in conjunction with the `smart_wallet` program, as that would cause the call stack to be at least 2 programs deep.

With these restrictions in mind, this can be useful for:

- Representing the ownership of the rights to upgrade a program
- Representing the ownership of an "admin account" that merely changes settings such as fees

## Use as an RBAC whitelist

This merely checks for the presence of at least one token in the user's wallet. One can use the token key as a way to allow multiple users to perform an action, similar to a 1/n multisig.

For example, let's say there is a function for auto-compounding via market orders on an AMM. This function should be gated, as it is vulnerable to flash loan exploits. By distributing a token to each member, the permissions can be assigned to trusted parties.

Note that the account cannot be revoked, so this may be suboptimal. A whitelist may make more sense.
