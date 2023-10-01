use coindrip::{
    cancel_stream::CancelStreamModule,
    claim::ClaimModule,
    create_stream::CreateStreamModule,
    errors::{
        ERR_CANT_CANCEL, ERR_CANT_CLAIM, ERR_CLIFF_TOO_BIG, ERR_END_TIME, ERR_INVALID_ROLE,
        ERR_INVALID_STREAM, ERR_START_TIME, ERR_STREAM_IS_NOT_CANCELLED, ERR_STREAM_TO_CALLER,
        ERR_STREAM_TO_SC, ERR_ZERO_CLAIM, ERR_ZERO_DEPOSIT,
    },
    storage::{StorageModule, StreamAttributes},
    CoinDrip,
};
use multiversx_sc::{
    codec::{multi_types::OptionalValue, Empty},
    types::{BigUint, EgldOrEsdtTokenIdentifier, EgldOrMultiEsdtPayment, ManagedAddress},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_egld_token_id, managed_token_id, rust_biguint,
    DebugApi,
};

mod contract_setup;
use contract_setup::{setup_contract, STREAM_NFT_TOKEN_ID, TOKEN_ID};

#[test]
fn deploy_test() {
    let mut setup = setup_contract(coindrip::contract_obj);

    // simulate deploy
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.init();
            },
        )
        .assert_ok();
}

/**
 * Utility function to get current timestamp
 */
fn get_current_timestamp() -> u64 {
    return 1668518731;
}

// TODO: Protocol fee test
// TODO: Broker fee test
// TODO: Segments test (when fully implemented)
// TODO: Renounce cancel tests

// TODO: Stream status tests

#[test]
fn create_stream_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let owner_address = setup.owner_address;

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                let stream_id = sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 60,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );

                let stream_mapper = sc.stream_by_id(stream_id);
                assert_eq!(stream_mapper.is_empty(), false);
            },
        )
        .assert_ok();

    // Check if the stream NFT was minted and sent to the recipient
    b_wrapper.check_nft_balance(
        &first_user,
        STREAM_NFT_TOKEN_ID,
        1,
        &rust_biguint!(1),
        Some(&StreamAttributes::<DebugApi> {
            sender: managed_address!(&owner_address),
            payment_token: EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_ID)),
            payment_nonce: 1u64,
            deposit: managed_biguint!(3000u64),
            remaining_balance: managed_biguint!(3000u64),
            can_cancel: true,
            start_time: current_timestamp + 60,
            end_time: current_timestamp + 60 * 60,
            cliff: 0,
            is_canceled: false,
        }),
    );

    // Create an invalid stream of 0 tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(0),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 60,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error(ERR_ZERO_DEPOSIT);

    // Stream towards the SC
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(c_wrapper.address_ref()),
                    current_timestamp + 60,
                    current_timestamp + 60 * 60,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error(ERR_STREAM_TO_SC);

    // Stream towards the caller
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&owner_address),
                    current_timestamp + 60,
                    current_timestamp + 60 * 60,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error(ERR_STREAM_TO_CALLER);

    // Start time before current time
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp - 60,
                    current_timestamp + 60 * 60,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error(ERR_START_TIME);

    // End time before start time
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60 * 60,
                    current_timestamp + 60,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error(ERR_END_TIME);
}

#[test]
fn stream_cliff_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let owner_address = setup.owner_address;

    // Create stream with invalid cliff
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 60,
                    OptionalValue::Some(60 * 80),
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_user_error(ERR_CLIFF_TOO_BIG);

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 11,
                    OptionalValue::Some(60 * 3),
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 2);

    // Claim from stream during cliff period
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_user_error(ERR_ZERO_CLAIM);

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 6);

    // Claim from stream after cliff period
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(1500));
}

#[test]
fn claim_from_stream_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let owner_address = setup.owner_address;

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 3,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // Claim from stream wrong recipient
    b_wrapper
        .execute_tx(&owner_address, c_wrapper, &rust_biguint!(0), |sc| {
            sc.claim_from_stream(1);
        })
        .assert_user_error(ERR_INVALID_ROLE);

    // Amount to claim is zero
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_user_error(ERR_ZERO_CLAIM);

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 2);

    // Claim 1.5K tokens
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(1500));

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 5);

    b_wrapper.check_nft_balance(
        &first_user,
        STREAM_NFT_TOKEN_ID,
        1,
        &rust_biguint!(1),
        Some(&Empty),
    );

    // Claim rest of the 1.5K tokens
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(3000));

    b_wrapper.check_nft_balance(
        &first_user,
        STREAM_NFT_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Empty),
    );

    // Stream is deleted
    b_wrapper
        .execute_tx(&first_user, c_wrapper, &rust_biguint!(0), |sc| {
            sc.claim_from_stream(1);
        })
        .assert_user_error(ERR_INVALID_STREAM);
}

#[test]
fn cancel_stream_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let second_user = setup.second_user_address;
    let owner_address = setup.owner_address;
    let owner_balance = b_wrapper.get_esdt_balance(&owner_address, TOKEN_ID, 0);

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 3,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // Only sender and recipient can cencel stream
    b_wrapper
        .execute_tx(&second_user, c_wrapper, &rust_biguint!(0), |sc| {
            sc.cancel_stream(1, OptionalValue::None)
        })
        .assert_user_error(ERR_INVALID_ROLE);

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 2);

    // Cancel stream in the middle
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.cancel_stream(1, OptionalValue::None);
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(1500));
    b_wrapper.check_esdt_balance(
        &owner_address,
        TOKEN_ID,
        &(owner_balance - rust_biguint!(3000)),
    );
    b_wrapper.check_nft_balance(
        &first_user,
        STREAM_NFT_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Empty),
    );

    b_wrapper.set_block_timestamp(current_timestamp);

    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 3,
                    OptionalValue::None,
                    OptionalValue::Some(false),
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            2u64,
            &rust_biguint!(1),
            |sc| {
                sc.cancel_stream(2, OptionalValue::None);
            },
        )
        .assert_user_error(ERR_CANT_CANCEL);
}

#[test]
fn claim_from_stream_after_cancel_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let second_user = setup.second_user_address;
    let owner_address = setup.owner_address;
    let owner_balance = b_wrapper.get_esdt_balance(&owner_address, TOKEN_ID, 0);

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 3,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 2);

    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream_after_cancel(1);
            },
        )
        .assert_user_error(ERR_STREAM_IS_NOT_CANCELLED);

    // Cancel stream in the middle
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.cancel_stream(1, OptionalValue::Some(false));
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(0));
    b_wrapper.check_esdt_balance(
        &owner_address,
        TOKEN_ID,
        &(owner_balance.clone() - rust_biguint!(3000)),
    );

    b_wrapper.check_nft_balance(
        &first_user,
        STREAM_NFT_TOKEN_ID,
        1,
        &rust_biguint!(1),
        Some(&Empty),
    );

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 6);

    b_wrapper
        .execute_tx(&second_user, c_wrapper, &rust_biguint!(0), |sc| {
            sc.claim_from_stream_after_cancel(1);
        })
        .assert_user_error(ERR_INVALID_ROLE);

    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream_after_cancel(1);

                assert!(!sc.stream_by_id(1).is_empty());
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(1500));
    b_wrapper.check_esdt_balance(
        &owner_address,
        TOKEN_ID,
        &(owner_balance.clone() - rust_biguint!(3000)),
    );

    b_wrapper.check_nft_balance(
        &first_user,
        STREAM_NFT_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Empty),
    );

    b_wrapper
        .execute_tx(&owner_address, c_wrapper, &rust_biguint!(0), |sc| {
            sc.claim_from_stream_after_cancel(1);

            assert!(sc.stream_by_id(1).is_empty());
        })
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(1500));
    b_wrapper.check_esdt_balance(
        &owner_address,
        TOKEN_ID,
        &(owner_balance.clone() - rust_biguint!(1500)),
    );
}

#[test]
fn streamed_so_far_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let owner_address = setup.owner_address;

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(3_000),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 3,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    // Streamed before start
    b_wrapper
        .execute_query(c_wrapper, |sc| {
            let streamed_so_far = sc.recipient_balance(1);
            assert_eq!(streamed_so_far, BigUint::zero());
        })
        .assert_ok();

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 2);

    // Streamed at half of the period
    b_wrapper
        .execute_query(c_wrapper, |sc| {
            let streamed_so_far = sc.recipient_balance(1);
            assert_eq!(streamed_so_far, BigUint::from(1500u64));
        })
        .assert_ok();

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 6);

    // Streamed after end time
    b_wrapper
        .execute_query(c_wrapper, |sc| {
            let streamed_so_far = sc.recipient_balance(1);
            assert_eq!(streamed_so_far, BigUint::from(3000u64));
        })
        .assert_ok();
}

#[test]
fn claim_from_stream_rounding_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.third_user_address;
    let owner_address = setup.owner_address;

    b_wrapper
        .execute_esdt_transfer(
            &owner_address,
            c_wrapper,
            TOKEN_ID,
            0,
            &rust_biguint!(2),
            |sc| {
                let current_timestamp = get_current_timestamp();
                sc.create_stream(
                    managed_address!(&first_user),
                    current_timestamp + 60,
                    current_timestamp + 60 * 31,
                    OptionalValue::None,
                    OptionalValue::None,
                    OptionalValue::None,
                );
            },
        )
        .assert_ok();

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 5);

    // Claim 0 tokens
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_user_error(ERR_ZERO_CLAIM);

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(0));

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 26);

    // Claim 1 token
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(1));

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 31 + 60);

    // Claim 1 token
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_esdt_balance(&first_user, TOKEN_ID, &rust_biguint!(2));

    // Stream is deleted
    b_wrapper
        .execute_tx(&first_user, c_wrapper, &rust_biguint!(0), |sc| {
            sc.claim_from_stream(1);
        })
        .assert_user_error(ERR_INVALID_STREAM);
}

#[test]
fn claim_from_stream_egld_test() {
    let mut setup = setup_contract(coindrip::contract_obj);
    let b_wrapper = &mut setup.blockchain_wrapper;
    let current_timestamp = get_current_timestamp();
    b_wrapper.set_block_timestamp(current_timestamp);
    let c_wrapper = &mut setup.contract_wrapper;
    let first_user = setup.first_user_address;
    let owner_address = setup.owner_address;

    // Create a valid stream of 3K tokens
    b_wrapper
        .execute_tx(&owner_address, c_wrapper, &rust_biguint!(100), |sc| {
            let current_timestamp = get_current_timestamp();
            sc.create_stream(
                managed_address!(&first_user),
                current_timestamp + 60,
                current_timestamp + 60 * 3,
                OptionalValue::None,
                OptionalValue::None,
                OptionalValue::None,
            );
        })
        .assert_ok();

    // Amount to claim is zero
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_user_error(ERR_ZERO_CLAIM);

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 2);

    // Claim 50 EGLD
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_egld_balance(&first_user, &rust_biguint!(50));

    b_wrapper.set_block_timestamp(current_timestamp + 60 * 5);

    // Claim rest of the 50 EGLD
    b_wrapper
        .execute_esdt_transfer(
            &first_user,
            c_wrapper,
            STREAM_NFT_TOKEN_ID,
            1u64,
            &rust_biguint!(1),
            |sc| {
                sc.claim_from_stream(1);
            },
        )
        .assert_ok();

    b_wrapper.check_egld_balance(&first_user, &rust_biguint!(100));
}
