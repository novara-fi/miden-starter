use miden_client::{
    Client, ClientError, DebugMode,
    account::{
        AccountBuilder, AccountId, AccountIdAddress, AccountStorageMode, AccountType, Address,
        AddressInterface,
        component::{AuthRpoFalcon512, BasicWallet},
    },
    auth::AuthSecretKey,
    builder::ClientBuilder,
    crypto::SecretKey,
    keystore::FilesystemKeyStore,
    rpc::{Endpoint, TonicRpcClient},
};
use miden_objects::account::NetworkId;
use rand::{RngCore, rngs::StdRng};
use std::sync::Arc;

pub struct Account {
    id: AccountId,
    account_type: AccountType,
    client: Client<FilesystemKeyStore<StdRng>>,
    network_id: NetworkId,
}

impl Account {
    pub async fn deploy_account() -> Result<Self, ClientError> {
        let endpoint = Endpoint::testnet();
        let timeout_ms = 10_000;
        let rpc_api = Arc::new(TonicRpcClient::new(&endpoint, timeout_ms));

        let mut client = ClientBuilder::new()
            .rpc(rpc_api)
            .filesystem_keystore("./keystore")
            .in_debug_mode(DebugMode::Enabled)
            .build()
            .await?;

        client.sync_state().await.unwrap();

        let keystore: FilesystemKeyStore<rand::prelude::StdRng> =
            FilesystemKeyStore::new("./keystore".into()).unwrap();

        let mut user_seed = [0_u8; 32];
        client.rng().fill_bytes(&mut user_seed);

        let key_pair = SecretKey::with_rng(client.rng());
        let builder = AccountBuilder::new(user_seed)
            .account_type(AccountType::RegularAccountUpdatableCode)
            .storage_mode(AccountStorageMode::Private)
            .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
            .with_component(BasicWallet);

        let (account, seed) = builder.build().unwrap();

        client.add_account(&account, Some(seed), false).await?;

        keystore
            .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
            .unwrap();

        Ok(Account {
            id: account.id(),
            account_type: AccountType::RegularAccountUpdatableCode,
            client,
            network_id: NetworkId::Testnet,
        })
    }

    pub fn id(&self) -> String {
        let account_id_address = AccountIdAddress::new(self.id, AddressInterface::BasicWallet);

        let address = Address::from(account_id_address);

        address.to_bech32(NetworkId::Testnet)
    }

    pub fn account_type(&self) -> &AccountType {
        &self.account_type
    }

    pub fn client(&mut self) -> &mut Client<FilesystemKeyStore<StdRng>> {
        &mut self.client
    }

    pub fn network_id(&self) -> &NetworkId {
        &self.network_id
    }
}
