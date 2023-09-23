use crate::errors::ERR_TOKEN_ALREADY_ISSUED;

multiversx_sc::imports!();

const TOKEN_NAME: &[u8] = b"CoinDripStreams";
const TOKEN_TICKER: &[u8] = b"DRIP";

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
}
