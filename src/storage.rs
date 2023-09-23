use crate::errors::ERR_INVALID_STREAM;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum Status {
    Pending,
    InProgress,
    Canceled,
    Settled,
    Finished,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct BalancesAfterCancel<M: ManagedTypeApi> {
    pub sender_balance: BigUint<M>,
    pub recipient_balance: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Exponent {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Segment<M: ManagedTypeApi> {
    pub amount: BigUint<M>,
    pub exponent: Exponent,
    pub duration: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Stream<M: ManagedTypeApi> {
    pub id: u64,
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
pub struct StreamClaimResult<M: ManagedTypeApi> {
    pub stream_id: u64,
    pub stream_nft_nonce: u64,
    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_nonce: u64,
    pub claimed_amount: BigUint<M>,
    pub is_finalized: bool,
}

#[derive(TopEncode, TopDecode, TypeAbi, ManagedVecItem, NestedEncode, NestedDecode, Clone)]
pub struct BrokerFee<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub fee: BigUint<M>,
}

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getStreamData)]
    fn get_stream(&self, stream_id: u64) -> Stream<Self::Api> {
        let stream_mapper = self.stream_by_id(stream_id);
        require!(!stream_mapper.is_empty(), ERR_INVALID_STREAM);
        stream_mapper.get()
    }

    #[storage_mapper("streamById")]
    fn stream_by_id(&self, stream_id: u64) -> SingleValueMapper<Stream<Self::Api>>;

    #[storage_mapper("streamByNft")]
    fn stream_by_nft(&self, nft_nonce: u64) -> SingleValueMapper<u64>;

    #[view(getStreamByNft)]
    fn get_stream_by_nft(&self, nonce: u64) -> Stream<Self::Api> {
        let stream_id = self.stream_by_nft(nonce).get();
        require!(stream_id > 0, ERR_INVALID_STREAM);
        self.get_stream(stream_id)
    }

    #[view(getLastStreamId)]
    #[storage_mapper("lastStreamId")]
    fn last_stream_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("streamNftToken")]
    fn stream_nft_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    // Fees
    #[view(getProtocolFee)]
    #[storage_mapper("protocolFee")]
    fn protocol_fee(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;
}
