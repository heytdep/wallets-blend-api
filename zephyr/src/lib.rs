use serde::{Deserialize, Serialize};
use utils::{
    get_new_sequence_number, get_reserve_config, get_reserves, handle_tx_response, strkey_to_hash,
};
use zephyr_sdk::{
    protocols::blend::storage::ReserveConfig,
    soroban_sdk::{self, contracttype, Address, Symbol, TryIntoVal},
    utils::{address_from_str, address_to_alloc_string},
    EnvClient,
};

mod utils;

// Making type assumptions about storage types
#[contracttype]
pub enum DataKey {
    Admin,
}

impl DataKey {
    fn get_admin(env: &EnvClient, vault: [u8; 32]) -> String {
        let value: Address = env
            .read_contract_entry_by_key(vault, Self::Admin)
            .unwrap()
            .unwrap();
        address_to_alloc_string(env, value)
    }
}

#[derive(Serialize)]
pub struct StandardResponse {
    status: u32,
    message: String,
}

#[derive(Deserialize)]
pub struct InitializeRequest {
    vault: String,
    admin: String,
    pool: String,
    take_rate: i64,
}

#[derive(Deserialize)]
pub struct AddReserveRequest {
    vault: String,
    reserve: String,
}

#[derive(Deserialize)]
pub struct SetAdminRequest {
    vault: String,
    new_admin: String,
}

#[derive(Deserialize)]
pub struct SetTakeRateRequest {
    vault: String,
    take_rate: i64,
}

#[derive(Deserialize)]
pub struct ClaimFeesRequest {
    vault: String,
    reserve: String,
    to: String,
    amount: i64,
}

#[derive(Deserialize)]
pub struct ClaimEmissionsRequest {
    vault: String,
    to: String,
}

#[derive(Deserialize)]
pub struct LiquidityRequest {
    vault: String,
    // Either "deposit" or "withdraw".
    action: String,
    reserve: String,
    user: String,
    amount: i64,
}

//
// ADMIN FUNCTIONS
//
#[no_mangle]
pub extern "C" fn initialize() {
    let env = EnvClient::empty();
    let InitializeRequest {
        admin,
        pool,
        take_rate,
        vault,
    } = env.read_request_body();
    let sequence = get_new_sequence_number(&env, strkey_to_hash(&admin));

    let take_rate: i128 = take_rate.try_into().unwrap();
    let args = (
        address_from_str(&env, &admin),
        address_from_str(&env, &pool),
        take_rate,
    )
        .try_into_val(env.soroban())
        .unwrap();

    let simulated = env.simulate_contract_call_to_tx(
        admin,
        sequence,
        strkey_to_hash(&vault),
        Symbol::new(&env.soroban(), "initialize"),
        args,
    );

    let response = handle_tx_response(simulated);

    env.conclude(response);
}

#[no_mangle]
pub extern "C" fn add_reserve_vault() {
    let env = EnvClient::empty();
    let AddReserveRequest { reserve, vault } = env.read_request_body();
    let vault_hash = strkey_to_hash(&vault);

    let admin = DataKey::get_admin(&env, vault_hash);
    let reserve_idx = get_reserve_config(&env, vault_hash, &reserve).index;
    let args = (reserve_idx, address_from_str(&env, &reserve))
        .try_into_val(env.soroban())
        .unwrap();

    let sequence = get_new_sequence_number(&env, strkey_to_hash(&admin));
    let simulated = env.simulate_contract_call_to_tx(
        admin,
        sequence,
        vault_hash,
        Symbol::new(&env.soroban(), "add_reserve_vault"),
        args,
    );

    let response = handle_tx_response(simulated);
    env.conclude(response);
}

#[no_mangle]
pub extern "C" fn set_admin() {
    let env = EnvClient::empty();
    let SetAdminRequest { vault, new_admin } = env.read_request_body();
    let vault_hash = strkey_to_hash(&vault);
    let old_admin = DataKey::get_admin(&env, vault_hash);

    let args = (address_from_str(&env, &new_admin),)
        .try_into_val(env.soroban())
        .unwrap();

    let sequence = get_new_sequence_number(&env, strkey_to_hash(&old_admin));
    let simulated = env.simulate_contract_call_to_tx(
        old_admin,
        sequence,
        vault_hash,
        Symbol::new(&env.soroban(), "set_admin"),
        args,
    );

    let response = handle_tx_response(simulated);
    env.conclude(response);
}

#[no_mangle]
pub extern "C" fn set_take_rate() {
    let env = EnvClient::empty();
    let SetTakeRateRequest { vault, take_rate } = env.read_request_body();
    let vault_hash = strkey_to_hash(&vault);
    let admin = DataKey::get_admin(&env, vault_hash);
    let take_rate: i128 = take_rate.try_into().unwrap();
    let args = (take_rate,).try_into_val(env.soroban()).unwrap();

    let sequence = get_new_sequence_number(&env, strkey_to_hash(&admin));
    let simulated = env.simulate_contract_call_to_tx(
        admin,
        sequence,
        vault_hash,
        Symbol::new(&env.soroban(), "set_take_rate"),
        args,
    );

    let response = handle_tx_response(simulated);
    env.conclude(response);
}

#[no_mangle]
pub extern "C" fn claim_fees() {
    let env = EnvClient::empty();
    let ClaimFeesRequest {
        vault,
        reserve,
        to,
        amount,
    } = env.read_request_body();
    let vault_hash = strkey_to_hash(&vault);
    let admin = DataKey::get_admin(&env, vault_hash);
    let amount: i128 = amount.try_into().unwrap();

    let args = (
        address_from_str(&env, &reserve),
        address_from_str(&env, &to),
        amount,
    )
        .try_into_val(env.soroban())
        .unwrap();

    let sequence = get_new_sequence_number(&env, strkey_to_hash(&admin));
    let simulated = env.simulate_contract_call_to_tx(
        admin,
        sequence,
        vault_hash,
        Symbol::new(&env.soroban(), "claim_fees"),
        args,
    );

    let response = handle_tx_response(simulated);
    env.conclude(response);
}

#[no_mangle]
pub extern "C" fn claim_emissions() {
    let env = EnvClient::empty();
    let ClaimEmissionsRequest { vault, to } = env.read_request_body();
    let vault_hash = strkey_to_hash(&vault);
    let admin = DataKey::get_admin(&env, vault_hash);

    let all_reserves = get_reserves(&env, vault_hash);
    let mut claimed_tokens_ids = soroban_sdk::Vec::new(&env.soroban());

    for ReserveConfig { index, .. } in all_reserves {
        claimed_tokens_ids.push_back(index * 2);
        claimed_tokens_ids.push_back(index * 2 + 1);
    }

    let args = (claimed_tokens_ids, address_from_str(&env, &to))
        .try_into_val(env.soroban())
        .unwrap();

    let sequence = get_new_sequence_number(&env, strkey_to_hash(&admin));
    let simulated = env.simulate_contract_call_to_tx(
        admin,
        sequence,
        vault_hash,
        Symbol::new(&env.soroban(), "claim_emissions"),
        args,
    );

    let response = handle_tx_response(simulated);
    env.conclude(response);
}

#[no_mangle]
pub extern "C" fn liquidity() {
    let env = EnvClient::empty();
    let LiquidityRequest {
        vault,
        action,
        reserve,
        user,
        amount,
    } = env.read_request_body();
    let vault_hash = strkey_to_hash(&vault);

    let amount: i128 = amount.try_into().unwrap();
    let args = (
        address_from_str(&env, &reserve),
        address_from_str(&env, &user),
        amount,
    )
        .try_into_val(env.soroban())
        .unwrap();

    let sequence = get_new_sequence_number(&env, strkey_to_hash(&user));
    let simulated = env.simulate_contract_call_to_tx(
        user,
        sequence,
        vault_hash,
        Symbol::new(&env.soroban(), &action),
        args,
    );

    let response = handle_tx_response(simulated);
    env.conclude(response);
}
