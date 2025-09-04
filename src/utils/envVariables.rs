use std::env;
use std::fs;

pub struct envVariables {}

impl envVariables {
    pub fn get_conf_path(&self) -> String {
        let home = env::var("HOME").expect("HOME not set");
        return String::from(format!("{}/.config/ClusterNoodle", home));
    }

    pub fn get_docker_file_path(&self) -> String {
        return format!("{}/docker-compose.yml", self.get_conf_path());
    }

    pub fn get_conf_file_path(&self) -> String {
        return format!("{}/.config/conf.cluster_noodle", self.get_conf_path());
    }
}
