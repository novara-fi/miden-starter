mod deploy;
use deploy::account::Account;
use deploy::contract::Contract;

use miden_client::Felt;
use miden_objects::account::NetworkId;
use miden_objects::vm::AdviceMap;
use miden_starter::common::prepare_felt_vec;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Deploy basic account
    let mut account = Account::deploy_account().await.unwrap();
    info!(
        "Account ID: {:?}",
        account.id().to_bech32(NetworkId::Testnet)
    );

    // Compile and build calculator contract
    let mut contract = Contract::build_contract(account.client()).await.unwrap();
    info!(
        "Contract ID: {:?}",
        contract.id().to_bech32(NetworkId::Testnet)
    );

    let (x, y, a, b) = (1, 2, 3, 4);

    // Public inputs
    let operand_stack: [Felt; 4] = [Felt::new(0), Felt::new(0), Felt::new(y), Felt::new(x)];

    // Private inputs
    let mut advice_map = AdviceMap::default();
    advice_map.insert(
        prepare_felt_vec(0 as u64).into(),
        prepare_felt_vec(a).into(),
    );
    advice_map.insert(
        prepare_felt_vec(1 as u64).into(),
        prepare_felt_vec(b).into(),
    );

    // Calculate
    let tx_id = contract.calculate(operand_stack, advice_map).await.unwrap();
    info!(
        "View increment transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );
}
