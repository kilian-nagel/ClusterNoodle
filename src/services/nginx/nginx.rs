pub struct NginxConfig;

impl NginxConfig {
    pub fn get_config_path() -> String {
        return ("/opt/ClusterNoodle/src/services/nginx/nginx.conf").to_string();
    }
}