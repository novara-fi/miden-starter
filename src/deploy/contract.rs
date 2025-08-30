use rand::RngCore;
use std::{
    fs,
    sync::{Arc, OnceLock},
};

use miden_assembly::{
    DefaultSourceManager, Library, LibraryPath,
    ast::{Module, ModuleKind},
};
use miden_client::{
    Client, ClientError, Felt, Word,
    account::{AccountBuilder, AccountId, AccountStorageMode, AccountType, StorageSlot},
    transaction::{TransactionId, TransactionKernel, TransactionRequestBuilder, TransactionScript},
};
use miden_lib::account::auth::NoAuth;
use miden_objects::{account::AccountComponent, assembly::Assembler, vm::AdviceMap};

use crate::deploy::constants::{
    CALCULATE_SCRIPT_PATH, CALCULATOR_CODE_PATH, CALCULATOR_LIBRARY_PATH,
};

static CALCULATOR_CODE: OnceLock<String> = OnceLock::new();
static CALCULATE_SCRIPT_CODE: OnceLock<String> = OnceLock::new();

pub struct Contract<'a> {
    id: AccountId,
    client: &'a mut Client,
}

impl<'a> Contract<'a> {
    pub async fn build_contract(client: &'a mut Client) -> Result<Self, ClientError> {
        client.sync_state().await?;

        let calculator_code = CALCULATOR_CODE.get_or_init(|| {
            fs::read_to_string(CALCULATOR_CODE_PATH).expect("Failed to read calculator")
        });

        let assembler = TransactionKernel::assembler().with_debug_mode(true);
        let calculator_component = AccountComponent::compile(
            calculator_code.clone(),
            assembler,
            vec![StorageSlot::Value([Felt::new(0); 4])],
        )?
        .with_supports_all_types();

        let mut seed = [0_u8; 32];
        client.rng().fill_bytes(&mut seed);

        let (calculator_contract, calculator_seed) = AccountBuilder::new(seed)
            .account_type(AccountType::RegularAccountImmutableCode)
            .storage_mode(AccountStorageMode::Network)
            .with_component(calculator_component)
            .with_auth_component(NoAuth)
            .build()
            .unwrap();

        client
            .add_account(&calculator_contract, Some(calculator_seed), false)
            .await
            .unwrap();

        Ok(Contract {
            id: calculator_contract.id(),
            client,
        })
    }

    pub async fn calculate(
        &mut self,
        operand_stack: [Felt; 4],
        advice_map: AdviceMap,
    ) -> Result<TransactionId, ClientError> {
        let script_code = CALCULATE_SCRIPT_CODE.get_or_init(|| {
            fs::read_to_string(CALCULATE_SCRIPT_PATH).expect("Failed to read increment.masm")
        });

        self.build_and_submit_tx(script_code.clone(), operand_stack, advice_map)
            .await
    }

    pub async fn get_result(&self) -> Result<Word, ClientError> {
        let account = self.client.get_account(self.id).await?;
        let storage = account
            .ok_or_else(|| ClientError::AccountDataNotFound(self.id))?
            .account()
            .storage()
            .get_item(0)?
            .into();
        Ok(storage)
    }

    pub const fn id(&self) -> AccountId {
        self.id
    }

    async fn build_and_submit_tx(
        &mut self,
        script_code: String,
        operand_stack: [Felt; 4],
        advice_map: AdviceMap,
    ) -> Result<TransactionId, ClientError> {
        let calculator_code = CALCULATOR_CODE
            .get()
            .expect("Calculator code not initialized");

        let assembler = TransactionKernel::assembler().with_debug_mode(true);
        let library = create_library(calculator_code, CALCULATOR_LIBRARY_PATH)
            .unwrap_or_else(|err| panic!("Failed to create library: {err}"));

        let tx_script = TransactionScript::compile(
            script_code.clone(),
            assembler.with_library(&library).unwrap(),
        )
        .unwrap_or_else(|err| panic!("Failed to compile transaction script: {err}"));

        let tx_request = TransactionRequestBuilder::new()
            .custom_script(tx_script.clone())
            .script_arg(operand_stack)
            .extend_advice_map(advice_map)
            .build()?;

        let tx_result = self.client.new_transaction(self.id, tx_request).await?;

        let tx_id = tx_result.executed_transaction().id();

        self.client.submit_transaction(tx_result).await?;
        self.client.sync_state().await?;

        Ok(tx_id)
    }
}

fn create_library(
    account_code: &str,
    library_path: &str,
) -> Result<Library, Box<dyn std::error::Error>> {
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library).parse_str(
        LibraryPath::new(library_path)?,
        account_code,
        &source_manager,
    )?;
    let library = assembler.clone().assemble_library([module])?;
    Ok(library)
}
