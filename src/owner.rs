use crate::errors::ERR_PROTOCOL_FEE_ZERO;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait OwnerModule: crate::storage::StorageModule {

    // Protocol Fee
    #[only_owner]
    #[endpoint(setProtocolFee)]
    fn set_protocol_fee(
        &self,
        token: EgldOrEsdtTokenIdentifier,
        value: BigUint
    ) {
        require!(value > BigUint::zero(), ERR_PROTOCOL_FEE_ZERO);
        self.protocol_fee(&token).set(value);
    }

    #[only_owner]
    #[endpoint(removeProtocolFee)]
    fn remove_protocol_fee(
        &self,
        token: EgldOrEsdtTokenIdentifier
    ) {
        self.protocol_fee(&token).clear();
    }
}
