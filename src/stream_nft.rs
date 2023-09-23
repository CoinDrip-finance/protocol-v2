use crate::{
    errors::{ERR_TOKEN_ALREADY_ISSUED, ERR_TOKEN_NOT_ISSUED},
    storage::Stream,
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
        let empty_buffer = ManagedBuffer::new();

        let mut token_name = ManagedBuffer::new_from_bytes(b"CoinDrip Stream #");
        let stream_id_buffer = self.u64_to_ascii(stream.id);
        token_name.append(&stream_id_buffer);

        let mut uris = ManagedVec::new();
        let mut full_uri = ManagedBuffer::new_from_bytes(b"data:image/svg+xml;base64,");

        let svg_image = self.generate_svg_from_stream(stream);
        full_uri.append(&svg_image);

        uris.push(full_uri);

        let royalties = BigUint::from(NFT_ROYALTIES);

        let nonce = self.send().esdt_nft_create(
            self.stream_nft_token().get_token_id_ref(),
            &big_one,
            &token_name,
            &royalties,
            &empty_buffer,
            &empty_buffer,
            &uris,
        );

        nonce
    }
}
