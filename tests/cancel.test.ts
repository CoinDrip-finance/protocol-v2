import { expect, test } from 'vitest';
import { e } from 'xsuite';

import { ERR_CANT_CANCEL, ERR_INVALID_ROLE } from './errors';
import { createStream, getStream, requireEgldBalance } from './utils';

test("Wrong recipient", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.deployer
    .callContract({
      callee: ctx.contract,
      gasLimit: 50_000_000,
      funcName: "cancelStream",
      funcArgs: [e.U64(streamId)],
      value: 0,
    })
    .assertFail({ message: ERR_INVALID_ROLE });
});

test("Non-Cancelable stream", async (ctx) => {
  const streamId = await createStream(ctx, 600, 0, false);

  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 50_000_000,
      funcName: "cancelStream",
      funcArgs: [e.U64(streamId)],
      value: 0,
    })
    .assertFail({ message: ERR_CANT_CANCEL });
});

test("Successfully cancel stream as sender", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  const initialBalance = Number(await ctx.sender_wallet.getAccountBalance());

  await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "cancelStream",
    funcArgs: [e.U64(streamId)],
    value: 0,
  });

  await requireEgldBalance(ctx, ctx.sender_wallet, initialBalance + 5);

  // Check that the remaining balance is correct
  const stream = await getStream(ctx, streamId);
  expect(stream.balances_after_cancel?.sender_balance).toBe(0n);
  expect(stream.balances_after_cancel?.recipient_balance).toBe(5n);
});

test("Successfully cancel stream as recipient", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  const initialBalance = Number(await ctx.recipient_wallet.getAccountBalance());

  await ctx.recipient_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "cancelStream",
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

  await requireEgldBalance(ctx, ctx.recipient_wallet, initialBalance + 5);

  // Check that the remaining balance is correct
  const stream = await getStream(ctx, streamId);
  expect(stream.balances_after_cancel?.sender_balance).toBe(5n);
  expect(stream.balances_after_cancel?.recipient_balance).toBe(0n);
});
