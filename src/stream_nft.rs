use crate::{
    errors::{
        ERR_INVALID_NFT_TOKEN, ERR_INVALID_ROLE, ERR_SEND_ONE_STREAM_NFT, ERR_TOKEN_ALREADY_ISSUED,
        ERR_TOKEN_NOT_ISSUED,
    },
    storage::{Stream, StreamAttributes, StreamRole},
};

multiversx_sc::imports!();

const TOKEN_NAME: &[u8] = b"CoinDripStreams";
const TOKEN_TICKER: &[u8] = b"DRIP";

const NFT_ROYALTIES: u64 = 2_00;

#[multiversx_sc::module]
pub trait StreamNftModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + crate::svg::SvgModule
{
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(&self) {
        require!(self.stream_nft_token().is_empty(), ERR_TOKEN_ALREADY_ISSUED);

        let issue_cost = self.call_value().egld_value().clone_value();

        let token_name = ManagedBuffer::new_from_bytes(TOKEN_NAME);
        let token_ticker = ManagedBuffer::new_from_bytes(TOKEN_TICKER);

        self.stream_nft_token().issue_and_set_all_roles(
            EsdtTokenType::NonFungible,
            issue_cost,
            token_name,
            token_ticker,
            18,
            None,
        );
    }

    fn mint_stream_nft(&self, stream: &Stream<Self::Api>) -> u64 {
        require!(!self.stream_nft_token().is_empty(), ERR_TOKEN_NOT_ISSUED);

        let big_one = BigUint::from(1u64);

        let mut token_name = ManagedBuffer::new_from_bytes(b"CoinDrip Stream #");
        let stream_id_buffer = self.u64_to_ascii(stream.nft_nonce);
        token_name.append(&stream_id_buffer);

        let mut uris = ManagedVec::new();
        let mut full_uri = ManagedBuffer::new_from_bytes(b"data:image/svg+xml;base64,");

        let svg_image = self.generate_svg_from_stream(stream);
        full_uri.append(&svg_image);

        uris.push(full_uri);

        let royalties = BigUint::from(NFT_ROYALTIES);

        let attributes = StreamAttributes {
            sender: stream.sender.clone(),
            payment_token: stream.payment_token.clone(),
            payment_nonce: stream.payment_nonce,
            deposit: stream.deposit.clone(),
            remaining_balance: stream.deposit.clone(),
            can_cancel: stream.can_cancel,
            start_time: stream.start_time,
            end_time: stream.end_time,
            cliff: stream.cliff,
            is_canceled: false,
        };
        let mut serialized_attributes = ManagedBuffer::new();
        if let core::result::Result::Err(err) = attributes.top_encode(&mut serialized_attributes) {
            sc_panic!("Attributes encode error: {}", err.message_bytes());
        }

        let attributes_sha256 = self.crypto().sha256(&serialized_attributes);
        let attributes_hash = attributes_sha256.as_managed_buffer();

        let nonce = self.send().esdt_nft_create(
            self.stream_nft_token().get_token_id_ref(),
            &big_one,
            &token_name,
            &royalties,
            &attributes_hash,
            &attributes,
            &uris,
        );

        nonce
    }

    // This endpoint checks if a valid stream NFTs is send or if caller is the stream sender
    fn require_valid_stream_nft(
        &self,
        stream_id: u64,
        required_role_opt: OptionalValue<StreamRole>,
    ) -> (StreamRole, Stream<Self::Api>) {
        let mut stream_role = StreamRole::Sender;

        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers().clone_value();
        let stream = self.get_stream(stream_id);

        if payments.len() == 0 {
            require!(caller == stream.sender, ERR_INVALID_ROLE);
        } else {
            require!(payments.len() == 1, ERR_SEND_ONE_STREAM_NFT);
            let payment = payments.get(0);
            require!(
                self.stream_nft_token().get_token_id() == payment.token_identifier,
                ERR_INVALID_NFT_TOKEN
            );
            require!(stream.nft_nonce == payment.token_nonce, ERR_INVALID_ROLE);
            stream_role = StreamRole::Recipient;
        }

        if required_role_opt.is_some() {
            let required_role = required_role_opt.into_option().unwrap();
            require!(required_role == stream_role, ERR_INVALID_ROLE);
        }

        (stream_role, stream)
    }

    fn burn_stream_nft(&self, stream_id: u64) {
        let stream = self.get_stream(stream_id);
        let nft_nonce = stream.nft_nonce;

        self.stream_nft_token()
            .nft_burn(nft_nonce, &BigUint::from(1u32));
    }
}
