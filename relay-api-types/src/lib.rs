use beacon_api_types::FullPayloadContents;
use serde::{Deserialize, Serialize};
use serde_utils::quoted_u64::Quoted;
use ssz_derive::{Decode, Encode};
use types::{
    superstruct, Address, EthSpec, ExecutionBlockHash, ExecutionPayloadBellatrix,
    ExecutionPayloadCapella, ExecutionPayloadDeneb, ExecutionPayloadElectra, PublicKeyBytes,
    Signature, SignedValidatorRegistrationData, Slot, Uint256,
};

// Builder API requests

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitBlockQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellations: Option<bool>,
}

#[superstruct(
    variants(Bellatrix, Capella, Deneb, Electra),
    variant_attributes(
        derive(Debug, Clone, Serialize, Deserialize, Encode, Decode),
        serde(bound = "E: EthSpec", deny_unknown_fields),
    ),
    map_into(ExecutionPayload),
    map_ref_into(ExecutionPayload)
)]
#[derive(Debug, Clone, Serialize, Deserialize, Encode)]
#[serde(bound = "E: EthSpec", untagged)]
#[ssz(enum_behaviour = "transparent")]
pub struct SubmitBlockRequest<E: EthSpec> {
    message: BidTraceV1,
    #[superstruct(flatten)]
    execution_payload: ExecutionPayload<E>,
    signature: Signature,
}

impl<E: EthSpec> ssz::Decode for SubmitBlockRequest<E> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    // No Eth-Consensus-Types specified https://github.com/flashbots/relay-specs/issues/36
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let Ok(req) = SubmitBlockRequestElectra::from_ssz_bytes(bytes) else {
            let Ok(req) = SubmitBlockRequestDeneb::from_ssz_bytes(bytes) else {
                let Ok(req) = SubmitBlockRequestCapella::from_ssz_bytes(bytes) else {
                    return Ok(Self::Bellatrix(
                        SubmitBlockRequestBellatrix::from_ssz_bytes(bytes)?,
                    ));
                };
                return Ok(Self::Capella(req));
            };
            return Ok(Self::Deneb(req));
        };
        Ok(Self::Electra(req))
    }
}

// Data API requests

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderBy {
    #[serde(rename = "value")]
    Value,
    #[serde(rename = "-value")]
    NegativeValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetDeliveredPayloadsQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Slot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<Slot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<ExecutionBlockHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<Quoted<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposer_pubkey: Option<PublicKeyBytes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder_pubkey: Option<PublicKeyBytes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetReceivedBidsQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<ExecutionBlockHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<Quoted<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder_pubkey: Option<PublicKeyBytes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<Slot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetValidatorRegistrationQueryParams {
    pub pubkey: PublicKeyBytes,
}

// Builder API responses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatorsResponse {
    pub slot: Slot,
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,
    pub entry: SignedValidatorRegistrationData,
}

// Data API responses

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct BidTraceV1 {
    pub slot: Slot,
    pub parent_hash: ExecutionBlockHash,
    pub block_hash: ExecutionBlockHash,
    pub builder_pubkey: PublicKeyBytes,
    pub proposer_pubkey: PublicKeyBytes,
    pub proposer_fee_recipient: Address,
    #[serde(with = "serde_utils::quoted_u64")]
    pub gas_limit: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    pub gas_used: u64,
    #[serde(with = "serde_utils::quoted_u256")]
    pub value: Uint256,
    #[serde(with = "serde_utils::quoted_u64")]
    pub block_number: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    pub num_tx: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BidTraceV2 {
    #[serde(flatten)]
    pub bid_trace: BidTraceV1,
    #[serde(with = "serde_utils::quoted_u64")]
    pub block_number: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    pub num_tx: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BidTraceV2WithTimestamp {
    #[serde(flatten)]
    pub bid_trace: BidTraceV2,
    #[serde(with = "serde_utils::quoted_i64")]
    pub timestamp: i64,
    #[serde(with = "serde_utils::quoted_i64")]
    pub timestamp_ms: i64,
}

// Response types common

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum Response<T> {
    Success(T),
    Error(ErrorResponse),
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stacktraces: Option<Vec<String>>,
}

// Builder API response types
pub type GetValidatorsResponse = Response<Vec<ValidatorsResponse>>;
pub type SubmitBlockResponse<E> = Response<FullPayloadContents<E>>;

// Data API response types
pub type GetDeliveredPayloadsResponse = Response<Vec<BidTraceV2WithTimestamp>>;
pub type GetReceivedBidsResponse = Response<Vec<BidTraceV2>>;
pub type GetValidatorRegistrationResponse = Response<SignedValidatorRegistrationData>;
