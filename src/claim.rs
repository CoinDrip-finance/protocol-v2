multiversx_sc::imports!();

use crate::{
    errors::{ERR_CANT_CLAIM, ERR_ZERO_CLAIM},
    storage::{Segment, Status, StreamAttributes, StreamRole},
};

#[multiversx_sc::module]
pub trait ClaimModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + crate::status::StatusModule
    + crate::stream_nft::StreamNftModule
    + crate::svg::SvgModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    /// Compute the streamed amount from a specific stream segment
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
    /// Calculates the entire streamed amount until the current time
    /// |************|--|
    /// S            C  E
    /// S = start time
    /// C = current time
    /// E = end time
    /// The zone marked with "****..." represents the streamed amount
    #[view(streamedAmount)]
    fn streamed_amount(&self, stream_id: u64) -> BigUint {
        let stream = self.get_stream(stream_id);
        let current_time = self.blockchain().get_block_timestamp();

        if current_time < stream.start_time {
            return BigUint::zero();
        }

        if current_time > stream.end_time {
            return stream.deposit;
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

        let streamed_amount = self.streamed_amount(stream_id);
        streamed_amount - stream.claimed_amount
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

    fn is_stream_finalized(&self, stream_id: u64) -> bool {
        let stream = self.get_stream(stream_id);
        let current_time = self.blockchain().get_block_timestamp();
        let is_finalized = current_time >= stream.end_time;
        return is_finalized;
    }

    /// This endpoint can be used by the recipient of the stream to claim the stream amount of tokens
    #[payable("*")]
    #[endpoint(claimFromStream)]
    fn claim_from_stream(&self, stream_id: u64) {
        // Validate the NFT and retrieve the associated stream
        let (_, mut stream) =
            self.require_valid_stream_nft(stream_id, OptionalValue::Some(StreamRole::Recipient));

        // Check the stream status
        let current_status = self.status_of(stream_id);
        let is_warm = self.is_warm(stream_id);
        require!(is_warm || current_status == Status::Settled, ERR_CANT_CLAIM);

        // Get and validate the claimable amount
        let amount = self.recipient_balance(stream_id);
        require!(amount > 0, ERR_ZERO_CLAIM);

        let is_finalized = self.is_stream_finalized(stream_id);
        let caller = self.blockchain().get_caller();

        if is_finalized {
            self.remove_stream(stream_id, true);
        } else {
            stream.claimed_amount += &amount;
            self.stream_by_id(stream_id).set(&stream);

            let mut nft_attributes: StreamAttributes<Self::Api> = self
                .stream_nft_token()
                .get_token_attributes(stream.nft_nonce);
            nft_attributes.remaining_balance -= &amount;
            self.stream_nft_token()
                .nft_update_attributes(stream.nft_nonce, &nft_attributes);

            self.send().direct_esdt(
                &caller,
                self.stream_nft_token().get_token_id_ref(),
                stream.nft_nonce,
                &BigUint::from(1u32),
            );
        }

        // Send claimed tokens
        self.send().direct(
            &caller,
            &stream.payment_token,
            stream.payment_nonce,
            &amount,
        );

        self.claim_from_stream_event(stream_id, &amount, &caller);

        // TODO: Check to see what props to return here
    }

    fn remove_stream(&self, stream_id: u64, with_burn: bool) {
        if with_burn {
            self.burn_stream_nft(stream_id);
        }

        self.stream_by_id(stream_id).clear();

        self.finished_stream_event(stream_id);
    }
}
