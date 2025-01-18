use crate::errors::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
pub struct AzureTokenResponse {
    token_type: String,
    expires_in: i64,
    access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyVaultSecret {
    pub value: String,
}

pub struct KeyVault {
    client: Client,
    token: String,
}

impl KeyVault {
    pub async fn new() -> Result<Self, AppError> {
        let client_id = env::var("AZURE_CLIENT_ID")?;
        let tenant_id = env::var("AZURE_TENANT_ID")?;
        let client_secret = env::var("AZURE_CLIENT_SECRET")?;

        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant_id
        );

        let client = Client::new();
        let params = [
            ("client_id", client_id),
            ("scope", "https://vault.azure.net/.default".to_string()),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials".to_string()),
        ];

        let res = client
            .post(&auth_url)
            .form(&params)
            .send()
            .await?
            .json::<AzureTokenResponse>()
            .await?;

        Ok(Self {
            client,
            token: res.access_token,
        })
    }

    pub async fn get_secret(&self, secret_name: &str) -> Result<String, AppError> {
        let vault_url = env::var("AZURE_KEY_VAULT_URL")?;
        let secret_url = format!("{}/secrets/{}?api-version=7.4", vault_url, secret_name);

        let res = self
            .client
            .get(&secret_url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json::<KeyVaultSecret>()
            .await?;

        Ok(res.value)
    }
}
