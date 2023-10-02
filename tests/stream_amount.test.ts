import { expect, test } from 'vitest';

import { createStream, getRecipientBalance } from './utils';

test("Zero stream", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  const recipientBalance = await getRecipientBalance(ctx, streamId);

  expect(recipientBalance).toBe(0n);
});

test("Recipient balance at half", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  const recipientBalance = await getRecipientBalance(ctx, streamId);

  expect(recipientBalance).toBe(5n);
});

test("Recipient balance after finish", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 650,
  });

  const recipientBalance = await getRecipientBalance(ctx, streamId);

  expect(recipientBalance).toBe(10n);
});
