use crate::{
    errors::{
        ERR_INVALID_NFT_TOKEN, ERR_INVALID_ROLE, ERR_SEND_ONE_STREAM_NFT, ERR_TOKEN_ALREADY_ISSUED,
        ERR_TOKEN_NOT_ISSUED,
    },
    storage::{Stream, StreamAttributes, StreamRole},
};

multiversx_sc::imports!();

const TOKEN_NAME: &[u8] = b"CoindripStreams";
const TOKEN_TICKER: &[u8] = b"DRIP";

const NFT_ROYALTIES: u64 = 3_00;

#[multiversx_sc::module]
pub trait StreamNftModule:
    crate::storage::StorageModule
    + crate::events::EventsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
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
        let mut full_uri = self.stream_nft_base_uri().get();
        full_uri.append_bytes(b"/api/stream/");
        full_uri.append(&stream_id_buffer);
        full_uri.append_bytes(b"/nft");

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

    /**
     * This endpoint checks if a valid stream NFTs is send or if caller is the stream sender
     */
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

    // TODO: Give credit for this function to Martin Wagner | CIO | Knights of Cathena
    fn u64_to_ascii(&self, number: u64) -> ManagedBuffer {
        let mut reversed_digits = ManagedVec::<Self::Api, u8>::new();
        let mut result = number.clone();

        while result > 0 {
            let digit = result % 10;
            result /= 10;

            let digit_char = match digit {
                0 => b'0',
                1 => b'1',
                2 => b'2',
                3 => b'3',
                4 => b'4',
                5 => b'5',
                6 => b'6',
                7 => b'7',
                8 => b'8',
                9 => b'9',
                _ => sc_panic!("invalid digit"),
            };

            reversed_digits.push(digit_char);
        }

        if &reversed_digits.len() == &0 {
            return ManagedBuffer::new_from_bytes(b"0");
        }

        let mut o = ManagedBuffer::new();

        for digit in reversed_digits.iter().rev() {
            o.append_bytes(&[digit]);
        }

        o
    }
}
