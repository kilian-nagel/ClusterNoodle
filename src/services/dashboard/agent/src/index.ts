import express from "express";
import { Request, Response } from 'express';
import Docker from "dockerode";
import cors from "cors";

const app = express();
app.use(cors());

const docker = new Docker({ socketPath: "/var/run/docker.sock" });

app.get("/api/docker/health", async (_req: Request, res: Response) => {
  try {
    const info = await docker.info();
    res.json(info);
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
    const nodes = await docker.listNodes();
    res.json(nodes);
  } catch (error) {
    console.error("Error fetching nodes:", error);
    res.status(500).json({ error: "Failed to fetch nodes" });
  }
});

// Fetch all services
app.get("/api/docker/services", async (_req: Request, res: Response) => {
  try {
    const services = await docker.listServices();
    res.json(services);
  } catch (error) {
    console.error("Error fetching services:", error);
    res.status(500).json({ error: "Failed to fetch services" });
  }
});

// Health check
app.get("/api/health", (_req: Request, res: Response) => {
  res.json({ status: "ok" });
});

const PORT = 8090;
app.listen(8090, () => {
  console.log(`✅ Docker Swarm API server running on http://localhost:${PORT}`);
});