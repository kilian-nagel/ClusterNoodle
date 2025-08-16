use crate::config::config;
use crate::utils::command;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub fn generate_ssh_key() {
    // Resolve ~/.ssh path
    let mut key_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    key_path.push(".ssh/cluster_noodle");

    // Convert to string path
    let key_path_str = key_path.to_str().unwrap();

    // Run ssh-keygen with arguments
    let output = Command::new("ssh-keygen")
        .args(&["-t", "rsa", "-b", "4096", "-f", key_path_str, "-N", ""])
        .output()
        .expect(&format!(
            "Failed to execute ssh-keygen for in the following path : {}",
            key_path_str
        ));

    if !output.status.success() {
        println!(
            "Erreur de génération de clé : {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn check_existing_ssh_key() -> bool {
    let mut ssh_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    ssh_dir.push(".ssh");

    let private_key = ssh_dir.join("cluster_noodle");
    let public_key = ssh_dir.join("cluster_noodle.pub");

    private_key.exists() && public_key.exists()
}

pub fn copy_ssh_key_to_machines(config: &config::ClusterConfig) {
    for node_config in &config.nodes_configs {
        let target = format!("{}@{}", node_config.username, node_config.ip);

        let mut cmd = Command::new("ssh-copy-id");
        cmd.arg(&target);

        match command::run_with_timeout(cmd, Duration::from_secs(5)) {
            Ok(Some(output)) => {
                if output.status.success() {
                    println!("Success!");
                } else {
                    println!(
                        "ssh-copy-id failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Ok(None) => {
                println!("ssh-copy-id timed out after 5 seconds for {}", target);
            }
            Err(e) => {
                println!("Failed to execute: {}", e);
            }
        }
    }
}

pub fn join_cluster(config: &config::ClusterConfig) {
    for node_config in &config.nodes_configs {
        let target = format!("{}@{}", node_config.username, node_config.ip);

        let mut cmd = Command::new("sshpass");
        cmd.arg("-p")
            .arg(&node_config.password)
            .arg("ssh")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg(&target)
            .arg(format!(
                "echo {} | sudo -S apt-get update -y && echo {} | sudo -S apt-get install -y docker.io",
                node_config.password,
                node_config.password
            ));

        match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
            Ok(Some(output)) => {
                if output.status.success() {
                    println!("Docker installed on {}", target);
                } else {
                    println!(
                        "Install failed on {}: {}",
                        target,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Ok(None) => println!("Timeout for {}", target),
            Err(e) => println!("Execution failed on {}: {}", target, e),
        }
    }
}
