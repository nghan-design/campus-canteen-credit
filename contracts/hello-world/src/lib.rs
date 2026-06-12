#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    symbol_short, Address, Env, String,
};

const DAY: u32 = 17280;

#[contracttype]
pub enum DataKey {
    Admin,
    Balance(Address),
    TotalSupply,
    Name,
    Symbol,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotAdmin = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
}

#[contract]
pub struct CampusCredit;

#[contractimpl]
impl CampusCredit {
    /// Initialize contract (called once)
    pub fn initialize(env: Env, admin: Address, name: String, symbol: String) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::TotalSupply, &0_i128);

        env.storage().instance().extend_ttl(6 * DAY, 7 * DAY);
    }

    /// Admin creates new credits
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let balance = Self::balance(env.clone(), to.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(balance + amount));

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Balance(to.clone()), 29 * DAY, 30 * DAY);

        let supply: i128 = env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply + amount));

        env.events().publish((symbol_short!("mint"), to), amount);

        Ok(())
    }

    /// Transfer credits
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        from.require_auth();

        let from_balance = Self::balance(env.clone(), from.clone());

        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let to_balance = Self::balance(env.clone(), to.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Balance(from.clone()), 29 * DAY, 30 * DAY);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Balance(to.clone()), 29 * DAY, 30 * DAY);

        env.events()
            .publish((symbol_short!("transfer"), from, to), amount);

        Ok(())
    }

    /// Get balance of an address
    pub fn balance(env: Env, addr: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(addr))
            .unwrap_or(0)
    }

    /// Token name
    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .unwrap()
    }

    /// Token symbol
    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .unwrap()
    }

    /// Total supply
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }
}