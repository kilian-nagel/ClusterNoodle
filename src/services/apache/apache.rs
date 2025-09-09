pub struct ApacheConfig;

impl ApacheConfig {
    pub fn get_vhost_config_path() -> String {
        format!("{}/src/services/apache/vhost.conf", env!("CARGO_MANIFEST_DIR"))
    }
}