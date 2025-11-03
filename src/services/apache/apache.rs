pub struct ApacheConfig;

impl ApacheConfig {
    pub fn get_vhost_config_path() -> String {
        return ("/opt/ClusterNoodle/src/services/apache/vhost.conf").to_string();
    }
}