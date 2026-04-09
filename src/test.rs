#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

use super::*;

// ── helpers ────────────────────────────────────────────────────────────────

/// Creates a Stellar Asset Contract (SAC) and returns its address.
fn create_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

struct TestSetup {
    env: Env,
    contract_id: Address,
    client_addr: Address,
    freelancer_addr: Address,
    token_addr: Address,
    amount: i128,
}

fn setup() -> TestSetup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FreelanceEscrow, ());

    let token_admin = Address::generate(&env);
    let token_addr = create_token(&env, &token_admin);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let amount: i128 = 5_000_000; // 0.5 XLM in stroops

    // Fund the client wallet with 10 XLM worth of tokens
    StellarAssetClient::new(&env, &token_addr).mint(&client_addr, &10_000_000);

    TestSetup {
        env,
        contract_id,
        client_addr,
        freelancer_addr,
        token_addr,
        amount,
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

/// Happy path: deposit then release transfers funds to freelancer.
#[test]
fn test_deposit_and_release_success() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let token = TokenClient::new(&s.env, &s.token_addr);

    // Deposit
    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.token_addr,
        &s.amount,
    );

    // Funds moved out of client wallet into contract
    assert_eq!(token.balance(&s.client_addr), 5_000_000);
    assert_eq!(token.balance(&s.contract_id), 5_000_000);

    // Verify on-chain state
    let escrow = contract.get_escrow();
    assert_eq!(escrow.client, s.client_addr);
    assert_eq!(escrow.freelancer, s.freelancer_addr);
    assert_eq!(escrow.amount, s.amount);

    // Release to freelancer
    contract.release(&s.client_addr);

    // Freelancer received funds; contract is drained
    assert_eq!(token.balance(&s.freelancer_addr), 5_000_000);
    assert_eq!(token.balance(&s.contract_id), 0);
}

/// Only the original client may release — attacker is rejected.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_release_by_non_client_is_rejected() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let attacker = Address::generate(&s.env);

    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.token_addr,
        &s.amount,
    );

    contract.release(&attacker); // must panic
}

/// Cancel refunds the full amount back to the client.
#[test]
fn test_cancel_refunds_client() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let token = TokenClient::new(&s.env, &s.token_addr);

    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.token_addr,
        &s.amount,
    );

    assert_eq!(token.balance(&s.client_addr), 5_000_000); // half gone

    contract.cancel(&s.client_addr);

    // Full refund
    assert_eq!(token.balance(&s.client_addr), 10_000_000);
    assert_eq!(token.balance(&s.contract_id), 0);
}

/// Double-deposit is rejected.
#[test]
#[should_panic(expected = "Escrow already exists")]
fn test_double_deposit_is_rejected() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);

    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.token_addr,
        &s.amount,
    );

    // Second deposit must fail
    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.token_addr,
        &s.amount,
    );
}

/// get_escrow returns correct state immediately after deposit.
#[test]
fn test_escrow_state_after_deposit() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);

    let amount: i128 = 10_000_000;
    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.token_addr,
        &amount,
    );

    let escrow = contract.get_escrow();
    assert_eq!(escrow.client, s.client_addr, "client mismatch");
    assert_eq!(escrow.freelancer, s.freelancer_addr, "freelancer mismatch");
    assert_eq!(escrow.amount, amount, "amount mismatch");
    assert_eq!(escrow.token, s.token_addr, "token mismatch");
}

/// Zero-amount deposit is rejected.
#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_zero_amount_deposit_rejected() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);

    contract.deposit(&s.client_addr, &s.freelancer_addr, &s.token_addr, &0);
}
