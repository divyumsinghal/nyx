use nun::IdentityId;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyxIdentity {
    pub id: IdentityId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppAlias {
    pub app: String,
    pub alias: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KratosSession {
    pub identity: KratosIdentity,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KratosIdentity {
    pub id: String,
    pub traits: KratosIdentityTraits,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KratosIdentityTraits {
    pub email: Option<String>,
    pub phone: Option<String>,
}
