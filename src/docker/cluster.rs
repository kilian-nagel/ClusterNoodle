use crate::{ClusterConfig, docker};
use crate::utils::command;
use crate::utils::envVariables::EnvVariables;
use std::path;
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
    pub fn init(&mut self, ip_adress: &Option<String>) -> () {
        let env = EnvVariables {};
        let usable_ip_script_path = &env.get_usable_ip_script_path();

        let ip: String;

        if ip_adress.is_none() {
            let ip_output = Command::new("/bin/bash")
                .arg(usable_ip_script_path)
                .output()
                .expect("Failed to get IP address");

            ip = str::from_utf8(&ip_output.stdout)
                .expect("Invalid UTF-8 output")
                .trim()
                .to_string();

            if ip.is_empty() {
                eprintln!("Could not find a valid network interface (eth, wlp, or virbr)");
                std::process::exit(1);
            }
        } else {
            ip = ip_adress.clone().unwrap();
        }

        println!("Using IP: {}", ip);

        // Run docker swarm init with the detected IP
        let output = Command::new("docker")
            .arg("swarm")
            .arg("init")
            .arg("--advertise-addr")
            .arg(ip)
            .output()
            .expect("Failed to run docker swarm init");

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
                        println!("Docker installed on {}", node_config.ip);
                    } else {
                        println!(
                            "Install failed on {}: {}",
                            node_config.ip,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for {}", node_config.ip),
                Err(e) => println!("Execution failed on {}: {}", node_config.ip, e),
            }
        }
    }

    pub fn pull_docker_images(&self){
        for docker_image in &self.docker_images {
            let mut cmd = Command::new("docker");
            cmd.arg("pull");
            cmd.arg(docker_image);
            println!("pulling docker image : {}", docker_image);

            match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
                Ok(Some(output)) => {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if output.status.success() {
                        println!("Docker image pulled");
                    } else {
                        println!(
                            "Failed to pull following docker image : {}. Error : {}",
                            &docker_image,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout while pulling following docker image : {}", docker_image),
                Err(e) => println!("Failed to pull following docker image : {}. Error : {}", docker_image, e),
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
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if output.status.success() {
                        println!("{} joined the cluster", node_config.ip);
                    } else {
                        println!(
                            "{} failed to join the cluster : {}",
                            node_config.ip,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for execution of {} on : {}", command, node_config.ip),
                Err(e) => println!("Execution of {} failed on {} : {}", command, node_config.ip, e),
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
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if output.status.success() {
                        println!("{} left the cluster", node_config.ip);
                    } else {
                        println!(
                            "{} failed to leave the cluster : {}",
                            node_config.ip,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for execution of {} on : {}", command, node_config.ip),
                Err(e) => println!("Execution of {} failed on {} : {}", command, node_config.ip, e),
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

    println!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

pub fn deploy_services(docker_file_path_param: Option<&str>) {
    // Si un chemin a été renseigné on l'utilise sinon utilise celui par défaut.
    let env = EnvVariables {};
    let mut docker_file_path = String::from(env.get_docker_file_path());

    let mut cmd = Command::new("docker");
    cmd.arg("stack")
        .arg("deploy")
        .arg("-c")
        .arg(&docker_file_path)
        .arg("server");

    match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
        Ok(Some(output)) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
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
