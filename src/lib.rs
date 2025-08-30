pub mod common;
pub mod deploy;

#[cfg(test)]
mod tests {
    use crate::common::{delete_keystore_and_store, prepare_felt_vec};
    pub use crate::deploy::{account::Account, contract::Contract};
    use miden_client::Felt;
    use miden_objects::{account::NetworkId, vm::AdviceMap};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    // Compile and build calculator contract
    async fn setup_account_and_contract<'a>(
        account: &'a mut Account,
    ) -> Result<Contract<'a>, Box<dyn std::error::Error>> {
        let contract = Contract::build_contract(account.client()).await?;
        println!(
            "Counter Contract ID: {:?}",
            contract.id().to_bech32(NetworkId::Testnet)
        );

        Ok(contract)
    }

    // Deploy basic wallet account
    async fn setup_account() -> Result<Account, Box<dyn std::error::Error>> {
        let account = Account::deploy_account().await?;
        println!(
            "Account ID: {:?}",
            account.id().to_bech32(NetworkId::Testnet)
        );

        Ok(account)
    }

    #[tokio::test]
    async fn calculate_success() -> TestResult {
        delete_keystore_and_store().await;

        let mut account = setup_account().await?;
        let mut contract = setup_account_and_contract(&mut account).await?;

        let (x, y, a, b) = (4, 2, 12, 9);

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
        contract.calculate(operand_stack, advice_map).await?;

        let storage = contract.get_result().await?;
        let expected_result = x * a + y * b;
        assert_eq!(storage.last(), Some(&Felt::new(expected_result)));

        Ok(())
    }
}
