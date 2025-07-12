use crate::config;
use std::io;
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

pub fn init_cluster() -> () {
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
        "You have to give servers informations. Exemple : 192.168.1.1,username,password 192.168.1.2,username2,password2"
    );
    let mut nodes_ips_input = String::new();

    io::stdin().read_line(&mut nodes_ips_input).unwrap();
    // On se débarasse du \n.
    nodes_ips_input.pop();

    return config::build_cluster_nodes_objects(&nodes_ips_input);
}
