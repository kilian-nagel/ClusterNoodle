use crate::config;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
        let output = Command::new("ssh-copy-id")
            .arg(&target)
            .output()
            .expect(&format!("Failed to execute ssh-copy-id for : {}", target));

        if !output.status.success() {
            println!(
                "Failed to execute ssh-copy-id for {}: {}",
                target,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
