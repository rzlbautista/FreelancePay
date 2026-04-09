#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Symbol,
};

const ESCROW_KEY: Symbol = symbol_short!("ESCROW");

/// On-chain escrow record.
#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub client: Address,
    pub freelancer: Address,
    pub arbitrator: Address, // neutral party who can resolve disputes
    pub token: Address,
    pub amount: i128,
    pub deadline: u64, // Unix timestamp; 0 = no deadline
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {
    /// Client locks tokens into escrow.
    /// `deadline` is a Unix timestamp (0 = no deadline).
    /// `arbitrator` is the neutral party allowed to resolve disputes.
    pub fn deposit(
        env: Env,
        client: Address,
        freelancer: Address,
        arbitrator: Address,
        token: Address,
        amount: i128,
        deadline: u64,
    ) {
        client.require_auth();

        assert!(amount > 0, "Amount must be positive");
        assert!(
            !env.storage().instance().has(&ESCROW_KEY),
            "Escrow already exists"
        );

        // Transfer tokens from client into this contract
        token::Client::new(&env, &token).transfer(
            &client,
            &env.current_contract_address(),
            &amount,
        );

        env.storage().instance().set(
            &ESCROW_KEY,
            &Escrow {
                client: client.clone(),
                freelancer: freelancer.clone(),
                arbitrator: arbitrator.clone(),
                token,
                amount,
                deadline,
            },
        );

        env.events().publish(
            (symbol_short!("deposit"), client),
            (freelancer, amount, deadline),
        );
    }

    /// Client approves work and releases funds to the freelancer.
    /// Only the original client may call this.
    pub fn release(env: Env, caller: Address) {
        caller.require_auth();

        let escrow: Escrow = env
            .storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found");

        assert!(
            caller == escrow.client,
            "Unauthorized: only the client can release funds"
        );

        env.storage().instance().remove(&ESCROW_KEY);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.freelancer,
            &escrow.amount,
        );

        env.events().publish(
            (symbol_short!("release"), escrow.client),
            (escrow.freelancer, escrow.amount),
        );
    }

    /// Client cancels before work is approved and reclaims locked funds.
    /// Only the original client may call this.
    pub fn cancel(env: Env, caller: Address) {
        caller.require_auth();

        let escrow: Escrow = env
            .storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found");

        assert!(
            caller == escrow.client,
            "Unauthorized: only the client can cancel"
        );

        env.storage().instance().remove(&ESCROW_KEY);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.client,
            &escrow.amount,
        );

        env.events().publish(
            (symbol_short!("cancel"), escrow.client.clone()),
            escrow.amount,
        );
    }

    /// Arbitrator resolves a dispute.
    /// `pay_freelancer = true`  → funds go to freelancer (work accepted).
    /// `pay_freelancer = false` → funds refunded to client (work rejected).
    /// Only the designated arbitrator may call this.
    pub fn resolve(env: Env, caller: Address, pay_freelancer: bool) {
        caller.require_auth();

        let escrow: Escrow = env
            .storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found");

        assert!(
            caller == escrow.arbitrator,
            "Unauthorized: only the arbitrator can resolve disputes"
        );

        env.storage().instance().remove(&ESCROW_KEY);

        let recipient = if pay_freelancer {
            escrow.freelancer.clone()
        } else {
            escrow.client.clone()
        };

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &recipient,
            &escrow.amount,
        );

        env.events().publish(
            (symbol_short!("resolve"), escrow.arbitrator),
            (recipient, escrow.amount, pay_freelancer),
        );
    }

    /// Returns the current escrow state. Panics if none exists.
    pub fn get_escrow(env: Env) -> Escrow {
        env.storage()
            .instance()
            .get(&ESCROW_KEY)
            .expect("No escrow found")
    }
}

#[cfg(test)]
mod test;
