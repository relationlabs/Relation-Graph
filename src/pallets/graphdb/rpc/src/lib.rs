//! RPC interface for the subgraph.

use std::sync::Arc;

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use subgraph_runtime_api::SubGraphApi as SubGraphRuntimeApi;

#[rpc]
pub trait SubGraphApi<BlockHash> {
    #[rpc(name = "sparql_query")]
    fn query(&self, query: String, at: Option<BlockHash>) -> Result<String>;
}

pub struct SubGraph<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> SubGraph<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> SubGraphApi<<Block as BlockT>::Hash> for SubGraph<C, Block>
    where
        Block: BlockT,
        C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
        C::Api: SubGraphRuntimeApi<Block>,
{
    fn query(&self, query: String, at: Option<<Block as BlockT>::Hash>) -> Result<String> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash)
        );

        let runtime_api_result = api.query(&at, query);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1001),
            message: "Sparql query error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
