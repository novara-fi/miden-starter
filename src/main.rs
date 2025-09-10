mod common;
mod deploy;

use deploy::account::Account;
use deploy::contract::Contract;

use crate::common::{delete_keystore_and_store, prepare_felt_vec};
use miden_client::Felt;
use miden_crypto::Word;
use miden_objects::vm::AdviceMap;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    delete_keystore_and_store().await;

    // Deploy basic account
    let mut account = Account::deploy_account().await.unwrap();
    info!("Account ID: {:?}", account.id());

    // Compile and build calculator contract
    let mut contract = Contract::build_contract(account.client()).await.unwrap();
    info!("Contract ID: {:?}", contract.id());

    let (x, y, a, b) = (1, 2, 3, 4);

    // Public inputs
    let operand_stack = Word::new([Felt::new(0), Felt::new(0), Felt::new(y), Felt::new(x)]);

    // Private inputs
    let mut advice_map = AdviceMap::default();
    advice_map.insert(prepare_felt_vec(0 as u64).into(), prepare_felt_vec(a));
    advice_map.insert(prepare_felt_vec(1 as u64).into(), prepare_felt_vec(b));

    // Calculate
    let tx_id = contract.calculate(operand_stack, advice_map).await.unwrap();
    info!(
        "View increment transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );

    let storage = contract.get_result().await.unwrap();
    let expected_result = x * a + y * b;
    assert_eq!(storage.last(), Some(&Felt::new(expected_result)));
}
