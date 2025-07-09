use crate::config;
use std::process::Command;

pub fn check_existing_cluster() -> bool {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("ca")
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("Error response from daemon") {
        return false;
    }

    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    return true;
}

pub fn init_cluster(config: &config::ClusterConfig) -> () {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("init")
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

pub fn destroy_cluster() -> () {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("leave")
        .arg("--force")
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
