use axum::{
    async_trait,
    body::Body,
    extract::{FromRequest, Query, Request, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, RequestExt, Router,
};
use bytes::Bytes;
use http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use relay_api_types::{
    GetDeliveredPayloadsQueryParams, GetReceivedBidsQueryParams,
    GetValidatorRegistrationQueryParams, Response as RelayResponse, SubmitBlockQueryParams,
    SubmitBlockRequest,
};
use serde::Serialize;
use tracing::error;
use types::eth_spec::EthSpec;

use crate::{builder::Builder, data::Data};

/// Setup API Server.
pub fn new<I, A, E>(api_impl: I) -> Router
where
    E: EthSpec,
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: Builder<E> + Data + 'static,
{
    // build our application with a route
    Router::new()
        .route("/relay/v1/builder/blocks", post(submit_block::<I, A, E>))
        .route(
            "/relay/v1/builder/validators",
            get(get_validators::<I, A, E>),
        )
        .route(
            "/relay/v1/data/bidtraces/builder_blocks_received",
            get(get_received_bids::<I, A>),
        )
        .route(
            "/relay/v1/data/bidtraces/proposer_payload_delivered",
            get(get_delivered_payloads::<I, A>),
        )
        .route(
            "/relay/v1/data/validator_registration",
            get(get_validator_registration::<I, A>),
        )
        .with_state(api_impl)
}

async fn build_response<T>(result: RelayResponse<T>) -> Result<Response<Body>, StatusCode>
where
    T: Serialize + Send + 'static,
{
    let response_builder = Response::builder();

    let resp = match result {
        RelayResponse::Success(body) => {
            let mut response = response_builder.status(200);

            if let Some(response_headers) = response.headers_mut() {
                response_headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str("application/json").map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?,
                );
            }

            let body_content = tokio::task::spawn_blocking(move || {
                serde_json::to_vec(&body).map_err(|e| {
                    error!(error = ?e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
            })
            .await
            .map_err(|e| {
                error!(error = ?e);
                StatusCode::INTERNAL_SERVER_ERROR
            })??;

            response.body(Body::from(body_content)).map_err(|e| {
                error!(error = ?e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
        }
        RelayResponse::Error(body) => {
            let mut response = response_builder.status(body.code);

            if let Some(response_headers) = response.headers_mut() {
                response_headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str("application/json").map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?,
                );
            }

            let body_content = tokio::task::spawn_blocking(move || {
                serde_json::to_vec(&body).map_err(|e| {
                    error!(error = ?e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
            })
            .await
            .map_err(|e| {
                error!(error = ?e);
                StatusCode::INTERNAL_SERVER_ERROR
            })??;

            response.body(Body::from(body_content)).map_err(|e| {
                error!(error = ?e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
        }
    };

    resp
}

/// SubmitBlock - POST /relay/v1/builder/blocks
#[tracing::instrument(skip_all)]
async fn submit_block<I, A, E>(
    Query(query_params): Query<SubmitBlockQueryParams>,
    State(api_impl): State<I>,
    JsonOrSsz(body): JsonOrSsz<SubmitBlockRequest<E>>,
) -> Result<Response<Body>, StatusCode>
where
    E: EthSpec,
    I: AsRef<A> + Send + Sync,
    A: Builder<E>,
{
    let result = api_impl.as_ref().submit_block(query_params, body).await;
    build_response(result).await
}

/// GetValidators - GET /relay/v1/builder/validators
#[tracing::instrument(skip_all)]
async fn get_validators<I, A, E>(State(api_impl): State<I>) -> Result<Response<Body>, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: Builder<E>,
    E: EthSpec,
{
    let result = api_impl.as_ref().get_validators().await;
    build_response(result).await
}

/// GetDeliveredPayloads - GET /relay/v1/data/bidtraces/proposer_payload_delivered
#[tracing::instrument(skip_all)]
async fn get_delivered_payloads<I, A>(
    Query(query_params): Query<GetDeliveredPayloadsQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response<Body>, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: Data,
{
    let result = api_impl.as_ref().get_delivered_payloads(query_params).await;
    build_response(result).await
}

/// GetReceivedBids - GET /relay/v1/data/bidtraces/builder_blocks_received
#[tracing::instrument(skip_all)]
async fn get_received_bids<I, A>(
    Query(query_params): Query<GetReceivedBidsQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response<Body>, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: Data,
{
    let result = api_impl.as_ref().get_received_bids(query_params).await;
    build_response(result).await
}

/// GetValidatorRegistration - GET /relay/v1/data/validator_registration
#[tracing::instrument(skip_all)]
async fn get_validator_registration<I, A>(
    Query(query_params): Query<GetValidatorRegistrationQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response<Body>, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: Data,
{
    let result = api_impl
        .as_ref()
        .get_validator_registration(query_params)
        .await;
    build_response(result).await
}

#[must_use]
#[derive(Debug, Clone, Copy, Default)]
struct Ssz<T>(T);

#[async_trait]
impl<T, S> FromRequest<S> for Ssz<T>
where
    T: ssz::Decode,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type_header = req.headers().get(CONTENT_TYPE);
        let content_type = content_type_header.and_then(|value| value.to_str().ok());

        if let Some(content_type) = content_type {
            if content_type.starts_with("application/octet-stream") {
                let bytes = Bytes::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                return Ok(T::from_ssz_bytes(&bytes)
                    .map(Ssz)
                    .map_err(|_| StatusCode::BAD_REQUEST.into_response())?);
            }
        }

        Err(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response())
    }
}

#[must_use]
#[derive(Debug, Clone, Copy, Default)]
struct JsonOrSsz<T>(T);

#[async_trait]
impl<T, S> FromRequest<S> for JsonOrSsz<T>
where
    T: serde::de::DeserializeOwned + ssz::Decode + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type_header = req.headers().get(CONTENT_TYPE);
        let content_type = content_type_header.and_then(|value| value.to_str().ok());

        if let Some(content_type) = content_type {
            if content_type.starts_with("application/json") {
                let Json(payload) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(Self(payload));
            }

            if content_type.starts_with("application/octet-stream") {
                let Ssz(payload) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(Self(payload));
            }
        }

        Err(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response())
    }
}
