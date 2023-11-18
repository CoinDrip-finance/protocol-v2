use crate::errors::ERR_INVALID_STREAM;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum StreamRole {
    Sender,
    Recipient,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum Status {
    Pending,
    InProgress,
    Canceled,
    Settled,
    Finished,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct BalancesAfterCancel<M: ManagedTypeApi> {
    pub sender_balance: BigUint<M>,
    pub recipient_balance: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Segment<M: ManagedTypeApi> {
    pub amount: BigUint<M>,
    pub exponent: u32,
    pub duration: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi, Clone)]
pub struct Stream<M: ManagedTypeApi> {
    pub sender: ManagedAddress<M>,
    pub nft_nonce: u64,
    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub deposit: BigUint<M>,
    pub claimed_amount: BigUint<M>,
    pub can_cancel: bool,
    pub start_time: u64,
    pub end_time: u64,
    pub cliff: u64,
    pub segments: ManagedVec<M, Segment<M>>,
    pub balances_after_cancel: Option<BalancesAfterCancel<M>>,
}

#[derive(TopEncode, TopDecode, TypeAbi, ManagedVecItem, NestedEncode, NestedDecode, Clone)]
pub struct BrokerFee<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub fee: BigUint<M>,
}

#[derive(TopEncode, TopDecode, TypeAbi, Clone, PartialEq, Debug)]
pub struct StreamAttributes<M: ManagedTypeApi> {
    pub sender: ManagedAddress<M>,
    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub deposit: BigUint<M>,
    pub remaining_balance: BigUint<M>,
    pub can_cancel: bool,
    pub start_time: u64,
    pub end_time: u64,
    pub cliff: u64,
    pub is_canceled: bool,
}

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getStreamData)]
    fn get_stream(&self, stream_id: u64) -> Stream<Self::Api> {
        let stream_mapper = self.stream_by_id(stream_id);
        require!(!stream_mapper.is_empty(), ERR_INVALID_STREAM);
        stream_mapper.get()
    }

    fn get_last_stream_id(&self) -> u64 {
        self.blockchain().get_current_esdt_nft_nonce(
            &self.blockchain().get_sc_address(),
            self.stream_nft_token().get_token_id_ref(),
        )
    }

    #[storage_mapper("streamById")]
    fn stream_by_id(&self, stream_id: u64) -> SingleValueMapper<Stream<Self::Api>>;

    #[storage_mapper("streamNftToken")]
    fn stream_nft_token(&self) -> NonFungibleTokenMapper<Self::Api>;
    #[storage_mapper("streamNftBaseUri")]
    fn stream_nft_base_uri(&self) -> SingleValueMapper<ManagedBuffer>;

    // Fees
    #[view(getProtocolFee)]
    #[storage_mapper("protocolFee")]
    fn protocol_fee(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;
}
