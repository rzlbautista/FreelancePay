#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

use super::*;

// ── helpers ────────────────────────────────────────────────────────────────

struct TestSetup {
    env: Env,
    contract_id: Address,
    client_addr: Address,
    freelancer_addr: Address,
    arbitrator_addr: Address,
    token_addr: Address,
    amount: i128,
}

fn setup() -> TestSetup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FreelanceEscrow, ());

    let token_admin = Address::generate(&env);
    let token_addr = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let arbitrator_addr = Address::generate(&env);
    let amount: i128 = 5_000_000;

    // Fund client with 10 XLM worth of tokens
    StellarAssetClient::new(&env, &token_addr).mint(&client_addr, &10_000_000);

    TestSetup {
        env,
        contract_id,
        client_addr,
        freelancer_addr,
        arbitrator_addr,
        token_addr,
        amount,
    }
}

fn do_deposit(s: &TestSetup) {
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.arbitrator_addr,
        &s.token_addr,
        &s.amount,
        &0u64, // no deadline
    );
}

// ── tests ──────────────────────────────────────────────────────────────────

/// Happy path: deposit → release sends funds to freelancer.
#[test]
fn test_deposit_and_release_success() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let token = TokenClient::new(&s.env, &s.token_addr);

    do_deposit(&s);

    assert_eq!(token.balance(&s.client_addr), 5_000_000);
    assert_eq!(token.balance(&s.contract_id), 5_000_000);

    let escrow = contract.get_escrow();
    assert_eq!(escrow.client, s.client_addr);
    assert_eq!(escrow.freelancer, s.freelancer_addr);
    assert_eq!(escrow.amount, s.amount);

    contract.release(&s.client_addr);

    assert_eq!(token.balance(&s.freelancer_addr), 5_000_000);
    assert_eq!(token.balance(&s.contract_id), 0);
}

/// Only the original client may release.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_release_by_non_client_is_rejected() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let attacker = Address::generate(&s.env);

    do_deposit(&s);
    contract.release(&attacker);
}

/// Cancel returns funds to the client.
#[test]
fn test_cancel_refunds_client() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let token = TokenClient::new(&s.env, &s.token_addr);

    do_deposit(&s);
    assert_eq!(token.balance(&s.client_addr), 5_000_000);

    contract.cancel(&s.client_addr);

    assert_eq!(token.balance(&s.client_addr), 10_000_000);
    assert_eq!(token.balance(&s.contract_id), 0);
}

/// Arbitrator resolves in favour of the freelancer.
#[test]
fn test_resolve_pays_freelancer() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let token = TokenClient::new(&s.env, &s.token_addr);

    do_deposit(&s);
    contract.resolve(&s.arbitrator_addr, &true);

    assert_eq!(token.balance(&s.freelancer_addr), 5_000_000);
    assert_eq!(token.balance(&s.contract_id), 0);
}

/// Arbitrator resolves in favour of the client (refund).
#[test]
fn test_resolve_refunds_client() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let token = TokenClient::new(&s.env, &s.token_addr);

    do_deposit(&s);
    contract.resolve(&s.arbitrator_addr, &false);

    assert_eq!(token.balance(&s.client_addr), 10_000_000);
    assert_eq!(token.balance(&s.contract_id), 0);
}

/// Non-arbitrator cannot resolve.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_resolve_by_non_arbitrator_rejected() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let attacker = Address::generate(&s.env);

    do_deposit(&s);
    contract.resolve(&attacker, &true);
}

/// Double-deposit is rejected.
#[test]
#[should_panic(expected = "Escrow already exists")]
fn test_double_deposit_is_rejected() {
    let s = setup();
    do_deposit(&s);
    do_deposit(&s);
}

/// Zero-amount deposit is rejected.
#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_zero_amount_deposit_rejected() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.arbitrator_addr,
        &s.token_addr,
        &0,
        &0u64,
    );
}

/// Escrow state is correct after deposit (includes deadline and arbitrator).
#[test]
fn test_escrow_state_after_deposit() {
    let s = setup();
    let contract = FreelanceEscrowClient::new(&s.env, &s.contract_id);
    let deadline: u64 = 1_800_000_000;

    contract.deposit(
        &s.client_addr,
        &s.freelancer_addr,
        &s.arbitrator_addr,
        &s.token_addr,
        &s.amount,
        &deadline,
    );

    let escrow = contract.get_escrow();
    assert_eq!(escrow.client, s.client_addr);
    assert_eq!(escrow.freelancer, s.freelancer_addr);
    assert_eq!(escrow.arbitrator, s.arbitrator_addr);
    assert_eq!(escrow.amount, s.amount);
    assert_eq!(escrow.deadline, deadline);
}
