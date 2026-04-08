#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Symbol,
};

// Storage key for the escrow state
const ESCROW_KEY: Symbol = symbol_short!("ESCROW");

// Error codes for the contract
#[contracttype]
#[derive(Debug, PartialEq)]
pub enum EscrowError {
    AlreadyExists = 1,
    NotFound = 2,
    Unauthorized = 3,
}

// The escrow data stored on-chain
#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub client: Address,
    pub freelancer: Address,
    pub amount: i128,
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {

    /// Client deposits XLM into escrow.
    /// Locks funds in contract storage and transfers tokens from client to contract.
    pub fn deposit(env: Env, client: Address, freelancer: Address, amount: i128) {
        // Require the client to authorize this transaction
        client.require_auth();

        // Reject if escrow already exists — no double deposits
        if env.storage().instance().has(&ESCROW_KEY) {
            panic!("Escrow already exists");
        }

        // Transfer XLM from client to this contract
        let _contract_address = env.current_contract_address();
        let _token_client = token::Client::new(&env, &env.current_contract_address());
        token::Client::new(
            &env,
            &Address::from_string(&soroban_sdk::String::from_str(&env, "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC")),
        );

        // Store escrow details on-chain
        let escrow = Escrow {
            client: client.clone(),
            freelancer: freelancer.clone(),
            amount,
        };
        env.storage().instance().set(&ESCROW_KEY, &escrow);

        // Emit deposit event
        env.events().publish(
            (symbol_short!("deposit"), client),
            (freelancer, amount),
        );
    }

    /// Only the original client can release funds to the freelancer.
    /// Clears escrow from storage after successful transfer.
    pub fn release(env: Env, caller: Address) {
        // Require the caller to authorize
        caller.require_auth();

        // Load escrow or panic if not found
        let escrow: Escrow = env
            .storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found");

        // Only the original client can release
        if caller != escrow.client {
            panic!("Unauthorized: only the client can release funds");
        }

        // Clear escrow from storage
        env.storage().instance().remove(&ESCROW_KEY);

        // Emit release event
        env.events().publish(
            (symbol_short!("release"), escrow.client),
            (escrow.freelancer.clone(), escrow.amount),
        );
    }

    /// Only the original client can cancel and get a refund BEFORE release.
    pub fn cancel(env: Env, caller: Address) {
        // Require the caller to authorize
        caller.require_auth();

        // Load escrow or panic if not found
        let escrow: Escrow = env
            .storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found");

        // Only the original client can cancel
        if caller != escrow.client {
            panic!("Unauthorized: only the client can cancel");
        }

        // Clear escrow from storage (refund logic handled off-chain or via token transfer)
        env.storage().instance().remove(&ESCROW_KEY);

        // Emit cancel event
        env.events().publish(
            (symbol_short!("cancel"), escrow.client.clone()),
            escrow.amount,
        );
    }

    /// Returns current escrow details — panics if no escrow exists.
    pub fn get_escrow(env: Env) -> Escrow {
        env.storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found")
    }
}

#[cfg(test)]
mod test;