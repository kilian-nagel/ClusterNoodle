use std::env;

pub struct EnvVariables {}

impl EnvVariables {
    pub fn get_conf_path(&self) -> String {
        let home = env::var("HOME").expect("HOME not set");
        return String::from(format!("{}/.config/ClusterNoodle", home));
    }

    pub fn get_docker_file_path(&self) -> String {
        return format!("{}/docker-compose.yml", self.get_conf_path());
    }

    pub fn get_conf_file_path(&self) -> String {
        return format!("{}/conf.cluster_noodle", self.get_conf_path());
    }

    pub fn get_env_file_path(&self) -> String {
        return format!("{}/app.env", self.get_conf_path());
    }

    pub fn get_usable_ip_script_path(&self) -> String {
        return String::from("/opt/ClusterNoodle/scripts/usable_ip_adress.sh");
    }

    pub fn get_docker_dashboard_frontend_image_tag(&self) -> Result<String, dotenvy::Error> {
        return dotenvy::var("DOCKER_DASHBOARD_FRONTEND_IMAGE_TAG");
    }

    pub fn get_docker_dashboard_backend_image_tag(&self) -> Result<String, dotenvy::Error> {
        return dotenvy::var("DOCKER_DASHBOARD_BACKEND_IMAGE_TAG");
    }

    pub fn get_docker_dashboard_agent_image_tag(&self) -> Result<String, dotenvy::Error> {
        return dotenvy::var("DOCKER_DASHBOARD_AGENT_IMAGE_TAG");
    }
}
