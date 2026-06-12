use std::time::{Duration, SystemTime};

use reqwest::Response;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{auth::Account, error::LauncherError, http::HttpClient, result::Result};

const DEVICE_CODE_URL: &str = "https://login.live.com/oauth20_connect.srf";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const XBOX_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";

#[derive(Debug, Clone)]
pub struct MicrosoftAuth<'a> {
    client: &'a HttpClient,
    client_id: String,
}

#[derive(Debug, Clone)]
pub struct MicrosoftLogin<'a> {
    pub auth: MicrosoftAuth<'a>,
    pub device_code: DeviceCode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    pub interval: u64,
}

impl DeviceCode {
    pub fn browser_uri(&self) -> String {
        if let Some(uri) = self.verification_uri_complete.as_ref() {
            return uri.clone();
        }

        if self.verification_uri.contains("microsoft.com/link") {
            return format!(
                "https://login.live.com/oauth20_remoteconnect.srf?otc={}",
                self.user_code
            );
        }

        self.verification_uri.clone()
    }
}

#[derive(Debug, Deserialize)]
struct MicrosoftToken {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftPending {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XboxToken {
    token: String,
    display_claims: XboxDisplayClaims,
}

#[derive(Debug, Deserialize)]
struct XboxDisplayClaims {
    xui: Vec<XboxUserInfo>,
}

#[derive(Debug, Deserialize)]
struct XboxUserInfo {
    uhs: String,
}

#[derive(Debug, Deserialize)]
struct MinecraftToken {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Entitlements {
    items: Vec<EntitlementItem>,
}

#[derive(Debug, Deserialize)]
struct EntitlementItem {
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct XboxAuthRequest<'a> {
    properties: XboxAuthProperties<'a>,
    relying_party: &'a str,
    token_type: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct XboxAuthProperties<'a> {
    auth_method: &'a str,
    site_name: &'a str,
    rps_ticket: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct XstsRequest<'a> {
    properties: XstsProperties<'a>,
    relying_party: &'a str,
    token_type: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct XstsProperties<'a> {
    sandbox_id: &'a str,
    user_tokens: [&'a str; 1],
}

impl<'a> MicrosoftAuth<'a> {
    pub fn create(http: &'a HttpClient, client_id: impl Into<String>) -> Self {
        Self {
            client: http,
            client_id: client_id.into(),
        }
    }

    pub async fn start_device_login(&self) -> Result<DeviceCode> {
        let response = self
            .client
            .post(DEVICE_CODE_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scope", SCOPE),
                ("response_type", "device_code"),
            ])
            .send()
            .await?;

        decode_response(response, "Microsoft device code").await
    }

    pub async fn poll_device_login(&self, device_code: &DeviceCode) -> Result<Option<Account>> {
        let response = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code.device_code.as_str()),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let token: MicrosoftToken = response.json().await?;
            return self.login_to_minecraft(&token.access_token).await.map(Some);
        }

        let pending: MicrosoftPending = response.json().await?;
        match pending.error.as_deref() {
            Some("authorization_pending" | "slow_down") => Ok(None),
            Some(error) => Err(LauncherError::Auth(
                pending
                    .error_description
                    .unwrap_or_else(|| error.to_string()),
            )),
            None => Err(LauncherError::Auth(
                "Microsoft login failed without an error message".to_string(),
            )),
        }
    }

    async fn login_to_minecraft(&self, microsoft_access_token: &str) -> Result<Account> {
        let xbox = self.login_to_xbox(microsoft_access_token).await?;
        let xsts = self.login_to_xsts(&xbox.token).await?;
        let user_hash = xsts
            .display_claims
            .xui
            .first()
            .ok_or_else(|| LauncherError::Auth("XSTS did not return a user".to_string()))?
            .uhs
            .clone();
        let minecraft = self
            .login_to_minecraft_services(&user_hash, &xsts.token)
            .await?;

        self.ensure_game_ownership(&minecraft.access_token).await?;
        let profile = self.minecraft_profile(&minecraft.access_token).await?;
        let expires_at = SystemTime::now() + Duration::from_secs(minecraft.expires_in);

        Ok(Account::microsoft(
            profile.id,
            profile.name,
            minecraft.access_token,
            expires_at,
        ))
    }

    async fn login_to_xbox(&self, microsoft_access_token: &str) -> Result<XboxToken> {
        let request = XboxAuthRequest {
            properties: XboxAuthProperties {
                auth_method: "RPS",
                site_name: "user.auth.xboxlive.com",
                rps_ticket: microsoft_access_token,
            },
            relying_party: "http://auth.xboxlive.com",
            token_type: "JWT",
        };

        let response = self
            .client
            .post(XBOX_AUTH_URL)
            .json(&request)
            .send()
            .await?;

        decode_response(response, "Xbox Live authentication").await
    }

    async fn login_to_xsts(&self, xbox_token: &str) -> Result<XboxToken> {
        let request = XstsRequest {
            properties: XstsProperties {
                sandbox_id: "RETAIL",
                user_tokens: [xbox_token],
            },
            relying_party: "rp://api.minecraftservices.com/",
            token_type: "JWT",
        };

        let response = self
            .client
            .post(XSTS_AUTH_URL)
            .json(&request)
            .send()
            .await?;

        decode_response(response, "XSTS authorization").await
    }

    async fn login_to_minecraft_services(
        &self,
        user_hash: &str,
        xsts_token: &str,
    ) -> Result<MinecraftToken> {
        let identity_token = format!("XBL3.0 x={user_hash};{xsts_token}");

        let response = self
            .client
            .post(MINECRAFT_AUTH_URL)
            .json(&json!({ "identityToken": identity_token }))
            .send()
            .await?;

        decode_response(response, "Minecraft Services authentication").await
    }

    async fn ensure_game_ownership(&self, access_token: &str) -> Result<()> {
        let response = self
            .client
            .get(ENTITLEMENTS_URL)
            .bearer_auth(access_token)
            .send()
            .await?;
        let entitlements: Entitlements =
            decode_response(response, "Minecraft ownership check").await?;

        let owns_minecraft = entitlements
            .items
            .iter()
            .any(|item| item.name == "game_minecraft" || item.name == "product_minecraft");

        if owns_minecraft {
            Ok(())
        } else {
            Err(LauncherError::Auth(
                "this Microsoft account does not own Minecraft Java Edition".to_string(),
            ))
        }
    }

    async fn minecraft_profile(&self, access_token: &str) -> Result<MinecraftProfile> {
        let response = self
            .client
            .get(PROFILE_URL)
            .bearer_auth(access_token)
            .send()
            .await?;

        decode_response(response, "Minecraft profile").await
    }
}

async fn decode_response<T>(response: Response, step: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();

    if status.is_success() {
        return Ok(response.json().await?);
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "no response body".to_string());
    let body = format_auth_error(step, status.as_u16(), &body);

    Err(LauncherError::Auth(body))
}

fn format_auth_error(step: &str, status: u16, body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return format!("{step} failed ({status}): {body}");
    };

    let code = value
        .get("error")
        .or_else(|| value.get("errorType"))
        .and_then(Value::as_str)
        .unwrap_or("unknown_error");
    let description = value
        .get("error_description")
        .or_else(|| value.get("errorMessage"))
        .or_else(|| value.get("message"))
        .and_then(Value::as_str)
        .unwrap_or(body);
    let description = description
        .split(" Trace ID:")
        .next()
        .unwrap_or(description)
        .trim();

    if step == "Microsoft device code" && code == "invalid_client" {
        return format!("Enable public client flows in Azure, then retry. ({code})");
    }

    format!("{step} failed ({code}): {description}")
}
