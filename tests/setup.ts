import { beforeEach } from 'vitest';
import { e, SContract, SWallet, SWorld } from 'xsuite';

declare module "vitest" {
  export interface TestContext {
    world: SWorld;
    deployer: SWallet;
    contract: SContract;
    sender_wallet: SWallet;
    recipient_wallet: SWallet;

    stream_nft_token_identifier: string;
    payment_esdt_token_identifier: string;
  }
}

const STREAM_NFT_TOKEN_IDENTIFIER = "DRIP-93cadd";
const PAYMENT_ESDT_TOKEN_IDENTIFIER = "TEST-93cadd";
export const PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING = "TESTR-93cadd";

beforeEach(async (ctx) => {
  ctx.world = await SWorld.start();
  ctx.deployer = await ctx.world.createWallet();
  const { contract } = await ctx.deployer.deployContract({
    code: "file:output/coindrip.wasm",
    codeMetadata: [],
    gasLimit: 100_000_000,
  });

  ctx.contract = contract;

  await ctx.contract.setAccount({
    ...(await ctx.contract.getAccount()),
    kvs: [
      e.kvs.Mapper("streamNftToken").Value(e.Str(STREAM_NFT_TOKEN_IDENTIFIER)),
      e.kvs.Esdts([
        {
          id: STREAM_NFT_TOKEN_IDENTIFIER,
          roles: [
            "ESDTRoleLocalBurn",
            "ESDTRoleLocalMint",
            "ESDTRoleNFTCreate",
            "ESDTRoleNFTUpdateAttributes",
            "ESDTRoleNFTBurn",
          ],
        },
      ]),
    ],
  });

  ctx.sender_wallet = await ctx.world.createWallet({
    balance: 100,
    kvs: [
      e.kvs.Esdts([
        {
          id: PAYMENT_ESDT_TOKEN_IDENTIFIER,
          amount: 6_000,
        },
        {
          id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING,
          amount: 10,
        },
      ]),
    ],
  });
  ctx.recipient_wallet = await ctx.world.createWallet();

  ctx.stream_nft_token_identifier = STREAM_NFT_TOKEN_IDENTIFIER;
  ctx.payment_esdt_token_identifier = PAYMENT_ESDT_TOKEN_IDENTIFIER;

  return async () => {
    await ctx.world.terminate();
  };
});
