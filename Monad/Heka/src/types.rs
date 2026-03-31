use nun::IdentityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyxIdentity {
    pub id: IdentityId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppAlias {
    pub app: String,
    pub alias: String,
}
