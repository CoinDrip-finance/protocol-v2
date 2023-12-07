import { expect, test } from "vitest";
import { e } from "xsuite";

import { ERR_CLIFF_TOO_BIG, ERR_ZERO_CLAIM } from "./errors";
import { claimFromStream, createStream, generateStreamSegment, getStream, requireValidStreamNft } from "./utils";

test("Valid cliff", async (ctx) => {
  const streamId = await createStream(ctx, 632, 200);

  const stream = await getStream(ctx, streamId);

  expect(stream.cliff).toBe(200n);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 100,
  });

  await claimFromStream(ctx, streamId).assertFail({
    message: ERR_ZERO_CLAIM,
  });

  await requireValidStreamNft(ctx);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 220,
  });

  await claimFromStream(ctx, streamId);

  await requireValidStreamNft(ctx);

  const balance = await ctx.recipient_wallet.getAccountBalance();
  expect(balance).toBeGreaterThan(0n);
});

test("Cliff too big", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 200_000_000,
      funcName: "createStreamNow",
      funcArgs: [ctx.recipient_wallet, generateStreamSegment(1, 1, 632), e.U64(700)],
      value: 1,
    })
    .assertFail({ message: ERR_CLIFF_TOO_BIG });
});
