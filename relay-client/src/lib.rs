use relay_api_types::{
    GetDeliveredPayloadsQueryParams, GetDeliveredPayloadsResponse, GetReceivedBidsQueryParams,
    GetReceivedBidsResponse, GetValidatorRegistrationQueryParams, GetValidatorRegistrationResponse,
    GetValidatorsResponse, SubmitBlockQueryParams, SubmitBlockRequest, SubmitBlockResponse,
    ValidatorsResponse,
};
use reqwest::Client;
use serde::Deserialize;
use types::{
    eth_spec::EthSpec, Address, PublicKeyBytes, Signature, SignedValidatorRegistrationData, Slot,
    ValidatorRegistrationData,
};

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

#[test]
fn get_validators_serde() {
    let value = r#"[
      {
        "slot": "9485504",
        "validator_index": "352280",
        "entry": {
          "message": {
            "fee_recipient": "0x388C818CA8B9251b393131C08a736A67ccB19297",
            "gas_limit": "30000000",
            "timestamp": "1709833797",
            "pubkey": "0x8224b3102db5dec20968111be999d54ee24c0712714793064f06ff239e1f71ac470cf5fdd2d2145a2991a94098f626e8"
          },
          "signature": "0x9709ff207fd1e5d2c141e476272ba652a2a0bb3700c1497a7363fb77f3e03b4810b19d07e78f22ff085dc902a8514a0ad0306ab87510d25cb84d768529662ff9e605010552059b395b76ad904a3936a4df7ccb32c970ac466ab5d27abdc"
        }
      },
      {
        "slot": "9485555",
        "validator_index": "991347",
        "entry": {
          "message": {
            "fee_recipient": "0xa6b4854fDf65873846f8060BBC68a363679AF17E",
            "gas_limit": "30000000",
            "timestamp": "1710424919",
            "pubkey": "0x86cc77e06d4c3903bf0159629466fdd7ae2b8ea9b2c4468c58c63de2ff505d220fa642d13770bf2021828314901d2692"
          },
          "signature": "0x81e2101401345d38d161e3171ebb822540fbd2878ce59f93e2d4301e20498e458146925b3ae32c460cc46fb96239e4c91396343c6c386e2e1fd635dffbc5892c5971e82bb84352f7aa9777a201a2f983e8f7fab6411dfe871515cea13e70afc4"
        }
      },
      {
        "slot": "9485557",
        "validator_index": "1038322",
        "entry": {
          "message": {
            "fee_recipient": "0x7e2a2FA2a064F693f0a55C5639476d913Ff12D05",
            "gas_limit": "30000000",
            "timestamp": "1706519279",
            "pubkey": "0xa31c0bc713d6faf1c76ff1952dc853b3b3c6d04c118f00aca3b36693cc3321e2d0b61668ff4ebbc3df04cc57a14afbaf"
          },
          "signature": "0xb0366be21e97b9d003dce8f359d91773dd2f82ddefd786666974bf4c26a8ff2808f896d1d86bbee9bad3e59e925facb206c8c105b9acc300b6e44d2e8e6b4bf3ae7d46dbe8a79624286f52601d7ab14e6f2d09a2984593fea934a31ecfeb42a4"
        }
      },
      {
        "slot": "9485558",
        "validator_index": "698853",
        "entry": {
          "message": {
            "fee_recipient": "0x388C818CA8B9251b393131C08a736A67ccB19297",
            "gas_limit": "30000000",
            "timestamp": "1707135021",
            "pubkey": "0xb59825c97a504a1a1daf170eb90b83448054abb32d96415baf91ced36161e7f63985a94b32508bfb8012b4d49f1e4ec6"
          },
          "signature": "0x912402369c5f64ffeb24a11d72cd43eb9ea1ed748ee68deec7b95182ed0075939b44d3d274130cbaef66c449adfbeb2f0782fb33a7c13d0b7dfeb5fa250952dc42d76a34e3d565bc9e5e397d41308026106a314a094f841344192bf8a83a17b1"
        }
      },
      {
        "slot": "9485559",
        "validator_index": "977044",
        "entry": {
          "message": {
            "fee_recipient": "0x388C818CA8B9251b393131C08a736A67ccB19297",
            "gas_limit": "30000000",
            "timestamp": "1709646562",
            "pubkey": "0xa37ce0c00322e910383f229aff53f865dfbb78de62e4601846551524db3f345a8b5b62a0f866b926baea47e21bb90903"
          },
          "signature": "0x8a5fc680ee010c424a3007e414d6c8d610f48025501278dae8764dd2ed062c9360f252a7900f755c1bec8da1f13e408a0267a79b906df932a370b02ca146e5d118217a3f34dcc9ca1d50147d1aee77f1c61169c0974e26b2d2896c343044b0d5"
        }
      },
      {
        "slot": "9485560",
        "validator_index": "1078748",
        "entry": {
          "message": {
            "fee_recipient": "0xD6E4aA932147A3FE5311dA1b67D9e73da06F9cEf",
            "gas_limit": "30000000",
            "timestamp": "1709716427",
            "pubkey": "0x9112e06a03f42218ed6317e93467d4ba95d3a1b6618615ab368bbe58a74791f0b2b6fe413939f71e0eb1ddb24e054129"
          },
          "signature": "0xa7fd20a267f0ad9c606ee40a526f54a4aeb3bb58d26123e1b6856559192de96636cc0b8e4b1517da8f0634b03a805049165c1db1da48bfa3ee047f8194bdc5604ab4f04527dc7ca93d7310f2a5dc488ee98036b6a77b5bf093bb6a36f1045b19"
        }
      },
      {
        "slot": "9485561",
        "validator_index": "1254067",
        "entry": {
          "message": {
            "fee_recipient": "0x4a0ca3fc506b809Ef17E851844198e5192C30404",
            "gas_limit": "30000000",
            "timestamp": "1719843861",
            "pubkey": "0x95fc9f536af83076e7b651833e8e0086d12ae7fab12e5bb2e33d3fc5f739388c3b789e0485f03a55e242f7b5e325a826"
          },
          "signature": "0xafbb9c8b2611d4f7cd526d496941347f6e0caa32850554fa933493f285f8675a80ae4d3ef01d938925868adee51a6700174563e84b72c1952edc571ec5cfcdc956be0114cc62eab6cf6d23a4ea40fa669e84db2b4098a588f6e26df5f6f9f905"
        }
      },
      {
        "slot": "9485562",
        "validator_index": "1378052",
        "entry": {
          "message": {
            "fee_recipient": "0x388C818CA8B9251b393131C08a736A67ccB19297",
            "gas_limit": "30000000",
            "timestamp": "1714894345",
            "pubkey": "0x98d599227665b75e84e80a288f4ad33991a1654419755f38291a6421855692256d0ae825968c4bb42175a6a27146cae2"
          },
          "signature": "0x96342b741ac189b86a7c9dc32af74034abc54de9187cc998656c8bfd344d64dbbd07a2b331dc62780bec541f625b958d138f5578cf132af8c154de19a98ae8c9375632a7823cdfc12191b8e3552ff25ea31f2127a7667a34c188fbaedcf9b016"
        }
      },
      {
        "slot": "9485563",
        "validator_index": "1115106",
        "entry": {
          "message": {
            "fee_recipient": "0xeBec795c9c8bBD61FFc14A6662944748F299cAcf",
            "gas_limit": "30000000",
            "timestamp": "1708969065",
            "pubkey": "0xb16fb0837f5e2e6cab44cabaafec7d0803793ec368403a97961d38232835af10a7479bd2b06fe20dda9b4ad8b3a8afb0"
          },
          "signature": "0xb371ddb68c352362c5276bd3da84edbe3fb65f7f9a105faac20103b08ba174b6fefa487024892252d6470648ab32b55801c64ccdad53bcb6ec8a137cf22d3be36b6d3e9676aee45e27e2bc26d4be48ee5ab5b6e7cbc6aba9acbe0eadf45b7657"
        }
      },
      {
        "slot": "9485564",
        "validator_index": "1251535",
        "entry": {
          "message": {
            "fee_recipient": "0xeC68CD160F0d8107304d667bf6Ae81394d8f88be",
            "gas_limit": "30000000",
            "timestamp": "1709122535",
            "pubkey": "0x84560f40cc1f5d064ffcf9b2e0831082e4524cbe803afbb05f992a9320b4d141519a3eefd4642e97900a5f93001aa102"
          },
          "signature": "0xb65fd7287f0f9e5fa99c91cbb21b7b34c0665916651a8e9a870343c43f0b4ca76eafaf0e2bc86fdf0483ed9fea2e1f3403e06bea27a462cb81e16e1836ee1445398f93da9864c263917e9dfb6a8799202ebbd70724057b3a569a2a60bbcafa5f"
        }
      },
      {
        "slot": "9485565",
        "validator_index": "1452686",
        "entry": {
          "message": {
            "fee_recipient": "0xEe5F5c53CE2159fC6DD4b0571E86a4A390D04846",
            "gas_limit": "30000000",
            "timestamp": "1719033432",
            "pubkey": "0x8f5fa1a0e04448c6a1877aa34f8420438e4007f22dace1104b49c49be01a232153b5b240e465bb61c437be028fb46d62"
          },
          "signature": "0xb82f40344822fd6707c74c5f0dc7718df9570622f63818e0fcc430e5c81e02eb8327c4f1a42f210b7b0b1368a8473f12030a89ab9c4aaac72d2aba5a110555828c61e18be573bda8dd6b98ca80144af5ceb41bd724c685281539dad07968e25f"
        }
      },
      {
        "slot": "9485567",
        "validator_index": "1001109",
        "entry": {
          "message": {
            "fee_recipient": "0xf960451867C62942Ca97BD192bB7a5df32D8BB02",
            "gas_limit": "30000000",
            "timestamp": "1711341443",
            "pubkey": "0x948333521aa5d85ddbee2eea612bb77b2c5c0e3f0082d9635f34965ac50c06496c94a1e30cb4b1196b3caae53598b87c"
          },
          "signature": "0xadbbcb54a6f25588689ffb35b47ac56de253ce1641793d405325fc36b5c3b5ab888f3c105b1f4667d998f629402b19d70df2bdab7ceb8d1e802b208b8751384fc05b4420f983dc543ca7b7e9a58f3463b29b06d677f9bb93667c03ec0a1ecb38"
        }
      }
    ]"#;

    let data = GetValidatorsResponse::Success(vec![ValidatorsResponse {
        slot: Slot::new(0),
        validator_index: 1,
        entry: SignedValidatorRegistrationData {
            message: ValidatorRegistrationData {
                fee_recipient: Address::zero(),
                gas_limit: 1,
                timestamp: 1,
                pubkey: PublicKeyBytes::empty(),
            },
            signature: Signature::empty(),
        },
    }]);

    let b = serde_json::to_string(&data).unwrap();
    dbg!(b);

    let test: GetValidatorsResponse = serde_json::from_str(&value).unwrap();
    dbg!(test);
    panic!()
}
