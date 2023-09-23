#![no_std]

multiversx_sc::imports!();

pub mod cancel_stream;
pub mod claim;
pub mod create_stream;
pub mod errors;
mod events;
mod owner;
mod status;
pub mod storage;
mod stream_nft;

#[multiversx_sc::contract]
pub trait CoinDrip:
    storage::StorageModule
    + events::EventsModule
    + create_stream::CreateStreamModule
    + claim::ClaimModule
    + cancel_stream::CancelStreamModule
    + owner::OwnerModule
    + status::StatusModule
    + stream_nft::StreamNftModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self) {}
}
