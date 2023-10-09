multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("createStream")]
    fn create_stream_event(
        &self,
        #[indexed] sender: &ManagedAddress,
        #[indexed] recipient: &ManagedAddress,
        #[indexed] stream_token_identifier: &TokenIdentifier,
        #[indexed] stream_token_nonce: u64,
        #[indexed] payment_token: &EgldOrEsdtTokenIdentifier,
        #[indexed] payment_nonce: u64,
        #[indexed] deposit: &BigUint,
        #[indexed] deposit_with_fees: &BigUint,
        #[indexed] start_time: u64,
        #[indexed] end_time: u64,
        #[indexed] can_cancel: bool,
        #[indexed] cliff: u64,
    );

    #[event("claimFromStream")]
    fn claim_from_stream_event(
        &self,
        #[indexed] stream_id: u64,
        #[indexed] amount: &BigUint,
        #[indexed] recipient: &ManagedAddress,
    );

    #[event("cancelStream")]
    fn cancel_stream_event(
        &self,
        #[indexed] stream_id: u64,
        #[indexed] canceled_by: &ManagedAddress,
        #[indexed] claimed_amount: &BigUint,
    );

    #[event("finishedStream")]
    fn finished_stream_event(&self, #[indexed] stream_id: u64);

    #[event("renounceCancelStream")]
    fn renounce_cancel_stream_event(&self, #[indexed] stream_id: u64);
}
