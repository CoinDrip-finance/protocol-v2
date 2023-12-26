import { expect, test } from "vitest";
import { e } from "xsuite";

import { ERR_BROKER_FEE_TOO_BIG } from "./errors";
import { generateStreamSegment, getStream, requireEgldBalance } from "./utils";

test("Stream created with broker fee", async (ctx) => {
  const createStreamResult = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [
      ctx.recipient_wallet,
      e.List(generateStreamSegment(90, 1, 100)),
      e.U64(0),
      e.Bool(true),
      e.Tuple(e.Addr(ctx.deployer.toTopBytes()), e.U(1000)),
    ],
    value: 100,
  });

  const streamId = parseInt(createStreamResult.returnData[0]);

  const stream = await getStream(ctx, streamId);

  expect(stream.deposit).toBe(90n);

  await requireEgldBalance(ctx, ctx.deployer, 10);
});

test("Stream created with invalid broker fee", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 200_000_000,
      funcName: "createStreamNow",
      funcArgs: [
        ctx.recipient_wallet,
        e.List(generateStreamSegment(90, 1, 100)),
        e.U64(0),
        e.Bool(true),
        e.Tuple(e.Addr(ctx.deployer.toTopBytes()), e.U(2000)),
      ],
      value: 100,
    })
    .assertFail({ message: ERR_BROKER_FEE_TOO_BIG });
});
