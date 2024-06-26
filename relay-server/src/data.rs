use async_trait::async_trait;
use relay_api_types::{
    GetDeliveredPayloadsQueryParams, GetDeliveredPayloadsResponse, GetReceivedBidsQueryParams,
    GetReceivedBidsResponse, GetValidatorRegistrationQueryParams, GetValidatorRegistrationResponse,
};

/// Data
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Data {
    /// Get payloads that were delivered to proposers..
    ///
    /// GetDeliveredPayloads - GET /relay/v1/data/bidtraces/proposer_payload_delivered
    async fn get_delivered_payloads(
        &self,
        query_params: GetDeliveredPayloadsQueryParams,
    ) -> GetDeliveredPayloadsResponse;

    /// Get builder bid submissions..
    ///
    /// GetReceivedBids - GET /relay/v1/data/bidtraces/builder_blocks_received
    async fn get_received_bids(
        &self,
        query_params: GetReceivedBidsQueryParams,
    ) -> GetReceivedBidsResponse;

    /// Check that a validator is registered with the relay..
    ///
    /// GetValidatorRegistration - GET /relay/v1/data/validator_registration
    async fn get_validator_registration(
        &self,
        query_params: GetValidatorRegistrationQueryParams,
    ) -> GetValidatorRegistrationResponse;
}
