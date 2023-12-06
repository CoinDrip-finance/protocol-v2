use crate::storage::Status;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait StatusModule: crate::storage::StorageModule {
    #[view(getStatusOf)]
    fn status_of(&self, stream_id: u64) -> Status {
        let stream_mapper = self.stream_by_id(stream_id);
        let last_stream_id = self.get_last_stream_id();

        if stream_mapper.is_empty() && stream_id <= last_stream_id {
            return Status::Finished;
        }

        let stream = stream_mapper.get();

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

        Status::Settled
    }

    fn is_warm(&self, stream_id: u64) -> bool {
        let stream_status = self.status_of(stream_id);
        stream_status == Status::Pending || stream_status == Status::InProgress
    }
}
