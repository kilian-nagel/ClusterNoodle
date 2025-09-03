use crate::ClusterConfig;
use crate::utils::command;
use std::process::Command;
use std::time::Duration;

pub fn check_existing_cluster() -> bool {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("ca")
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output.stderr);
    if output_str.contains("Error response from daemon") {
        return false;
    }

    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    return true;
}

impl ClusterConfig {
    pub fn init(&mut self) -> () {
        let output = Command::new("docker")
            .arg("swarm")
            .arg("init")
            .output()
            .unwrap();

        let output_string = String::from_utf8_lossy(&output.stdout).to_string(); // owns the String

        let output_sentences: Vec<&str> = output_string.lines().collect();

        for sentence in output_sentences {
            if sentence.trim().starts_with("docker swarm join") {
                self.cluster_docker_command = sentence.trim().to_string();
            }
        }

        if !output.status.success() {
            eprintln!("Error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    pub fn install_docker(&mut self) {
        for node_config in &self.nodes_configs {
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

    pub fn join_cluster(&self) {
        for node_config in &self.nodes_configs {
            let target = format!("{}@{}", node_config.username, node_config.ip);
            let command = &self.cluster_docker_command;

            let mut cmd = Command::new("ssh");
            cmd.arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg(&target)
                .arg(format!("{}", command));

            match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
                Ok(Some(output)) => {
                    if output.status.success() {
                        println!("{} joined the cluster", target);
                    } else {
                        println!(
                            "{} failed to join the cluster : {}",
                            target,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for execution of {} on : {}", command, target),
                Err(e) => println!("Execution of {} failed on {} : {}", command, target, e),
            }
        }
    }

    pub fn leave_cluster(&mut self) {
        for node_config in &self.nodes_configs {
            let target = format!("{}@{}", node_config.username, node_config.ip);
            let command = "docker swarm leave --force";

            let mut cmd = Command::new("ssh");
            cmd.arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg(&target)
                .arg(format!("{}", command));

            match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
                Ok(Some(output)) => {
                    if output.status.success() {
                        println!("{} left the cluster", target);
                    } else {
                        println!(
                            "{} failed to leave the cluster : {}",
                            target,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for execution of {} on : {}", command, target),
                Err(e) => println!("Execution of {} failed on {} : {}", command, target, e),
            }
        }
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

pub fn deploy_services() {
    let mut cmd = Command::new("docker");
    cmd.arg("stack")
        .arg("deploy")
        .arg("-c")
        .arg("config.yaml")
        .arg("server");

    match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
        Ok(Some(output)) => {
            if output.status.success() {
                println!("Services deployed.");
            } else {
                println!(
                    "Failed to deploy services : {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Ok(None) => {
            println!("Timeout for execution of 'docker stack deploy -c config.yaml server' ")
        }
        Err(e) => println!(
            "Execution of 'docker stack deploy -c config.yaml server' failed: {}",
            e
        ),
    }
}
