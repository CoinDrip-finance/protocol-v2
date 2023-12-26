use crate::{
    errors::{
        ERR_BROKER_FEE_TOO_BIG, ERR_CLIFF_TOO_BIG, ERR_END_TIME, ERR_INVALID_SEGMENTS_DEPOSIT,
        ERR_INVALID_SEGMENTS_DURATION, ERR_START_TIME, ERR_STREAM_TO_CALLER, ERR_STREAM_TO_SC,
        ERR_TOO_MANY_SEGMENTS, ERR_ZERO_DEPOSIT,
    },
    storage::{BrokerFee, Segment, Stream},
};

multiversx_sc::imports!();

const MAX_FEE: u64 = 10_00;
const MAX_SEGMENTS: usize = 25;

#[multiversx_sc::module]
pub trait CreateStreamModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + crate::stream_nft::StreamNftModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[allow_multiple_var_args]
    #[payable("*")]
    #[endpoint(createStreamNow)]
    fn create_stream_now(
        &self,
        recipient: ManagedAddress,
        segments: ManagedVec<Segment<Self::Api>>,
        cliff_opt: OptionalValue<u64>,
        can_cancel_opt: OptionalValue<bool>,
        broker_opt: OptionalValue<BrokerFee<Self::Api>>,
    ) -> u64 {
        let start_time = self.blockchain().get_block_timestamp();

        self.create_stream(
            recipient,
            start_time,
            segments,
            cliff_opt,
            can_cancel_opt,
            broker_opt,
        )
    }

    #[allow_multiple_var_args]
    #[payable("*")]
    #[endpoint(createStream)]
    fn create_stream(
        &self,
        recipient: ManagedAddress,
        start_time: u64,
        segments: ManagedVec<Segment<Self::Api>>,
        cliff_opt: OptionalValue<u64>,
        can_cancel_opt: OptionalValue<bool>,
        broker_opt: OptionalValue<BrokerFee<Self::Api>>,
    ) -> u64 {
        let caller = self.blockchain().get_caller();
        require!(
            recipient != self.blockchain().get_sc_address(),
            ERR_STREAM_TO_SC
        );
        require!(recipient != caller, ERR_STREAM_TO_CALLER);

        let (token_identifier, token_nonce, token_amount) =
            self.call_value().egld_or_single_esdt().into_tuple();

        require!(token_amount > 0, ERR_ZERO_DEPOSIT);

        let current_time = self.blockchain().get_block_timestamp();
        require!(start_time >= current_time, ERR_START_TIME);

        let stream_id = self.get_last_stream_id() + 1;

        let can_cancel: bool = (&can_cancel_opt.into_option()).unwrap_or(true);

        let mut stream_amount = token_amount.clone();
        // Check and send protocol fee
        if !self.protocol_fee(&token_identifier).is_empty() {
            let protocol_fee = self
                .protocol_fee(&token_identifier)
                .get()
                .mul(stream_amount.clone())
                .div(100_00u32);
            self.send().direct(
                &self.blockchain().get_owner_address(),
                &token_identifier,
                token_nonce,
                &protocol_fee,
            );
            stream_amount = stream_amount.sub(protocol_fee);
        }

        // Check and send broker fee
        if broker_opt.is_some() {
            let broker = broker_opt.into_option().unwrap();
            if broker.fee > BigUint::zero() {
                require!(broker.fee <= BigUint::from(MAX_FEE), ERR_BROKER_FEE_TOO_BIG);
                let broker_fee = broker.fee.mul(stream_amount.clone()).div(100_00u32);
                self.send()
                    .direct(&broker.address, &token_identifier, token_nonce, &broker_fee);
                stream_amount = stream_amount.sub(broker_fee);
            }
        }

        let stream_duration = self.validate_stream_segments(&stream_amount, &segments);
        let end_time = start_time + stream_duration;
        require!(end_time > start_time, ERR_END_TIME);

        let cliff = cliff_opt.into_option().unwrap_or_default();
        require!(start_time + cliff < end_time, ERR_CLIFF_TOO_BIG);

        let stream = Stream {
            sender: caller.clone(),
            nft_nonce: stream_id,
            payment_token: token_identifier.clone(),
            payment_nonce: token_nonce,
            deposit: stream_amount.clone(),
            claimed_amount: BigUint::zero(),
            can_cancel,
            start_time,
            end_time,
            cliff,
            segments: segments.clone(),
            balances_after_cancel: None,
        };

        let stream_nft_nonce = self.mint_stream_nft(&stream);

        self.stream_by_id(stream_id).set(&stream);

        self.send().direct_esdt(
            &recipient,
            self.stream_nft_token().get_token_id_ref(),
            stream_nft_nonce,
            &BigUint::from(1u64),
        );

        self.create_stream_event(
            &caller,
            &recipient,
            self.stream_nft_token().get_token_id_ref(),
            stream_nft_nonce,
            &token_identifier,
            token_nonce,
            &stream_amount,
            &token_amount,
            start_time,
            end_time,
            can_cancel,
            cliff,
            &segments,
        );

        stream_id
    }

    fn validate_stream_segments(
        &self,
        deposit: &BigUint,
        segments: &ManagedVec<Segment<Self::Api>>,
    ) -> u64 {
        require!(segments.len() <= MAX_SEGMENTS, ERR_TOO_MANY_SEGMENTS);

        let mut segments_duration = 0u64;
        let mut segments_total_deposit = BigUint::zero();
        for segment in segments {
            require!(segment.duration > 0, ERR_INVALID_SEGMENTS_DURATION);

            segments_duration += segment.duration;
            segments_total_deposit += segment.amount;
        }

        require!(
            &segments_total_deposit == deposit,
            ERR_INVALID_SEGMENTS_DEPOSIT
        );

        segments_duration
    }
}
