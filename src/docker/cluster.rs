use crate::config;
use crate::config::ClusterConfig;
use crate::utils::command;
use std::io;
use std::process::Command;
use std::time::Duration;

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

pub fn init_cluster(config: &mut ClusterConfig) -> () {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("init")
        .output()
        .unwrap();

    let output_string = String::from_utf8_lossy(&output.stdout).to_string(); // owns the String

    let output_sentences: Vec<&str> = output_string.lines().collect();

    for sentence in output_sentences {
        if sentence.trim().starts_with("docker swarm join") {
            config.cluster_docker_command = sentence.trim().to_string();
        }
    }

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

pub fn ask_nodes_number() -> u16 {
    println!("How many nodes would like you deploy ?");
    let mut nodes_number_input = String::new();

    io::stdin().read_line(&mut nodes_number_input).unwrap();
    // On se débarasse du \n.
    nodes_number_input.pop();

    // Convertit l'input de l'utilisateur en entier.
    let nodes_number_input_result = nodes_number_input.parse::<u16>();
    if !nodes_number_input_result.is_ok() {
        panic!("Number of nodes has to be a number");
    }

    return nodes_number_input_result.unwrap();
}

pub fn ask_nodes_infos() -> Vec<config::NodeConfig> {
    println!(
        "Provide servers as: <ip>,<username>,<password>. Example: 192.168.1.1,username,password 192.168.1.2,username2,password2"
    );
    let mut nodes_ips_input = String::new();

    io::stdin().read_line(&mut nodes_ips_input).unwrap();
    // On se débarasse du \n.
    nodes_ips_input.pop();

    return config::build_cluster_nodes_objects(&nodes_ips_input);
}

pub fn install_docker(config: &config::ClusterConfig) {
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

pub fn join_cluster(config: &config::ClusterConfig) {
    for node_config in &config.nodes_configs {
        let target = format!("{}@{}", node_config.username, node_config.ip);
        let command = &config.cluster_docker_command;

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

pub fn leave_cluster(config: &config::ClusterConfig) {
    for node_config in &config.nodes_configs {
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
