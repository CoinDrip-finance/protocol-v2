import { TestContext } from 'vitest';
import { assertAccount, d, e, SWallet } from 'xsuite';
import { TupleEncodable } from 'xsuite/dist/data/TupleEncodable';
import { CallContractTxResult, TxResultPromise } from 'xsuite/dist/world/world';

export const createStream = async (ctx: TestContext, duration = 632, cliff = 10, canCancel = true) => {
  const result = await ctx.sender_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 150_000_000,
    funcName: "createStreamDuration",
    funcArgs: [ctx.recipient_wallet, e.U64(duration), e.U64(cliff), e.Bool(canCancel)],
    value: 10,
  });

  return parseInt(result.returnData[0]);
};

export const getStream = async (ctx: TestContext, streamId: number) => {
  const { returnData: returnDataStream } = await ctx.world.query({
    callee: ctx.contract,
    funcName: "getStreamData",
    funcArgs: [e.U64(streamId)],
  });

  return streamDecoder.topDecode(returnDataStream[0]);
};

export const claimFromStream = (
  ctx: TestContext,
  streamId: number,
  streamNftNonce = 1
): TxResultPromise<CallContractTxResult> => {
  return ctx.recipient_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "claimFromStream",
    funcArgs: [e.U64(streamId)],
    value: 0,
    esdts: [
      {
        id: ctx.stream_nft_token_identifier,
        nonce: streamNftNonce,
        amount: 1,
      },
    ],
  });
};

export const requireValidStreamNft = async (ctx: TestContext, amount = 1, nonce = 1, attrs?: TupleEncodable) => {
  assertAccount(await ctx.recipient_wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: ctx.stream_nft_token_identifier, nonce, amount }])],
  });

  if (attrs) {
    assertAccount(await ctx.world.sysAcc.getAccountWithKvs(), {
      hasKvs: [e.kvs.Esdts([{ id: ctx.stream_nft_token_identifier, nonce, attrs }])],
    });
  }
};

export const requireEsdtBalance = async (ctx: TestContext, wallet: SWallet, amount: number) => {
  assertAccount(await wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: ctx.payment_esdt_token_identifier, nonce: 0, amount }])],
  });
};

const exponentDecoder = d.Tuple({
  numerator: d.U32(),
  denominator: d.U32(),
});

const segmentDecoder = d.Tuple({
  amount: d.U(),
  exponent: exponentDecoder,
  duration: d.U64(),
});

const balancesAfterCancelDecoder = d.Tuple({
  sender_balance: d.U(),
  recipient_balance: d.U(),
});

export const streamDecoder = d.Tuple({
  sender: d.Addr(),
  nft_nonce: d.U64(),
  payment_token: d.Str(),
  payment_nonce: d.U64(),
  deposit: d.U(),
  claimed_amount: d.U(),
  can_cancel: d.Bool(),
  start_time: d.U64(),
  end_time: d.U64(),
  cliff: d.U64(),
  segments: d.List(segmentDecoder),
  balances_after_cancel: d.Option(balancesAfterCancelDecoder),
});

export const generateStreamNftAttr = (stream: any) => {
  return e.Tuple(
    e.Addr(stream.sender),
    e.Str(stream.payment_token),
    e.U64(stream.payment_nonce),
    e.U(stream.deposit),
    e.U(stream.remaining_balance),
    e.Bool(stream.can_cancel),
    e.U64(stream.cliff),
    e.Bool(false)
  );
};
