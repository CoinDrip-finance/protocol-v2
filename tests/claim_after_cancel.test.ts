import { expect, test } from 'vitest';
import { assertAccount, e } from 'xsuite';

import { ERR_STREAM_IS_NOT_CANCELLED } from './errors';
import { cancelStream, createStream, requireValidStreamNft } from './utils';

test("Stream is not canceled", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 100,
  });

  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 50_000_000,
      funcName: "claimFromStreamAfterCancel",
      funcArgs: [e.U64(streamId)],
      value: 0,
    })
    .assertFail({ message: ERR_STREAM_IS_NOT_CANCELLED });
});

test("Successful sender claim after cancel", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  const initialBalance = await ctx.sender_wallet.getAccountBalance();

  await cancelStream(ctx, streamId, true, false);

  expect(await ctx.sender_wallet.getAccountBalance()).toBe(initialBalance);

  await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "claimFromStreamAfterCancel",
    funcArgs: [e.U64(streamId)],
    value: 0,
  });

  expect(await ctx.sender_wallet.getAccountBalance()).toBe(initialBalance + 5n);
});

test("Successful recipient claim after cancel", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  const initialBalance = await ctx.recipient_wallet.getAccountBalance();

  await cancelStream(ctx, streamId, false, false);

  expect(await ctx.recipient_wallet.getAccountBalance()).toBe(initialBalance);

  await ctx.recipient_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "claimFromStreamAfterCancel",
    funcArgs: [e.U64(streamId)],
    value: 0,
    esdts: [
      {
        id: ctx.stream_nft_token_identifier,
        nonce: streamId,
        amount: 1,
      },
    ],
  });

  expect(await ctx.recipient_wallet.getAccountBalance()).toBe(initialBalance + 5n);
});

test("Successful claim after cancel and remove stream", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  await cancelStream(ctx, streamId, false, false);

  await ctx.recipient_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "claimFromStreamAfterCancel",
    funcArgs: [e.U64(streamId)],
    value: 0,
    esdts: [
      {
        id: ctx.stream_nft_token_identifier,
        nonce: streamId,
        amount: 1,
      },
    ],
  });

  await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "claimFromStreamAfterCancel",
    funcArgs: [e.U64(streamId)],
    value: 0,
  });

  assertAccount(await ctx.contract.getAccountWithKvs(), {
    hasKvs: [e.kvs.Mapper("streamById", e.U64(streamId)).Value(null)],
  });

  await requireValidStreamNft(ctx, 0);
});
