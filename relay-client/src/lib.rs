use relay_api_types::{
    GetDeliveredPayloadsQueryParams, GetDeliveredPayloadsResponse, GetReceivedBidsQueryParams,
    GetReceivedBidsResponse, GetValidatorRegistrationQueryParams, GetValidatorRegistrationResponse,
    GetValidatorsResponse, SubmitBlockQueryParams, SubmitBlockRequest, SubmitBlockResponse,
};
use reqwest::Client;
use serde::Deserialize;
use types::eth_spec::EthSpec;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    InvalidJson(serde_json::Error, String),
    ServerMessage(String),
    StatusCode(http::StatusCode),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

pub struct RelayClient {
    client: Client,
    base_url: String,
}

impl RelayClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    async fn build_response<T>(&self, response: reqwest::Response) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let text = response.text().await;

        if status.is_success() {
            let text = text?;
            serde_json::from_str(&text).map_err(|e| Error::InvalidJson(e, text))
        } else if let Ok(message) = text {
            Err(Error::ServerMessage(message))
        } else {
            Err(Error::StatusCode(status))
        }
    }

    pub async fn submit_block<E>(
        &self,
        query_params: SubmitBlockQueryParams,
        body: SubmitBlockRequest<E>,
    ) -> Result<SubmitBlockResponse, Error>
    where
        E: EthSpec,
    {
        let url = format!("{}/relay/v1/builder/blocks", self.base_url);
        let response = self
            .client
            .post(&url)
            .query(&query_params)
            .json(&body)
            .send()
            .await?;

        self.build_response(response).await
    }

    pub async fn get_validators<E>(&self) -> Result<GetValidatorsResponse, Error>
    where
        E: EthSpec,
    {
        let url = format!("{}/relay/v1/builder/validators", self.base_url);
        let response = self.client.get(&url).send().await?;

        self.build_response(response).await
    }

    pub async fn get_delivered_payloads(
        &self,
        query_params: GetDeliveredPayloadsQueryParams,
    ) -> Result<GetDeliveredPayloadsResponse, Error> {
        let url = format!(
            "{}/relay/v1/data/bidtraces/proposer_payload_delivered",
            self.base_url
        );
        let response = self.client.get(&url).query(&query_params).send().await?;

        self.build_response(response).await
    }

    pub async fn get_received_bids(
        &self,
        query_params: GetReceivedBidsQueryParams,
    ) -> Result<GetReceivedBidsResponse, Error> {
        let url = format!(
            "{}/relay/v1/data/bidtraces/builder_blocks_received",
            self.base_url
        );
        let response = self.client.get(&url).query(&query_params).send().await?;

        self.build_response(response).await
    }

    pub async fn get_validator_registration(
        &self,
        query_params: GetValidatorRegistrationQueryParams,
    ) -> Result<GetValidatorRegistrationResponse, Error> {
        let url = format!("{}/relay/v1/data/validator_registration", self.base_url);
        let response = self.client.get(&url).query(&query_params).send().await?;

        self.build_response(response).await
    }
}
