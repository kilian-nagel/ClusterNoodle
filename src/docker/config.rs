pub struct NodeConfig {
    pub ip: String,
    pub username: String,
    pub password: String,
}

pub fn build_cluster_nodes_objects(input: &str) -> Vec<NodeConfig> {
    let nodes_ips = input.split(" ");
    let mut nodes_configs: Vec<NodeConfig> = vec![];

    for node_ip in nodes_ips {
        let data = node_ip.split(",").collect::<Vec<_>>();
        if data.len() > 2 {
            let node_config = NodeConfig {
                ip: data[0].to_string(),
                username: data[1].to_string(),
                password: data[2].to_string(),
            };
            nodes_configs.push(node_config);
        }
    }
    return nodes_configs;
}

pub struct ClusterConfig {
    pub nodes_number: u16,
    pub nodes_configs: Vec<NodeConfig>,
}
