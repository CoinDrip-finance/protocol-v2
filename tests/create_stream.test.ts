import { expect, test } from 'vitest';
import { d, e } from 'xsuite';

import { ERR_END_TIME, ERR_START_TIME, ERR_STREAM_TO_CALLER, ERR_STREAM_TO_SC, ERR_ZERO_DEPOSIT } from './errors';
import { generateStreamNftAttr, getStream, requireValidStreamNft } from './utils';

test("Create valid stream", async (ctx) => {
  const { returnData } = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 130_000_000,
    funcName: "createStreamDuration",
    funcArgs: [ctx.recipient_wallet, e.U64(632), e.U64(12), e.Bool(false)],
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
        exponent: {
          numerator: 1n,
          denominator: 1n,
        },
        duration: 632n,
      },
    ],
    balances_after_cancel: null,
  });
});

test("Stream with 0 payments", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 130_000_000,
      funcName: "createStreamDuration",
      funcArgs: [ctx.recipient_wallet, e.U64(632), e.U64(12), e.Bool(false)],
      value: 0,
    })
    .assertFail({ message: ERR_ZERO_DEPOSIT });
});

test("Stream towards SC", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 130_000_000,
      funcName: "createStreamDuration",
      funcArgs: [ctx.contract, e.U64(632), e.U64(12), e.Bool(false)],
      value: 1,
    })
    .assertFail({ message: ERR_STREAM_TO_SC });
});

test("Stream towards self", async (ctx) => {
  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 130_000_000,
      funcName: "createStreamDuration",
      funcArgs: [ctx.sender_wallet, e.U64(632), e.U64(12), e.Bool(false)],
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
      gasLimit: 130_000_000,
      funcName: "createStream",
      funcArgs: [ctx.recipient_wallet, e.U64(50), e.U64(200)],
      value: 1,
    })
    .assertFail({ message: ERR_START_TIME });
});

test("End time before start time", async (ctx) => {
  await ctx.world.setCurrentBlockInfo({
    timestamp: 100,
  });

  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 130_000_000,
      funcName: "createStream",
      funcArgs: [ctx.recipient_wallet, e.U64(200), e.U64(150)],
      value: 1,
    })
    .assertFail({ message: ERR_END_TIME });
});
