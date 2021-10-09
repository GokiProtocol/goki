require("./.pnp.cjs").setup();
require("ts-node/register");

process.env.ANCHOR_PROVIDER_URL = "http://localhost:8899";
process.env.ANCHOR_WALLET = `${require("os").homedir()}/.config/solana/id.json`;
