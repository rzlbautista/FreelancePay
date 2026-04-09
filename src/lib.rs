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
    pub token: Address,
    pub amount: i128,
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {
    /// Client locks `amount` of `token` into the contract.
    /// Fails if an escrow already exists or amount is not positive.
    pub fn deposit(
        env: Env,
        client: Address,
        freelancer: Address,
        token: Address,
        amount: i128,
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
                token,
                amount,
            },
        );

        env.events().publish(
            (symbol_short!("deposit"), client),
            (freelancer, amount),
        );
    }

    /// Client releases escrowed funds to the freelancer.
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

        // Pay the freelancer from the contract's token balance
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

    /// Client cancels the escrow and reclaims the locked funds.
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

        // Refund the client from the contract's token balance
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
