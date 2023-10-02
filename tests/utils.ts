import { expect, TestContext } from 'vitest';
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
  const {
    returnData: returnDataStream,
    returnCode,
    returnMessage,
  } = await ctx.world.query({
    callee: ctx.contract,
    funcName: "getStreamData",
    funcArgs: [e.U64(streamId)],
  });

  if (parseInt(returnCode) > 0) {
    throw Error(returnMessage);
  }

  return streamDecoder.topDecode(returnDataStream[0]);
};

export const getRecipientBalance = async (ctx: TestContext, streamId: number) => {
  const {
    returnData: returnDataStream,
    returnCode,
    returnMessage,
  } = await ctx.world.query({
    callee: ctx.contract,
    funcName: "recipientBalance",
    funcArgs: [e.U64(streamId)],
  });

  if (parseInt(returnCode) > 0) {
    throw Error(returnMessage);
  }

  return d.U().topDecode(returnDataStream[0]);
};

export const claimFromStream = (ctx: TestContext, streamId: number): TxResultPromise<CallContractTxResult> => {
  return ctx.recipient_wallet.callContract({
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "claimFromStream",
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
};

export const cancelStream = (
  ctx: TestContext,
  streamId: number,
  isSender: boolean,
  withClaim: boolean
): TxResultPromise<CallContractTxResult> => {
  const call = {
    callee: ctx.contract,
    gasLimit: 50_000_000,
    funcName: "cancelStream",
    funcArgs: [e.U64(streamId), e.Bool(withClaim)],
    value: 0,
  };
  if (!isSender) {
    call["esdts"] = [
      {
        id: ctx.stream_nft_token_identifier,
        nonce: streamId,
        amount: 1,
      },
    ];
  }
  const wallet = isSender ? ctx.sender_wallet : ctx.recipient_wallet;
  return wallet.callContract(call);
};

export const requireValidStreamNft = async (ctx: TestContext, amount = 1, nonce = 1, attrs?: TupleEncodable) => {
  assertAccount(await ctx.recipient_wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: ctx.stream_nft_token_identifier, nonce, amount }])],
  });

  if (attrs) {
    const key = e.Str(`ELRONDesdt${ctx.stream_nft_token_identifier}`).toTopHex() + e.U64(1).toTopHex();
    const value = (await ctx.world.sysAcc.getAccountKvs())[key];
    expect(value).toContain(attrs.toTopHex())
  }
};

export const requireEsdtBalance = async (ctx: TestContext, wallet: SWallet, amount: number) => {
  assertAccount(await wallet.getAccountWithKvs(), {
    hasKvs: [e.kvs.Esdts([{ id: ctx.payment_esdt_token_identifier, amount }])],
  });
};

export const requireEgldBalance = async (ctx: TestContext, wallet: SWallet, amount: number) => {
  const balance = await wallet.getAccountBalance();
  expect(balance).toBe(BigInt(amount));
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
    e.U(stream.deposit - stream.claimed_amount),
    e.Bool(stream.can_cancel),
    e.U64(stream.start_time),
    e.U64(stream.end_time),
    e.U64(stream.cliff),
    e.Bool(false)
  );
};
