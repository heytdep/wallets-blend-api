use crate::StandardResponse;
use zephyr_sdk::{
    protocols::blend::storage::{PoolDataKey, ReserveConfig, RES_LIST_KEY},
    soroban_sdk::{Address, Symbol},
    utils::address_from_str,
    EnvClient, SdkError, TransactionResponse,
};

pub(crate) fn strkey_to_hash(strkey: &str) -> [u8; 32] {
    match stellar_strkey::Strkey::from_string(&strkey).unwrap() {
        stellar_strkey::Strkey::Contract(hash) => hash.0,
        stellar_strkey::Strkey::PublicKeyEd25519(hash) => hash.0,
        _ => {
            EnvClient::empty().log().error("Unsupported strkey", None);
            panic!()
        }
    }
}

pub(crate) fn handle_tx_response(
    simulated: Result<TransactionResponse, SdkError>,
) -> StandardResponse {
    if let Ok(txresult) = simulated {
        if let Some(tx) = txresult.tx {
            StandardResponse {
                status: 200,
                message: tx,
            }
        } else {
            StandardResponse {
                status: 400,
                message: txresult.error.unwrap_or("unknown error".into()),
            }
        }
    } else {
        StandardResponse {
            status: 500,
            message: "Internal simulation error".into(),
        }
    }
}

pub(crate) fn get_new_sequence_number(env: &EnvClient, account: [u8; 32]) -> i64 {
    env.read_account_from_ledger(account)
        .unwrap()
        .unwrap()
        .seq_num as i64
        + 1
}

pub(crate) fn get_reserve_config(env: &EnvClient, vault: [u8; 32], reserve: &str) -> ReserveConfig {
    let reserve_config = PoolDataKey::ResConfig(address_from_str(&env, reserve));
    env.read_contract_entry_by_key(vault, reserve_config)
        .unwrap()
        .unwrap()
}

pub(crate) fn get_reserves(env: &EnvClient, vault: [u8; 32]) -> Vec<ReserveConfig> {
    let address_list: zephyr_sdk::soroban_sdk::Vec<Address> = env
        .read_contract_entry_by_key(vault, Symbol::new(&env.soroban(), RES_LIST_KEY))
        .unwrap()
        .unwrap();
    let mut configs = Vec::new();

    for address in address_list {
        let reserve_config = PoolDataKey::ResConfig(address);
        let config = env
            .read_contract_entry_by_key(vault, reserve_config)
            .unwrap()
            .unwrap();

        configs.push(config);
    }

    configs
}
