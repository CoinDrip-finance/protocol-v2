import { expect, test } from "vitest";
import { e } from "xsuite";

import { ERR_INVALID_ROLE, ERR_INVALID_STREAM, ERR_ZERO_CLAIM } from "./errors";
import {
  claimFromStream,
  createStream,
  generateStreamSegment,
  getStream,
  requireEgldBalance,
  requireStreamInvalid,
  requireValidStreamNft,
} from "./utils";

test("Wrong recipient", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.sender_wallet
    .callContract({
      callee: ctx.contract,
      gasLimit: 50_000_000,
      funcName: "claimFromStream",
      funcArgs: [e.U64(streamId)],
      value: 0,
    })
    .assertFail({ message: ERR_INVALID_ROLE });
});

test("Amount to claim is zero", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await claimFromStream(ctx, streamId).assertFail({ message: ERR_ZERO_CLAIM });
});

test("Successful claim", async (ctx) => {
  const streamId = await createStream(ctx, 600);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 300,
  });

  await claimFromStream(ctx, streamId);

  // Check if half of the EGLD was claimed and the NFT is back to the recipient's wallet
  await requireEgldBalance(ctx, ctx.recipient_wallet, 5);
  await requireValidStreamNft(ctx);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 650,
  });

  await claimFromStream(ctx, streamId);

  // Check if all remaining EGLD was claimed and the NFT was burned
  await requireEgldBalance(ctx, ctx.recipient_wallet, 10);
  await requireValidStreamNft(ctx, 0);
  // Check if stream was removed from storage
  await expect(getStream(ctx, streamId)).rejects.toThrowError(ERR_INVALID_STREAM);

  await requireStreamInvalid(ctx, streamId);
});

test("Successful claim multiple segments", async (ctx) => {
  const createStreamResult = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 200_000_000,
    funcName: "createStreamNow",
    funcArgs: [ctx.recipient_wallet, e.List(generateStreamSegment(1, 1, 100), generateStreamSegment(2, 1, 200))],
    value: 3,
  });

  const streamId = parseInt(createStreamResult.returnData[0]);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 200,
  });

  await claimFromStream(ctx, streamId);

  // Check if half of the EGLD was claimed and the NFT is back to the recipient's wallet
  await requireEgldBalance(ctx, ctx.recipient_wallet, 2);
  await requireValidStreamNft(ctx);

  await ctx.world.setCurrentBlockInfo({
    timestamp: 650,
  });

  await claimFromStream(ctx, streamId);

  // Check if all remaining EGLD was claimed and the NFT was burned
  await requireEgldBalance(ctx, ctx.recipient_wallet, 3);
  await requireValidStreamNft(ctx, 0);
  // Check if stream was removed from storage
  await expect(getStream(ctx, streamId)).rejects.toThrowError(ERR_INVALID_STREAM);

  await requireStreamInvalid(ctx, streamId);
});
