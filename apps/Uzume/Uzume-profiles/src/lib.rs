use std::collections::HashMap;

use async_trait::async_trait;
use heka::link_policy::LinkPolicyEngine;
use nun::{IdentityId, NyxApp, NyxError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait]
pub trait Authenticator: Clone + Send + Sync + 'static {
    async fn validate_session(&self, session_token: &str) -> Result<IdentityId>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub alias: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: bool,
    pub is_verified: bool,
    pub follower_count: u64,
    pub following_count: u64,
    pub post_count: u64,
    identity_id: IdentityId,
}

impl Profile {
    pub fn new(identity_id: IdentityId, alias: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            display_name: display_name.into(),
            bio: None,
            avatar_url: None,
            is_private: false,
            is_verified: false,
            follower_count: 0,
            following_count: 0,
            post_count: 0,
            identity_id,
        }
    }

    pub fn with_private(mut self, is_private: bool) -> Self {
        self.is_private = is_private;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfilePatch {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicProfileResponse {
    pub alias: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: bool,
    pub is_verified: bool,
    pub follower_count: u64,
    pub following_count: u64,
    pub post_count: u64,
}

impl From<Profile> for PublicProfileResponse {
    fn from(value: Profile) -> Self {
        Self {
            alias: value.alias,
            display_name: value.display_name,
            bio: value.bio,
            avatar_url: value.avatar_url,
            is_private: value.is_private,
            is_verified: value.is_verified,
            follower_count: value.follower_count,
            following_count: value.following_count,
            post_count: value.post_count,
        }
    }
}

pub struct ProfilesService<A>
where
    A: Authenticator,
{
    auth: A,
    policy: LinkPolicyEngine,
    profiles_by_identity: HashMap<IdentityId, Profile>,
    alias_to_identity: HashMap<String, IdentityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointMethod {
    Get,
    Patch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EndpointRequest {
    pub method: EndpointMethod,
    pub path: String,
    pub session_token: Option<String>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointResponse {
    pub status: u16,
    pub body: Value,
}

pub struct ProfilesEndpoints<A>
where
    A: Authenticator,
{
    service: ProfilesService<A>,
}

impl<A> ProfilesEndpoints<A>
where
    A: Authenticator,
{
    pub fn new(service: ProfilesService<A>) -> Self {
        Self { service }
    }

    pub fn insert_profile(&mut self, profile: Profile) {
        self.service.insert_profile(profile);
    }

    pub async fn handle(&mut self, request: EndpointRequest) -> EndpointResponse {
        match (request.method, request.path.as_str()) {
            (EndpointMethod::Get, "/me") => {
                let result = self
                    .service
                    .get_me(request.session_token.as_deref())
                    .await
                    .and_then(to_json_value);
                map_endpoint_result(result)
            }
            (EndpointMethod::Patch, "/me") => {
                let patch = request
                    .body
                    .ok_or_else(|| NyxError::bad_request("invalid_json", "Invalid JSON: missing request body"))
                    .and_then(|value| {
                        serde_json::from_value::<ProfilePatch>(value).map_err(NyxError::from)
                    });
                let result = match patch {
                    Ok(patch) => self
                        .service
                        .patch_me(request.session_token.as_deref(), patch)
                        .await
                        .and_then(to_json_value),
                    Err(error) => Err(error),
                };
                map_endpoint_result(result)
            }
            (EndpointMethod::Get, path) => {
                let alias = path.trim_start_matches('/');
                if alias.is_empty() {
                    return error_response(NyxError::not_found(
                        "route_not_found",
                        "Route was not found",
                    ));
                }
                let result = self
                    .service
                    .get_public_profile(alias, request.session_token.as_deref())
                    .await
                    .and_then(to_json_value);
                map_endpoint_result(result)
            }
            _ => error_response(NyxError::not_found("route_not_found", "Route was not found")),
        }
    }
}

fn map_endpoint_result(result: Result<Value>) -> EndpointResponse {
    match result {
        Ok(body) => EndpointResponse { status: 200, body },
        Err(error) => error_response(error),
    }
}

fn error_response(error: NyxError) -> EndpointResponse {
    EndpointResponse {
        status: error.status_code(),
        body: serde_json::to_value(error.to_error_response(None)).unwrap_or_else(|_| {
            serde_json::json!({
                "error": "An internal error occurred",
                "code": "internal_error"
            })
        }),
    }
}

fn to_json_value<T: Serialize>(value: T) -> Result<Value> {
    serde_json::to_value(value).map_err(NyxError::from)
}

impl<A> ProfilesService<A>
where
    A: Authenticator,
{
    pub fn new(auth: A, policy: LinkPolicyEngine) -> Self {
        Self {
            auth,
            policy,
            profiles_by_identity: HashMap::new(),
            alias_to_identity: HashMap::new(),
        }
    }

    pub fn insert_profile(&mut self, profile: Profile) {
        self.alias_to_identity
            .insert(profile.alias.clone(), profile.identity_id);
        self.profiles_by_identity.insert(profile.identity_id, profile);
    }

    pub async fn get_me(&self, session_token: Option<&str>) -> Result<PublicProfileResponse> {
        let identity = self.require_identity(session_token).await?;
        let profile = self.profile_for_identity(identity)?;
        Ok(PublicProfileResponse::from(profile.clone()))
    }

    pub async fn patch_me(
        &mut self,
        session_token: Option<&str>,
        patch: ProfilePatch,
    ) -> Result<PublicProfileResponse> {
        let identity = self.require_identity(session_token).await?;
        let profile = self
            .profiles_by_identity
            .get_mut(&identity)
            .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile was not found"))?;

        if let Some(display_name) = patch.display_name {
            profile.display_name = display_name;
        }
        if let Some(bio) = patch.bio {
            profile.bio = Some(bio);
        }
        if let Some(avatar_url) = patch.avatar_url {
            profile.avatar_url = Some(avatar_url);
        }
        if let Some(is_private) = patch.is_private {
            profile.is_private = is_private;
        }

        Ok(PublicProfileResponse::from(profile.clone()))
    }

    pub async fn get_public_profile(
        &self,
        alias: &str,
        maybe_session_token: Option<&str>,
    ) -> Result<PublicProfileResponse> {
        let owner = self
            .alias_to_identity
            .get(alias)
            .copied()
            .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile was not found"))?;

        let profile = self.profile_for_identity(owner)?;
        if !profile.is_private {
            return Ok(PublicProfileResponse::from(profile.clone()));
        }

        let viewer = match maybe_session_token {
            Some(token) => self.auth.validate_session(token).await?,
            None => {
                return Err(NyxError::forbidden(
                    "policy_denied",
                    "Cross-app access denied",
                ));
            }
        };

        if viewer == owner {
            return Ok(PublicProfileResponse::from(profile.clone()));
        }

        if self
            .policy
            .is_visible(owner, viewer, NyxApp::Uzume, NyxApp::Uzume)
        {
            return Ok(PublicProfileResponse::from(profile.clone()));
        }

        Err(NyxError::forbidden(
            "policy_denied",
            "Cross-app access denied",
        ))
    }

    async fn require_identity(&self, session_token: Option<&str>) -> Result<IdentityId> {
        let token = session_token.ok_or_else(|| {
            NyxError::unauthorized("auth_session_token_missing", "Session token is required")
        })?;

        if token.trim().is_empty() {
            return Err(NyxError::unauthorized(
                "auth_session_token_missing",
                "Session token is required",
            ));
        }

        self.auth.validate_session(token).await
    }

    fn profile_for_identity(&self, identity: IdentityId) -> Result<&Profile> {
        self.profiles_by_identity
            .get(&identity)
            .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile was not found"))
    }
}
