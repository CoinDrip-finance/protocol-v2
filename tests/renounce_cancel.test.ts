import { expect, test } from "vitest";
import { e } from "xsuite";

import { ERR_CANCEL_ONLY_SENDER, ERR_CANT_CANCEL } from "./errors";
import { createStream, getStream } from "./utils";

test("Non-cancelable stream", async (ctx) => {
  const streamId = await createStream(ctx, 600, 0, false);

  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 50_000_000,
      funcName: "renounceCancelStream",
      funcArgs: [e.U64(streamId)],
      value: 0,
    })
    .assertFail({ message: ERR_CANT_CANCEL });
});

test("Renounce cancel only by sender", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.recipient_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 50_000_000,
      funcName: "renounceCancelStream",
      funcArgs: [e.U64(streamId)],
      value: 0,
    })
    .assertFail({ message: ERR_CANCEL_ONLY_SENDER });
});

test("Renounce cancel successfully", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  let stream = await getStream(ctx, streamId);
  expect(stream.can_cancel).toEqual(true);

  await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "renounceCancelStream",
    funcArgs: [e.U64(streamId)],
    value: 0,
  });

  stream = await getStream(ctx, streamId);
  expect(stream.can_cancel).toEqual(false);
});
