#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Symbol, Vec,
};

// ─────────────────────────────────────────────
//  Storage Keys
// ─────────────────────────────────────────────
const ADMIN: Symbol = symbol_short!("ADMIN");
const TOKEN: Symbol = symbol_short!("TOKEN");

// ─────────────────────────────────────────────
//  Data Types
// ─────────────────────────────────────────────

/// Represents an employee registered in the system.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Employee {
    /// Employer who registered this employee.
    pub employer: Address,
    /// Total wages earned so far in the current pay period (in stroops / token units).
    pub earned_wages: i128,
    /// How much the employee has already advanced (withdrawn early).
    pub advanced_amount: i128,
    /// Flat fee charged per advance (in token units). Set by employer.
    pub fee_per_advance: i128,
    /// Whether the employee is active (employer can deactivate).
    pub active: bool,
}

/// Maps employee address → Employee struct.
fn employee_key(employee: &Address) -> (Symbol, Address) {
    (symbol_short!("EMP"), employee.clone())
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────

#[contract]
pub struct PaydayAdvance;

#[contractimpl]
impl PaydayAdvance {
    // ── Initialisation ───────────────────────

    /// Deploy the contract.
    /// * `admin`  – platform admin address (can upgrade, withdraw collected fees)
    /// * `token`  – Stellar asset contract address used for payments (e.g. USDC)
    pub fn initialize(env: Env, admin: Address, token: Address) {
        // Prevent re-initialisation
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&TOKEN, &token);
    }

    // ── Employer actions ─────────────────────

    /// Register a new employee and credit their earned wages.
    /// Must be called by the employer.  The employer must have pre-approved
    /// the contract to pull `earned_wages` tokens on their behalf.
    pub fn register_employee(
        env: Env,
        employer: Address,
        employee: Address,
        earned_wages: i128,
        fee_per_advance: i128,
    ) {
        employer.require_auth();
        assert!(earned_wages >= 0, "earned_wages must be non-negative");
        assert!(fee_per_advance >= 0, "fee must be non-negative");

        let key = employee_key(&employee);

        // If already registered, just update wages
        if env.storage().persistent().has(&key) {
            let mut emp: Employee = env.storage().persistent().get(&key).unwrap();
            assert!(emp.employer == employer, "not your employee");
            emp.earned_wages = earned_wages;
            env.storage().persistent().set(&key, &emp);
        } else {
            let emp = Employee {
                employer: employer.clone(),
                earned_wages,
                advanced_amount: 0,
                fee_per_advance,
                active: true,
            };
            env.storage().persistent().set(&key, &emp);
        }

        // Fund the contract escrow from employer wallet
        let token_id: Address = env.storage().instance().get(&TOKEN).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer_from(
            &env.current_contract_address(),
            &employer,
            &env.current_contract_address(),
            &earned_wages,
        );

        env.events().publish(
            (symbol_short!("register"), employer),
            (employee, earned_wages),
        );
    }

    /// At end of pay period: settle all remaining wages to an employee
    /// and reset their advance balance.
    pub fn settle_payroll(env: Env, employer: Address, employee: Address) {
        employer.require_auth();

        let key = employee_key(&employee);
        let mut emp: Employee = env
            .storage()
            .persistent()
            .get(&key)
            .expect("employee not found");

        assert!(emp.employer == employer, "not your employee");

        let remaining = emp.earned_wages - emp.advanced_amount;
        if remaining > 0 {
            let token_id: Address = env.storage().instance().get(&TOKEN).unwrap();
            let token_client = token::Client::new(&env, &token_id);
            token_client.transfer(&env.current_contract_address(), &employee, &remaining);
        }

        // Reset for next pay period
        emp.earned_wages = 0;
        emp.advanced_amount = 0;
        env.storage().persistent().set(&key, &emp);

        env.events().publish(
            (symbol_short!("settle"), employer),
            (employee, remaining),
        );
    }

    /// Deactivate / reactivate an employee.
    pub fn set_employee_status(
        env: Env,
        employer: Address,
        employee: Address,
        active: bool,
    ) {
        employer.require_auth();
        let key = employee_key(&employee);
        let mut emp: Employee = env
            .storage()
            .persistent()
            .get(&key)
            .expect("employee not found");
        assert!(emp.employer == employer, "not your employee");
        emp.active = active;
        env.storage().persistent().set(&key, &emp);
    }

    // ── Employee actions ─────────────────────

    /// Employee requests an early wage withdrawal.
    /// * `amount` – net amount the employee wants to receive (fee is charged on top)
    pub fn request_advance(env: Env, employee: Address, amount: i128) {
        employee.require_auth();
        assert!(amount > 0, "amount must be positive");

        let key = employee_key(&employee);
        let mut emp: Employee = env
            .storage()
            .persistent()
            .get(&key)
            .expect("employee not found");

        assert!(emp.active, "employee is not active");

        let total_deduction = amount + emp.fee_per_advance;
        let available = emp.earned_wages - emp.advanced_amount;

        assert!(
            total_deduction <= available,
            "insufficient earned wages for this advance"
        );

        // Transfer net amount to employee
        let token_id: Address = env.storage().instance().get(&TOKEN).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&env.current_contract_address(), &employee, &amount);

        // Fee stays in contract (admin can withdraw via collect_fees)
        emp.advanced_amount += total_deduction;
        env.storage().persistent().set(&key, &emp);

        env.events().publish(
            (symbol_short!("advance"), employee.clone()),
            (amount, emp.fee_per_advance),
        );
    }

    // ── Admin actions ────────────────────────

    /// Admin withdraws accumulated platform fees.
    pub fn collect_fees(env: Env, to: Address) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let token_id: Address = env.storage().instance().get(&TOKEN).unwrap();
        let token_client = token::Client::new(&env, &token_id);
        let balance = token_client.balance(&env.current_contract_address());
        assert!(balance > 0, "no fees to collect");

        token_client.transfer(&env.current_contract_address(), &to, &balance);
    }

    /// Transfer admin role to a new address.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &new_admin);
    }

    // ── View functions ───────────────────────

    /// Return all details for a given employee.
    pub fn get_employee(env: Env, employee: Address) -> Employee {
        let key = employee_key(&employee);
        env.storage()
            .persistent()
            .get(&key)
            .expect("employee not found")
    }

    /// How much can the employee still advance this pay period?
    pub fn available_to_advance(env: Env, employee: Address) -> i128 {
        let key = employee_key(&employee);
        let emp: Employee = env
            .storage()
            .persistent()
            .get(&key)
            .expect("employee not found");
        let gross_available = emp.earned_wages - emp.advanced_amount;
        // Subtract fee from what the user "sees" as available
        let net = gross_available - emp.fee_per_advance;
        if net < 0 { 0 } else { net }
    }

    /// Return current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).unwrap()
    }

    /// Return the token contract address.
    pub fn get_token(env: Env) -> Address {
        env.storage().instance().get(&TOKEN).unwrap()
    }
}

// ─────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env,
    };

    fn create_token(env: &Env, admin: &Address) -> (Address, TokenClient, StellarAssetClient) {
        let contract_address = env.register_stellar_asset_contract(admin.clone());
        let token = TokenClient::new(env, &contract_address);
        let asset_admin = StellarAssetClient::new(env, &contract_address);
        (contract_address, token, asset_admin)
    }

    #[test]
    fn test_full_advance_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);

        let (token_id, token, asset_admin) = create_token(&env, &admin);

        // Mint 1000 tokens to employer
        asset_admin.mint(&employer, &1_000);

        // Deploy contract
        let contract_id = env.register_contract(None, PaydayAdvance);
        let client = PaydayAdvanceClient::new(&env, &contract_id);

        client.initialize(&admin, &token_id);

        // Employer registers employee with 500 earned wages and fee of 10
        client.register_employee(&employer, &employee, &500, &10);

        // Employee requests advance of 100 (will cost 110 total from wages)
        assert_eq!(client.available_to_advance(&employee), 490);
        client.request_advance(&employee, &100);

        // Employee should have received 100 tokens
        assert_eq!(token.balance(&employee), 100);

        // Advance balance updated
        let emp = client.get_employee(&employee);
        assert_eq!(emp.advanced_amount, 110); // 100 net + 10 fee

        // Settle payroll: remaining 390 goes to employee
        client.settle_payroll(&employer, &employee);
        assert_eq!(token.balance(&employee), 490); // 100 advance + 390 settlement
    }

    #[test]
    #[should_panic(expected = "insufficient earned wages")]
    fn test_advance_exceeds_earned() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);

        let (token_id, _, asset_admin) = create_token(&env, &admin);
        asset_admin.mint(&employer, &500);

        let contract_id = env.register_contract(None, PaydayAdvance);
        let client = PaydayAdvanceClient::new(&env, &contract_id);

        client.initialize(&admin, &token_id);
        client.register_employee(&employer, &employee, &200, &5);

        // Try to advance more than earned — should panic
        client.request_advance(&employee, &300);
    }
}