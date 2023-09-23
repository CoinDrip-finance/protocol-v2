use crate::storage::Status;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StatusModule: crate::storage::StorageModule {
    #[view(getStatusOf)]
    fn status_of(&self, stream_id: u64) -> Status {
        let stream = self.get_stream(stream_id);

        if stream.balances_after_cancel.is_some() {
            return Status::Canceled;
        }

        let current_time = self.blockchain().get_block_timestamp();
        if current_time < stream.start_time {
            return Status::Pending;
        }

        if current_time < stream.end_time {
            return Status::InProgress;
        }

        if stream.deposit > stream.claimed_amount {
            return Status::Settled;
        }

        Status::Finished
    }

    fn is_warm(&self, stream_id: u64) -> bool {
        let stream_status = self.status_of(stream_id);
        stream_status == Status::Pending || stream_status == Status::InProgress
    }


    fn is_cold(&self, stream_id: u64) -> bool {
        let stream_status = self.status_of(stream_id);
        stream_status == Status::Canceled || stream_status == Status::Settled || stream_status == Status::Finished
    }
}
