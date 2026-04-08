#![cfg(test)]

use super::*;
use soroban_sdk::{Address, Env};

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    /// Test 1 (Happy Path):
    /// Client deposits funds, then successfully releases to freelancer.
    #[test]
    fn test_deposit_and_release_success() {
        let env = Env::default();
        env.mock_all_auths();

        // Register the contract
        let contract_id = env.register(FreelanceEscrow, ());
        let client = FreelanceEscrowClient::new(&env, &contract_id);

        // Generate mock addresses
        let client_address = Address::generate(&env);
        let freelancer_address = Address::generate(&env);
        let amount: i128 = 5_000_000; // 5 XLM in stroops

        // Step 1: Client deposits into escrow
        client.deposit(&client_address, &freelancer_address, &amount);

        // Step 2: Verify escrow exists with correct data
        let escrow = client.get_escrow();
        assert_eq!(escrow.client, client_address);
        assert_eq!(escrow.freelancer, freelancer_address);
        assert_eq!(escrow.amount, amount);

        // Step 3: Client releases funds to freelancer
        client.release(&client_address);
    }

    /// Test 2 (Edge Case):
    /// A non-client caller tries to release funds and gets rejected.
    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_release_by_non_client_is_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        // Register the contract
        let contract_id = env.register(FreelanceEscrow, ());
        let client = FreelanceEscrowClient::new(&env, &contract_id);

        // Generate mock addresses
        let client_address = Address::generate(&env);
        let freelancer_address = Address::generate(&env);
        let random_attacker = Address::generate(&env);
        let amount: i128 = 5_000_000;

        // Client deposits into escrow
        client.deposit(&client_address, &freelancer_address, &amount);

        // Attacker tries to release — should panic with Unauthorized
        client.release(&random_attacker);
    }

    /// Test 3 (State Verification):
    /// After deposit, get_escrow returns the correct client, freelancer, and amount.
    #[test]
    fn test_escrow_state_after_deposit() {
        let env = Env::default();
        env.mock_all_auths();

        // Register the contract
        let contract_id = env.register(FreelanceEscrow, ());
        let client = FreelanceEscrowClient::new(&env, &contract_id);

        // Generate mock addresses
        let client_address = Address::generate(&env);
        let freelancer_address = Address::generate(&env);
        let amount: i128 = 10_000_000; // 10 XLM in stroops

        // Deposit into escrow
        client.deposit(&client_address, &freelancer_address, &amount);

        // Fetch escrow state and verify every field
        let escrow = client.get_escrow();

        assert_eq!(
            escrow.client, client_address,
            "Client address should match"
        );
        assert_eq!(
            escrow.freelancer, freelancer_address,
            "Freelancer address should match"
        );
        assert_eq!(
            escrow.amount, amount,
            "Escrowed amount should match deposited amount"
        );
    }
}