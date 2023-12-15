import { expect, test } from "vitest";
import { assertAccount, e } from "xsuite";

import { ERR_ZERO_CLAIM } from "./errors";
import { PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING } from "./setup";
import {
  claimFromStream,
  createStream,
  generateStreamSegment,
  getRecipientBalance,
  requireStreamInvalid,
  requireValidStreamNft,
} from "./utils";

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

test("Rounding test", async (ctx) => {
  const result = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [ctx.recipient_wallet, generateStreamSegment(2, 1, 600)],
    value: 0,
    esdts: [
      {
        id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING,
        amount: 2,
      },
    ],
  });

  const streamId = parseInt(result.returnData[0]);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 250,
  });

  await claimFromStream(ctx, streamId).assertFail({ message: ERR_ZERO_CLAIM });

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  await claimFromStream(ctx, streamId);

  assertAccount(await ctx.recipient_wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING, amount: 1 }])],
  });

  await ctx.world.setCurrentBlockInfo({
    timestamp: 601,
  });

  await claimFromStream(ctx, streamId);

  assertAccount(await ctx.recipient_wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING, amount: 2 }])],
  });

  await requireValidStreamNft(ctx, 0);

  await requireStreamInvalid(ctx, streamId);
});

test("Rounding test 2", async (ctx) => {
  const amount = 1200000000000000000n;
  const result = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [ctx.recipient_wallet, generateStreamSegment(amount, 1, 86400)],
    value: 0,
    esdts: [
      {
        id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING,
        amount,
      },
    ],
  });

  const streamId = parseInt(result.returnData[0]);

  await claimFromStream(ctx, streamId).assertFail({ message: ERR_ZERO_CLAIM });

  await ctx.world.setCurrentBlockInfo({
    timestamp: 8640,
  });

  assertAccount(await ctx.recipient_wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING, amount: 0 }])],
  });

  let recipientBalance = await getRecipientBalance(ctx, streamId);

  await claimFromStream(ctx, streamId);

  recipientBalance = await getRecipientBalance(ctx, streamId);

  assertAccount(await ctx.recipient_wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: PAYMENT_ESDT_TOKEN_IDENTIFIER_ROUNDING, amount: amount / 10n }])],
  });

  await requireValidStreamNft(ctx);
});
