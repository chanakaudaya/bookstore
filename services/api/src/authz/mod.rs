use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use crate::common::{AppError, AppState};

/// Claims extracted from a Keycloak-issued JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

impl Claims {
    pub fn has_role(&self, role: &str) -> bool {
        self.realm_access
            .as_ref()
            .map(|ra| ra.roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }

    pub fn is_customer(&self) -> bool {
        self.has_role("customer")
    }

    pub fn is_tenant_admin(&self) -> bool {
        self.has_role("tenant_admin")
    }

    pub fn is_platform_admin(&self) -> bool {
        self.has_role("platform_admin")
    }
}

/// Request extension that carries the validated claims.
#[derive(Clone)]
pub struct AuthenticatedUser(pub Claims);

// ---------------------------------------------------------------------------
// JWKS cache — fetches and caches the Keycloak public keys
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Clone)]
pub struct JwksCache {
    jwks_url: String,
    cache: Arc<RwLock<Option<JwkSet>>>,
}

impl JwksCache {
    pub fn new(jwks_url: &str) -> Self {
        Self {
            jwks_url: jwks_url.to_string(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn refresh(&self) -> anyhow::Result<()> {
        let jwks: JwkSet = reqwest::get(&self.jwks_url).await?.json().await?;
        let mut cache = self.cache.write().await;
        *cache = Some(jwks);
        Ok(())
    }

    pub async fn decoding_key_for(&self, kid: &str) -> anyhow::Result<DecodingKey> {
        {
            let cache = self.cache.read().await;
            if let Some(jwks) = &*cache {
                if let Some(key) = find_key(jwks, kid) {
                    return Ok(key);
                }
            }
        }
        // Key not found — refresh and retry once
        self.refresh().await?;
        let cache = self.cache.read().await;
        let jwks = cache.as_ref().ok_or_else(|| anyhow::anyhow!("JWKS cache empty after refresh"))?;
        find_key(jwks, kid).ok_or_else(|| anyhow::anyhow!("No matching key for kid: {kid}"))
    }
}

fn find_key(jwks: &JwkSet, kid: &str) -> Option<DecodingKey> {
    jwks.keys
        .iter()
        .find(|k| k.kid == kid)
        .map(|k| DecodingKey::from_rsa_components(&k.n, &k.e).unwrap())
}

// ---------------------------------------------------------------------------
// JWT validation middleware
// ---------------------------------------------------------------------------

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer(&req).ok_or(AppError::Unauthorized)?;

    let header = decode_header(token).map_err(|_| AppError::Unauthorized)?;
    let kid = header.kid.ok_or(AppError::Unauthorized)?;

    let decoding_key = state.jwks.decoding_key_for(&kid).await.map_err(|e| {
        warn!(error = %e, "JWKS lookup failed");
        AppError::Unauthorized
    })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&state.config.keycloak_issuer]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(AuthenticatedUser(token_data.claims));
    Ok(next.run(req).await)
}

fn extract_bearer(req: &Request) -> Option<&str> {
    req.headers()
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}
