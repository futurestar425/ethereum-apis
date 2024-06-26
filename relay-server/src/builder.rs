use async_trait::async_trait;
use relay_api_types::{
    GetValidatorsResponse, SubmitBlockQueryParams, SubmitBlockRequest, SubmitBlockResponse,
};
use types::eth_spec::EthSpec;

/// Builder
#[async_trait]
pub trait Builder<E: EthSpec> {
    /// Get a list of validator registrations for validators scheduled to propose in the current and next epoch. .
    ///
    /// GetValidators - GET /relay/v1/builder/validators
    async fn get_validators(&self) -> GetValidatorsResponse;

    /// Submit a new block to the relay..
    ///
    /// SubmitBlock - POST /relay/v1/builder/blocks
    async fn submit_block(
        &self,
        query_params: SubmitBlockQueryParams,
        body: SubmitBlockRequest<E>,
    ) -> SubmitBlockResponse<E>;
}
