use crate::{
    errors::{
        ERR_BROKER_FEE_TOO_BIG, ERR_CLIFF_TOO_BIG, ERR_END_TIME, ERR_START_TIME,
        ERR_STREAM_TO_CALLER, ERR_STREAM_TO_SC, ERR_ZERO_DEPOSIT,
    },
    storage::{BrokerFee, Exponent, Segment, Stream},
};

multiversx_sc::imports!();

const MAX_FEE: u64 = 10_00;

#[multiversx_sc::module]
pub trait CreateStreamModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + crate::stream_nft::StreamNftModule
    + crate::svg::SvgModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("*")]
    #[endpoint(createStreamDuration)]
    fn create_stream_duration(
        &self,
        recipient: ManagedAddress,
        duration: u64,
        cliff_opt: OptionalValue<u64>,
        can_cancel_opt: OptionalValue<bool>,
        broker_opt: OptionalValue<BrokerFee<Self::Api>>,
    ) -> u64 {
        let start_time = self.blockchain().get_block_timestamp();
        let end_time = start_time + duration;
        self.create_stream(
            recipient,
            start_time,
            end_time,
            cliff_opt,
            can_cancel_opt,
            broker_opt,
        )
    }

    #[payable("*")]
    #[endpoint(createStream)]
    fn create_stream(
        &self,
        recipient: ManagedAddress,
        start_time: u64,
        end_time: u64,
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
        require!(end_time > start_time, ERR_END_TIME);

        let stream_id = self.last_stream_id().get() + 1;
        self.last_stream_id().set(&stream_id);

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

        // Create segment
        let mut segments = ManagedVec::new();
        let first_exponent = Exponent {
            numerator: 1u32,
            denominator: 1u32,
        };
        let first_segment = Segment {
            amount: stream_amount.clone(),
            exponent: first_exponent,
            duration: end_time - start_time,
        };
        segments.push(first_segment);

        // TODO: Validate segments

        let cliff = cliff_opt.into_option().unwrap_or_default();
        require!(start_time + cliff < end_time, ERR_CLIFF_TOO_BIG);

        // TODO: Update event with nft
        self.create_stream_event(
            stream_id,
            &caller,
            &recipient,
            &token_identifier,
            token_nonce,
            &stream_amount,
            start_time,
            end_time,
        );

        let mut stream = Stream {
            id: stream_id,
            sender: caller,
            nft_nonce: 0,
            payment_token: token_identifier,
            payment_nonce: token_nonce,
            deposit: stream_amount,
            claimed_amount: BigUint::zero(),
            can_cancel,
            start_time,
            end_time,
            cliff,
            segments,
            balances_after_cancel: None,
        };

        let stream_nft_nonce = self.mint_stream_nft(&stream);
        stream.nft_nonce = stream_nft_nonce;

        self.stream_by_id(stream_id).set(&stream);

        self.stream_by_nft(stream_nft_nonce).set(stream_id);

        self.send().direct_esdt(
            &recipient,
            self.stream_nft_token().get_token_id_ref(),
            stream_nft_nonce,
            &BigUint::from(1u64),
        );

        stream_id
    }
}