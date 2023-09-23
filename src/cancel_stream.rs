multiversx_sc::imports!();

use crate::{
    errors::{
        ERR_CANCEL_ONE_STREAM, ERR_CANCEL_ONLY_OWNERS, ERR_CANCEL_ONLY_SENDER, ERR_CANT_CANCEL,
        ERR_INVALID_NFT_TOKEN, ERR_INVALID_NFT_TOKEN_NONCE, ERR_ONLY_RECIPIENT_SENDER_CAN_CLAIM,
        ERR_STREAM_IS_NOT_CANCELLED, ERR_ZERO_CLAIM,
    },
    storage::BalancesAfterCancel,
};

#[multiversx_sc::module]
pub trait CancelStreamModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + crate::claim::ClaimModule
    + crate::status::StatusModule
{
    /// This endpoint can be used the by sender or recipient of a stream to cancel the stream.
    /// !!! The stream needs to be cancelable (a property that is set when the stream is created by the sender)
    #[payable("*")]
    #[endpoint(cancelStream)]
    fn cancel_stream(&self, stream_id: u64, _with_claim: OptionalValue<bool>) {
        let mut stream = self.get_stream(stream_id);

        let is_warm = self.is_warm(stream_id);
        require!(is_warm, ERR_CANT_CANCEL);

        require!(stream.can_cancel, ERR_CANT_CANCEL);

        let caller = self.blockchain().get_caller();

        let payments = self.call_value().all_esdt_transfers().clone_value();

        if payments.len() == 0 {
            require!(caller == stream.sender, ERR_CANCEL_ONLY_OWNERS);
        } else {
            require!(payments.len() == 1, ERR_CANCEL_ONE_STREAM);
            let payment = payments.get(0);
            require!(
                self.stream_nft_token().get_token_id() == payment.token_identifier,
                ERR_INVALID_NFT_TOKEN
            );
            require!(
                stream.nft_nonce == payment.token_nonce,
                ERR_INVALID_NFT_TOKEN_NONCE
            );
        }

        let sender_balance = self.sender_balance(stream_id);
        let recipient_balance = self.recipient_balance(stream_id);

        let streamed_until_cancel = recipient_balance.clone() + stream.claimed_amount.clone();

        stream.balances_after_cancel = Some(BalancesAfterCancel {
            sender_balance,
            recipient_balance,
        });

        self.stream_by_id(stream_id).set(stream);

        let with_claim: bool = (&_with_claim.into_option()).unwrap_or(true);
        if with_claim {
            self.claim_from_stream_after_cancel(stream_id);
        }

        self.cancel_stream_event(stream_id, &caller, &streamed_until_cancel);
    }

    /// After a stream was cancelled, you can call this endpoint to claim the streamed tokens as a recipient or the remaining tokens as a sender
    /// This endpoint is especially helpful when the recipient/sender is a non-payable smart contract
    /// For convenience, this endpoint is automatically called by default from the cancel_stream endpoint (is not instructed otherwise by the "_with_claim" param)
    #[endpoint(claimFromStreamAfterCancel)]
    fn claim_from_stream_after_cancel(&self, stream_id: u64) {
        let mut stream = self.get_stream(stream_id);

        require!(
            stream.balances_after_cancel.is_some(),
            ERR_STREAM_IS_NOT_CANCELLED
        );

        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers().clone_value();

        let mut is_recipient = false;
        if payments.len() == 0 {
            require!(caller == stream.sender, ERR_CANCEL_ONLY_OWNERS);
        } else {
            require!(payments.len() == 1, ERR_CANCEL_ONE_STREAM);
            let payment = payments.get(0);
            require!(
                self.stream_nft_token().get_token_id() == payment.token_identifier,
                ERR_INVALID_NFT_TOKEN
            );
            require!(
                stream.nft_nonce == payment.token_nonce,
                ERR_INVALID_NFT_TOKEN_NONCE
            );
            is_recipient = true;
        }

        let mut balances_after_cancel = stream.balances_after_cancel.unwrap();

        if is_recipient {
            require!(balances_after_cancel.recipient_balance > 0, ERR_ZERO_CLAIM);
            self.send().direct(
                &caller,
                &stream.payment_token,
                stream.payment_nonce,
                &balances_after_cancel.recipient_balance,
            );
            self.claim_from_stream_event(
                stream_id,
                &balances_after_cancel.recipient_balance,
                false,
            );
            balances_after_cancel.recipient_balance = BigUint::zero();
        }

        if caller == stream.sender {
            require!(balances_after_cancel.sender_balance > 0, ERR_ZERO_CLAIM);
            self.send().direct(
                &stream.sender,
                &stream.payment_token,
                stream.payment_nonce,
                &balances_after_cancel.sender_balance,
            );
            balances_after_cancel.sender_balance = BigUint::zero();
        }

        stream.balances_after_cancel = Some(balances_after_cancel);
        self.stream_by_id(stream_id).set(stream);
    }

    /// This endpoint can be used the by sender to make the stream non-cancelable
    #[endpoint(renounceCancelStream)]
    fn renounce_cancel_stream(&self, stream_id: u64) {
        let mut stream = self.get_stream(stream_id);

        let is_warm = self.is_warm(stream_id);
        require!(is_warm, ERR_CANT_CANCEL);

        require!(stream.can_cancel, ERR_CANT_CANCEL);

        let caller = self.blockchain().get_caller();
        require!(caller == stream.sender, ERR_CANCEL_ONLY_SENDER);

        stream.can_cancel = false;

        self.stream_by_id(stream_id).set(stream);

        self.renounce_cancel_stream_event(stream_id);
    }
}
