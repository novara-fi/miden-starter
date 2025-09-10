use rand::{RngCore, rngs::StdRng};
use std::sync::Arc;

use miden_client::{
    Client, ClientError, Word,
    account::{
        AccountBuilder, AccountId, AccountIdAddress, AccountStorageMode, AccountType, Address,
        AddressInterface, NetworkId, StorageSlot,
    },
    assembly::{DefaultSourceManager, Library, LibraryPath, Module, ModuleKind},
    keystore::FilesystemKeyStore,
    transaction::{TransactionId, TransactionKernel, TransactionRequestBuilder, TransactionScript},
};
use miden_lib::account::auth::NoAuth;
use miden_objects::{account::AccountComponent, assembly::Assembler, vm::AdviceMap};

const CALCULATOR_CODE: &str = include_str!("../../masm/accounts/calculator.masm");
static CALCULATE_SCRIPT_CODE: &str = include_str!("../../masm/scripts/calculate.masm");

pub struct Contract<'a> {
    id: AccountId,
    client: &'a mut Client<FilesystemKeyStore<StdRng>>,
}

impl<'a> Contract<'a> {
    pub async fn build_contract(
        client: &'a mut Client<FilesystemKeyStore<StdRng>>,
    ) -> Result<Self, ClientError> {
        client.sync_state().await?;

        let assembler = TransactionKernel::assembler().with_debug_mode(true);
        let calculator_component = AccountComponent::compile(
            CALCULATOR_CODE,
            assembler,
            vec![StorageSlot::Value(Word::empty())],
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
        operand_stack: Word,
        advice_map: AdviceMap,
    ) -> Result<TransactionId, ClientError> {
        self.build_and_submit_tx(CALCULATE_SCRIPT_CODE, operand_stack, advice_map)
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

    pub fn id(&self) -> String {
        let account_id_address = AccountIdAddress::new(self.id, AddressInterface::Unspecified);

        let address = Address::from(account_id_address);

        address.to_bech32(NetworkId::Testnet)
    }

    async fn build_and_submit_tx(
        &mut self,
        script_code: &str,
        operand_stack: Word,
        advice_map: AdviceMap,
    ) -> Result<TransactionId, ClientError> {
        let assembler = TransactionKernel::assembler().with_debug_mode(true);
        let library = create_library(CALCULATOR_CODE, "external_contract::calculator")
            .unwrap_or_else(|err| panic!("Failed to create library: {err}"));

        let assembler = assembler.with_dynamic_library(&library).unwrap();

        let program = assembler
            .assemble_program(script_code)
            .unwrap_or_else(|e| panic!("Failed to assemble program: {:?}", e));

        let tx_script = TransactionScript::new(program);
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
