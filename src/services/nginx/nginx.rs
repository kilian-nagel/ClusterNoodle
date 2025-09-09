pub struct NginxConfig;

impl NginxConfig {
    pub fn get_config_path() -> String {
        format!("{}/src/services/nginx/nginx.conf", env!("CARGO_MANIFEST_DIR"))
    }
}