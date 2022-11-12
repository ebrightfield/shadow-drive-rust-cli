use anyhow::anyhow;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use solana_sdk::bs58;
use solana_sdk::signature::Signer;

const SIGNIN_MSG: &str = "Sign in to GenesysGo Shadow Platform.";
const SIGNIN_URL: &str = "https://portal.genesysgo.net/api/signin";
const SIGNIN_URL_STEP2: &str = "https://portal.genesysgo.net/api/premium/token";

pub const GENESYSGO_AUTH_KEYWORD: &str = "genesysgo";

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesysGoAuthResponse {
    pub token: String,
    pub user: GenesysGoUser,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesysGoUser {
    pub id: u64,
    pub public_key: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesysGoAuth {
    message: String, // signed and base-58 encoded SIGNIN_MSG
    signer: String,
}

impl GenesysGoAuth {
    pub async fn sign_in(signer: &dyn Signer, account_id: &str) -> anyhow::Result<String> {
        let signature = signer.sign_message(SIGNIN_MSG.as_bytes());
        let body = Self {
            message: bs58::encode(signature.as_ref()).into_string(),
            signer: signer.pubkey().to_string(),
        };
        let client = reqwest::Client::new();
        // First request, acquire a JWT needed for the second request.
        let resp = client
            .post(Url::parse(SIGNIN_URL)?)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;
        let auth_resp: GenesysGoAuthResponse = serde_json::from_str(&resp.text().await?)?;
        // Second request, acquire JWT to be included with authenticated requests.
        let step2_url = SIGNIN_URL_STEP2.to_owned() + "/" + account_id;
        let token1 = format!("Bearer {}", auth_resp.token);
        let resp = client
            .post(Url::parse(&step2_url)?)
            .header("Content-Type", "application/json")
            .header("Authorization", &token1)
            .send()
            .await?;
        let resp: TokenResponse = serde_json::from_str(&resp.text().await?)?;
        Ok(resp.token)
    }
}

pub fn parse_account_id_from_url(genesysgo_url: String) -> anyhow::Result<String> {
    if !genesysgo_url.contains("genesysgo") {
        return Err(anyhow!("Not a genesysgo URL, cannot infer Account ID"));
    }
    let pieces = genesysgo_url.split("/");
    let last = pieces
        .last()
        .ok_or(anyhow!("Could not parse genesysgo url: {}", &genesysgo_url))?;
    Ok(last.to_string())
}
