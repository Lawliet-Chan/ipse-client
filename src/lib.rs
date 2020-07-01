mod calls;

use calls::{
    AccountId, Balance, CreateOrderCallExt, DeleteCallExt, IpseRuntime as Runtime, OrdersStoreExt,
};
use http::Uri;
use ipfs_api::{IpfsClient, TryFromUri};
use keccak_hasher::KeccakHasher;
use sp_keyring::AccountKeyring;
use std::collections::HashMap;
use sub_runtime::ipse::Order;
use substrate_subxt::{Client as SubClient, ClientBuilder, Error as SubError, PairSigner, Signer};
use triehash::ordered_trie_root;

pub struct Client {
    sub_cli: SubClient<Runtime>,
    ipfs_cli: IpfsClient,
    key_to_id: HashMap<Vec<u8>, u64>,
}

impl Client {
    pub fn new(chain_url: String, ipfs_url: String) -> Self {
        let sub_cli = async_std::task::block_on(async {
            ClientBuilder::<Runtime>::new()
                .set_url(chain_url)
                .build()
                .await
                .unwrap()
        });
        let key_to_id = HashMap::new();

        let uri = ipfs_url.parse::<Uri>().expect("url parse failed");
        let ipfs_cli = IpfsClient::build_with_base_uri(uri);
        Self {
            sub_cli,
            key_to_id,
            ipfs_cli,
        }
    }

    pub fn add_file(
        &self,
        key: Vec<u8>,
        file: Vec<u8>,
        miners: Vec<AccountId>,
        days: u64,
    ) -> Result<(), SubError> {
        let merkle_root = Self::make_merkle_root(&file);
        let data_length = file.len() as u64;
        self.call_create_order(key, merkle_root, data_length, miners, days)?;
    }

    pub fn get_file(&self, key: Vec<u8>) {}

    pub fn delete_file(&self, key: Vec<u8>) {
        let id = self.get_order_id_from_chain(key);
    }

    fn make_merkle_root(file: &Vec<u8>) -> [u8; 32] {
        let mut iter = file.chunks(64);
        let mut chunks = Vec::new();
        while let Some(chunk) = iter.next() {
            chunks.push(chunk)
        }
        ordered_trie_root::<KeccakHasher, _>(chunks)
    }

    fn get_order_id(&self, key: Vec<u8>) -> Result<Option<u64>, SubError> {
        if let Some(id) = self.key_to_id.get(key.as_ref()).cloned() {
            Ok(Some(id))
        } else {
            self.get_order_id_from_chain(key)
        }
    }

    fn get_order_id_from_chain(&self, key: Vec<u8>) -> Result<Option<u64>, SubError> {
        async_std::task::block_on(async {
            let account_id = PairSigner::new(AccountKeyring::Alice.pair()).account_id();
            let orders: Vec<Order<AccountId, Balance>> = self.sub_cli.orders(None).await?;
            let mut iter = orders.iter();
            let mut idx = 0_u64;
            while let Some(order) = iter.next() {
                if order.key == key && &order.user == account_id {
                    return Ok(Some(idx));
                } else {
                    idx += 1
                }
            }
            Ok(None)
        })
    }

    fn call_create_order(
        &self,
        key: Vec<u8>,
        merkle_root: [u8; 32],
        data_length: u64,
        miners: Vec<AccountId>,
        days: u64,
    ) -> Result<(), SubError> {
        async_std::task::block_on(async move {
            let signer = PairSigner::new(AccountKeyring::Alice.pair());
            self.sub_cli
                .create_order(&signer, key, merkle_root, data_length, miners, days)
                .await?;
            Ok(())
        })
    }

    fn call_delete(&self, id: u64) -> Result<(), SubError> {
        async_std::task::block_on(async move {
            let signer = PairSigner::new(AccountKeyring::Alice.pair());
            self.sub_cli.delete(&signer, id).await?;
            Ok(())
        })
    }
}