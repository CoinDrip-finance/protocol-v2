multiversx_sc::imports!();

const MAX_CLAIMS_PER_TX: usize = 250;

use crate::{
    errors::{
        ERR_CANT_CLAIM, ERR_CLAIM_FROM_TOO_MANY_STREAMS, ERR_INVALID_NFT_TOKEN,
        ERR_ONLY_RECIPIENT_CLAIM, ERR_ZERO_CLAIM,
    },
    storage::{Segment, Status, StreamClaimResult},
};

#[multiversx_sc::module]
pub trait ClaimModule:
    crate::storage::StorageModule + crate::events::EventsModule + crate::status::StatusModule
{
    fn compute_segment_value(
        &self,
        segment_start_time: u64,
        segment: Segment<Self::Api>,
    ) -> BigUint {
        let segment_end_time = segment_start_time + segment.duration;
        let current_time = self.blockchain().get_block_timestamp();

        if current_time < segment_start_time {
            return BigUint::zero();
        }

        if current_time > segment_end_time {
            return segment.amount;
        }

        // TODO: Implement nth_root in BigUint to allow fractional exponent
        let numerator = BigUint::from(current_time - segment_start_time)
            .pow(segment.exponent.numerator / segment.exponent.denominator)
            .mul(segment.amount);
        let denominator = BigUint::from(segment.duration)
            .pow(segment.exponent.numerator / segment.exponent.denominator);

        numerator.div(denominator)
    }

    ///
    /// Calculates the recipient balance based on the amount stream so far and the already claimed amount
    /// |xxxx|*******|--|
    /// S            C  E
    /// S = start time
    /// xxxx = already claimed amount
    /// C = current time
    /// E = end time
    /// The zone marked with "****..." represents the recipient balance
    #[view(recipientBalance)]
    fn recipient_balance(&self, stream_id: u64) -> BigUint {
        let stream = self.get_stream(stream_id);
        let current_time = self.blockchain().get_block_timestamp();

        if stream.start_time + stream.cliff > current_time {
            return BigUint::zero();
        }

        if current_time < stream.start_time {
            return BigUint::zero();
        }

        if current_time > stream.end_time {
            return stream.deposit.sub(stream.claimed_amount);
        }

        let mut last_segment_end_time = stream.start_time;
        let mut recipient_balance = BigUint::zero();
        for segment in &stream.segments {
            let segment_amount = self.compute_segment_value(last_segment_end_time, segment.clone());

            if segment_amount == 0 {
                break;
            }

            recipient_balance = recipient_balance.add(segment_amount);
            last_segment_end_time += segment.duration;
        }

        recipient_balance.min(stream.deposit)
    }

    /// Calculates the sender balance based on the recipient balance and the claimed balance
    /// |----|-------|**|
    /// S   L.C      C  E
    /// S = start time
    /// L.C = last claimed amount
    /// C = current time
    /// E = end time
    /// The zone marked with "**" represents the sender balance
    #[view(senderBalance)]
    fn sender_balance(&self, stream_id: u64) -> BigUint {
        let stream = self.get_stream(stream_id);

        stream.deposit - self.recipient_balance(stream_id) - stream.claimed_amount
    }

    /// This endpoint can be used by the recipient of the stream to claim the stream amount of tokens
    #[payable("*")]
    #[endpoint(claimFromStream)]
    fn claim_from_stream(&self) -> StreamClaimResult<Self::Api> {
        let stream_nft = self.call_value().single_esdt();
        require!(
            stream_nft.token_identifier == self.stream_nft_token().get_token_id(),
            ERR_INVALID_NFT_TOKEN
        );
        let mut stream = self.get_stream_by_nft(stream_nft.token_nonce);
        let stream_id = stream.id;

        let current_status = self.status_of(stream_id);
        let is_warm = self.is_warm(stream_id);
        require!(is_warm || current_status == Status::Settled, ERR_CANT_CLAIM);

        let caller = self.blockchain().get_caller();

        let amount = self.recipient_balance(stream_id);

        require!(amount > 0, ERR_ZERO_CLAIM);

        stream.claimed_amount += &amount;
        self.stream_by_id(stream_id).set(&stream);

        self.send().direct(
            &caller,
            &stream.payment_token,
            stream.payment_nonce,
            &amount,
        );

        let new_status = self.status_of(stream_id);
        let is_finalized = new_status == Status::Finished;

        self.claim_from_stream_event(stream_id, &amount, is_finalized);

        StreamClaimResult {
            stream_id,
            stream_nft_nonce: stream_nft.token_nonce,
            payment_token: stream.payment_token,
            payment_nonce: stream.payment_nonce,
            claimed_amount: amount,
            is_finalized,
        }
    }
}
