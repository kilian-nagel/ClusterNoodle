use serde::Deserialize;

#[derive(Deserialize)]
pub struct EnvConfig {
    pub database_user: Option<String>,
    pub database_password: Option<String>,
    pub database_name: Option<String>,
    pub database_rootpassword: Option<String>,
}
