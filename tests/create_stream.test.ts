import { expect, test } from "vitest";
import { d, e } from "xsuite";

import { ERR_START_TIME, ERR_STREAM_TO_CALLER, ERR_STREAM_TO_SC, ERR_ZERO_DEPOSIT } from "./errors";
import { generateStreamNftAttr, generateStreamSegment, getStream, requireValidStreamNft } from "./utils";

test("Create valid stream with ESDT", async (ctx) => {
  const { returnData } = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [ctx.recipient_wallet, generateStreamSegment(3000, 1, 632), e.U64(12), e.Bool(false)],
    value: 0,
    esdts: [
      {
        id: ctx.payment_esdt_token_identifier,
        nonce: 0,
        amount: 3000,
      },
    ],
  });

  const streamId = parseInt(d.U64().topDecode(returnData[0]).toString());
  expect(streamId).toBe(1);

  const stream = await getStream(ctx, streamId);

  // Check if recipient got the Stream NFT in their wallet
  const streamNftAttr = generateStreamNftAttr(stream);
  await requireValidStreamNft(ctx, 1, 1, streamNftAttr);

  expect(stream).toEqual({
    sender: "erd1qqqqqqsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq6jjrhq",
    nft_nonce: 1n,
    payment_token: "TEST-93cadd",
    payment_nonce: 0n,
    deposit: 3000n,
    claimed_amount: 0n,
    can_cancel: false,
    start_time: 0n,
    end_time: 632n,
    cliff: 12n,
    segments: [
      {
        amount: 3000n,
        exponent: 1n,
        duration: 632n,
      },
    ],
    balances_after_cancel: null,
  });
});

test("Create valid stream with EGLD", async (ctx) => {
  const { returnData } = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [ctx.recipient_wallet, generateStreamSegment(3, 1, 632), e.U64(12), e.Bool(false)],
    value: 3,
  });

  const streamId = parseInt(d.U64().topDecode(returnData[0]).toString());
  expect(streamId).toBe(1);

  const stream = await getStream(ctx, streamId);

  // Check if recipient got the Stream NFT in their wallet
  const streamNftAttr = generateStreamNftAttr(stream);
  await requireValidStreamNft(ctx, 1, 1, streamNftAttr);

  expect(stream).toEqual({
    sender: "erd1qqqqqpgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2p87gj",
    nft_nonce: 1n,
    payment_token: "EGLD",
    payment_nonce: 0n,
    deposit: 3n,
    claimed_amount: 0n,
    can_cancel: false,
    start_time: 0n,
    end_time: 632n,
    cliff: 12n,
    segments: [
      {
        amount: 3n,
        exponent: 1n,
        duration: 632n,
      },
    ],
    balances_after_cancel: null,
  });
});

test("Create valid stream with start & end time", async (ctx) => {
  const { returnData } = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStream",
    funcArgs: [ctx.recipient_wallet, e.U64(100), generateStreamSegment(3, 1, 600), e.U64(12), e.Bool(false)],
    value: 3,
  });

  const streamId = parseInt(d.U64().topDecode(returnData[0]).toString());
  expect(streamId).toBe(1);

  const stream = await getStream(ctx, streamId);

  // Check if recipient got the Stream NFT in their wallet
  const streamNftAttr = generateStreamNftAttr(stream);
  await requireValidStreamNft(ctx, 1, 1, streamNftAttr);

  expect(stream).toEqual({
    sender: "erd1qqqqqzqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqnqu4qu",
    nft_nonce: 1n,
    payment_token: "EGLD",
    payment_nonce: 0n,
    deposit: 3n,
    claimed_amount: 0n,
    can_cancel: false,
    start_time: 100n,
    end_time: 700n,
    cliff: 12n,
    segments: [
      {
        amount: 3n,
        exponent: 1n,
        duration: 600n,
      },
    ],
    balances_after_cancel: null,
  });
});

test("Stream with 0 payments", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 200_000_000,
      funcName: "createStreamNow",
      funcArgs: [ctx.recipient_wallet, generateStreamSegment(0, 1, 632), e.U64(12), e.Bool(false)],
      value: 0,
    })
    .assertFail({ message: ERR_ZERO_DEPOSIT });
});

test("Stream towards SC", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 200_000_000,
      funcName: "createStreamNow",
      funcArgs: [ctx.contract, generateStreamSegment(1, 1, 632), e.U64(12), e.Bool(false)],
      value: 1,
    })
    .assertFail({ message: ERR_STREAM_TO_SC });
});

test("Stream towards self", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 200_000_000,
      funcName: "createStreamNow",
      funcArgs: [ctx.sender_wallet, generateStreamSegment(1, 1, 632), e.U64(12), e.Bool(false)],
      value: 1,
    })
    .assertFail({ message: ERR_STREAM_TO_CALLER });
});

test("Start time before current time", async (ctx) => {
  await ctx.world.setCurrentBlockInfo({
    timestamp: 100,
  });

  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 200_000_000,
      funcName: "createStream",
      funcArgs: [ctx.recipient_wallet, e.U64(50), generateStreamSegment(1, 1, 100)],
      value: 1,
    })
    .assertFail({ message: ERR_START_TIME });
});

// TODO: Add tests with invalid segments
