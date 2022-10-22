use solana_sdk::signature::Signer;
use reqwest::Url;
use solana_sdk::bs58;
use serde::{Serialize, Deserialize};

const SIGNIN_MSG: &str = "Sign in to GenesysGo Shadow Platform.";
const SIGNIN_URL: &str = "https://portal.genesysgo.net/api/signin";

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesysGoAuthResponse {
    pub token: String, // signed and base-58 encoded SIGNIN_MSG
    pub user: GenesysGoUser,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesysGoUser{
    pub id: u64,
    pub public_key: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesysGoAuth {
    message: String, // signed and base-58 encoded SIGNIN_MSG
    signer: String,
}

impl GenesysGoAuth {
    pub async fn sign_in(signer: &dyn Signer) -> anyhow::Result<GenesysGoAuthResponse> {
        let signature = signer.sign_message(SIGNIN_MSG.as_bytes());
        let body = Self {
            message: bs58::encode(signature.as_ref()).into_string(),
            signer: signer.pubkey().to_string(),
        };
        let client = reqwest::Client::new();
        let resp = client
            .post(Url::parse(SIGNIN_URL)?)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;
        Ok(serde_json::from_str(&resp.text().await?)?)
    }
}
