import express from "express"

import { Request, Response } from 'express';
import cors from "cors";

declare const process : {
  env: {
    DOCKER_SOCKET_AGENT_URL: string
    DOCKER_FRONTEND_URL: string
    PORT: number
  }
}

const app = express();
app.use(cors());


interface DockerNode {
  ID: string;
  Description?: {
    Hostname?: string;
  };
  Status?: {
    State?: string;
  };
}

interface DockerService {
  ID: string;
  Spec?: {
    Name?: string;
    Mode?: {
      Replicated?: {
        Replicas?: number;
      };
    };
  };
}

interface NodeInfo {
  id: string;
  hostname: string;
  status: string;
}

interface ServiceInfo {
  id: string;
  name: string;
  replicas: string;
}


app.get("/api/docker/health", async (_req: Request, res: Response) => {
  try {
    const url = process.env.DOCKER_SOCKET_AGENT_URL + "/api/docker/health";
    console.log(url);
    const health_data_raw = await fetch(url);
    console.log("health_data_raw : ", health_data_raw); 
    const health_data = await health_data_raw.json();
    console.log("health_data : ", health_data); 

    res.json({
      connected: true,
      swarmActive: health_data.Swarm?.LocalNodeState === 'active',
      swarmNodeId: health_data.Swarm?.NodeID || null,
      swarmManagers: health_data.Swarm?.Managers || 0,
      swarmNodes: health_data.Swarm?.Nodes || 0
    });
    res.status(200).json(health_data);
  }  catch (error: unknown) {
    console.error("Docker connection error:", error);
    const errorMessage = error instanceof Error ? error.message : String(error);
    res.status(500).json({ 
      connected: false, 
      error: errorMessage
    });
  }
});

// Fetch all nodes
app.get("/api/docker/nodes", async (_req: Request, res: Response) => {
  try {
    const nodes_data_raw = await fetch(process.env["DOCKER_SOCKET_AGENT_URL"] + "/api/docker/nodes");
    console.log("nodes_data_raw : ", nodes_data_raw); 
    const nodes_data = await nodes_data_raw.json();
    console.log("nodes_data", nodes_data);
    const nodes_formatted: NodeInfo[] = nodes_data.map((n: DockerNode) => ({
      id: n.ID,
      hostname: n.Description?.Hostname || "unknown",
      status: n.Status?.State || "unknown",
    }));
    console.log("nodes_data_formatted : ", nodes_formatted); 
    res.status(200).json(nodes_formatted);
  } catch (error) {
    console.error("Error fetching nodes:", error);
    res.status(500).json({ error: "Failed to fetch nodes" });
  }
});

// Fetch all services
app.get("/api/docker/services", async (_req: Request, res: Response) => {
  try {

    const services_data_raw = await fetch(process.env["DOCKER_SOCKET_AGENT_URL"] + "/api/docker/services");
    console.log("services_data_raw : ", services_data_raw); 

    const services = await services_data_raw.json();
    console.log("services : ", services); 

    const formatted_services: ServiceInfo[] = services.map((s: DockerService) => ({
      id: s.ID,
      name: s.Spec?.Name || "unknown",
      replicas:
        s.Spec?.Mode?.Replicated?.Replicas?.toString() ??
        "Global / Unknown",
    }));
    console.log("formatted_services : ", formatted_services);

    res.json(formatted_services);
  } catch (error) {
    console.error("Error fetching services:", error);
    res.status(500).json({ error: "Failed to fetch services" });
  }
});

// Health check
app.get("/api/health", (_req: Request, res: Response) => {
  res.json({ status: "ok" });
});

const PORT = process.env.PORT || 3001;
app.listen(PORT, () => {
  console.log(`✅ Docker Swarm API server running on http://localhost:${PORT}`);
});
