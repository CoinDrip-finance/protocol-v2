import { expect, test } from "vitest";
import { d, e } from "xsuite";

import { ERR_PROTOCOL_FEE_ZERO } from "./errors";
import { generateStreamSegment, getStream } from "./utils";

test("Set protocol fee", async (ctx) => {
  await ctx.deployer.callContract({
    callee: ctx.contract,
    gasLimit: 10_000_000,
    funcName: "setProtocolFee",
    funcArgs: [e.Str("EGLD"), e.U(7_50n)],
    value: 0,
  });

  const { returnData, returnCode, returnMessage } = await ctx.world.query({
    callee: ctx.contract,
    funcName: "getProtocolFee",
    funcArgs: [e.Str("EGLD")],
  });

  const fee = d.U().topDecode(returnData[0]);

  expect(fee).toBe(7_50n);
});

test("Set invalid protocol fee", async (ctx) => {
  await ctx.deployer
    .callContract({
      callee: ctx.contract,
      gasLimit: 10_000_000,
      funcName: "setProtocolFee",
      funcArgs: [e.Str("EGLD"), e.U(0n)],
      value: 0,
    })
    .assertFail({ message: ERR_PROTOCOL_FEE_ZERO });
});

test("Remove protocol fee", async (ctx) => {
  await ctx.deployer.callContract({
    callee: ctx.contract,
    gasLimit: 10_000_000,
    funcName: "setProtocolFee",
    funcArgs: [e.Str("EGLD"), e.U(7_50n)],
    value: 0,
  });

  await ctx.deployer.callContract({
    callee: ctx.contract,
    gasLimit: 10_000_000,
    funcName: "removeProtocolFee",
    funcArgs: [e.Str("EGLD")],
    value: 0,
  });

  const { returnData, returnCode, returnMessage } = await ctx.world.query({
    callee: ctx.contract,
    funcName: "getProtocolFee",
    funcArgs: [e.Str("EGLD")],
  });

  const fee = d.U().topDecode(returnData[0]);

  expect(fee).toBe(0n);
});

test("Set protocol fee not owner", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 10_000_000,
      funcName: "setProtocolFee",
      funcArgs: [e.Str("EGLD"), e.U(7_50n)],
      value: 0,
    })
    .assertFail({ message: "Endpoint can only be called by owner" });

  const { returnData, returnCode, returnMessage } = await ctx.world.query({
    callee: ctx.contract,
    funcName: "getProtocolFee",
    funcArgs: [e.Str("EGLD")],
  });

  const fee = d.U().topDecode(returnData[0]);

  expect(fee).toBe(0n);
});

test("Set protocol fee not owner", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 10_000_000,
      funcName: "removeProtocolFee",
      funcArgs: [e.Str("EGLD"), e.U(7_50n)],
      value: 0,
    })
    .assertFail({ message: "Endpoint can only be called by owner" });
});

test("Stream created with protocol fee", async (ctx) => {
  await ctx.deployer.callContract({
    callee: ctx.contract,
    gasLimit: 10_000_000,
    funcName: "setProtocolFee",
    funcArgs: [e.Str("EGLD"), e.U(10_00n)],
    value: 0,
  });

  const result = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [ctx.recipient_wallet, generateStreamSegment(9, 1, 1000)],
    value: 10,
  });

  const streamId = parseInt(result.returnData[0]);

  const stream = await getStream(ctx, streamId);

  expect(stream.deposit).toBe(9n);
});
