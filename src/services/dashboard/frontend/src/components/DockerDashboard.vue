<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import Clipboard from './Clipboard.vue';

interface NodeInfo {
  id: string
  hostname: string
  status: string
  role?: string
  availability?: string
}

interface ServiceInfo {
  id: string
  name: string
  replicas: string
  status?: string
}

const nodes = ref<NodeInfo[]>([])
const services = ref<ServiceInfo[]>([])
const loading = ref(false)
const lastUpdate = ref<Date | null>(null)
// Prefer runtime-configured BACKEND_URL injected by the container, fallback to build-time VITE_BACKEND_URL
const runtimeBackend = (window as any).__ENV?.BACKEND_URL || (window as any).__ENV?.BACKEND_URL
const backendUrl = runtimeBackend || import.meta.env.VITE_BACKEND_URL
console.log('Using backend URL:', backendUrl)

enum ClusterState {
  Unreachable = 'Unreachable',
  Unknown = 'Unknown',
  Healthy = 'Healthy',
  Degraded = 'Degraded',
}

const healthyNodes = computed(() => nodes.value.filter(n => n.status === 'ready').length)
const totalNodes = computed(() => nodes.value.length)
const clusterHealth = computed(() => {
  if(!isClusterUp.value) return ClusterState.Unreachable
  if (totalNodes.value === 0) return ClusterState.Unknown
  return healthyNodes.value === totalNodes.value ? ClusterState.Healthy : ClusterState.Degraded
})
const isClusterUp = ref<Boolean>(true);

async function fetchSwarmData() {
  if(lastUpdate.value !== null){
    loading.value = true
  }
  try {
    const [nodesRes, servicesRes] = await Promise.all([
      fetch(`${backendUrl}/api/docker/nodes`),
      fetch(`${backendUrl}/api/docker/services`),
    ])

    lastUpdate.value = new Date()

    if(nodesRes?.status === 500){
      nodes.value = [];
      services.value = [];
      isClusterUp.value = false;
      return;
    }

    nodes.value = await nodesRes.json()
    services.value = await servicesRes.json()
  } catch (err) {
    console.error('Error fetching swarm data:', err)

    nodes.value = [];
    services.value = [];
    isClusterUp.value = false;
    lastUpdate.value = new Date()
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchSwarmData()
  setInterval(fetchSwarmData, 10000)
})
</script>

<template>
  <div class="min-h-screen bg-black text-white p-6">
    <!-- Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2 text-balance text-white">ClusterNoodle Dashboard</h1>
      <div class="flex items-center gap-4 text-sm text-gray-400">
        <div class="flex items-center gap-2">
          <div 
            class="relative w-2 h-2 rounded-full"
            :class="clusterHealth === ClusterState.Healthy ? 'bg-emerald-500' : 'bg-amber-500'"
          >
            <div 
              class="absolute inset-0 rounded-full animate-ping"
              :class="clusterHealth === ClusterState.Healthy ? 'bg-emerald-500' : 'bg-amber-500'"
            />
          </div>
          <span class="capitalize">{{ clusterHealth }}</span>
        </div>
        <span>•</span>
        <span>{{ healthyNodes }}/{{ totalNodes }} nodes active</span>
        <span v-if="lastUpdate">•</span>
        <span v-if="lastUpdate">Updated {{ lastUpdate.toLocaleTimeString() }}</span>
      </div>
    </div>

    <!-- Added error banner when cluster is down -->
    <div v-if="!isClusterUp" class="mb-6">
      <Card class="bg-red-950 border-red-800">
        <CardContent class="pt-6">
          <div class="flex items-center gap-4">
            <div class="relative flex-shrink-0">
              <div class="w-4 h-4 rounded-full bg-red-500" />
              <div class="absolute inset-0 rounded-full bg-red-500 animate-ping" />
            </div>
            <div>
              <h3 class="text-lg font-semibold text-red-200 mb-1">Cluster Unreachable</h3>
              <p class="text-sm text-red-300 max-w-[920px]">Unable to connect to Docker cluster. Please make sure you started your cluster and that your user can access the docker socket. Before executing the following command make sure you undestand the security implications.
              </p>
            </div>
          </div>
      <Clipboard class="mt-4" text="sudo chown $USER:$USER /var/run/docker.sock"/>

        </CardContent>
      </Card>
    </div>

    <!-- Main Grid -->
    <div class="grid gap-6 lg:grid-cols-2" v-if="isClusterUp">
      <!-- Cluster Nodes Card -->
      <Card class="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle class="text-xl font-semibold text-white">Cluster Nodes</CardTitle>
            <span class="text-sm text-gray-400">{{ nodes.length }} total</span>
          </div>
        </CardHeader>
        <CardContent>
          <div v-if="loading && nodes.length === 0" class="text-center py-8 text-gray-400">
            <div class="inline-block w-6 h-6 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
            <p class="mt-2">Loading nodes...</p>
          </div>

          <div v-else class="space-y-3">
            <div
              v-for="node in nodes"
              :key="node.id"
              class="flex items-center justify-between p-4 bg-black border border-zinc-800 rounded-lg hover:border-zinc-700 transition-colors"
            >
              <div class="flex items-center gap-3">
                <div class="relative">
                  <div
                    class="w-3 h-3 rounded-full"
                    :class="node.status === 'ready' ? 'bg-emerald-500' : 'bg-red-500'"
                  />
                  <div
                    class="absolute inset-0 rounded-full animate-ping"
                    :class="node.status === 'ready' ? 'bg-emerald-500' : 'bg-red-500'"
                  />
                </div>
                <div>
                  <p class="font-medium text-white">{{ node.hostname }}</p>
                  <p class="text-xs text-gray-400" v-if="node.role">{{ node.role }}</p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <Badge
                  :variant="node.status === 'ready' ? 'default' : 'destructive'"
                  :class="node.status === 'ready' ? 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20' : 'bg-red-500/10 text-red-400 hover:bg-red-500/20'"
                >
                  {{ node.status }}
                </Badge>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Services Card -->
      <Card class="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle class="text-xl font-semibold text-white">Services</CardTitle>
            <span class="text-sm text-gray-400">{{ services.length }} running</span>
          </div>
        </CardHeader>
        <CardContent>
          <div v-if="loading && services.length === 0" class="text-center py-8 text-gray-400">
            <div class="inline-block w-6 h-6 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
            <p class="mt-2">Loading services...</p>
          </div>

          <div v-else class="space-y-3">
            <div
              v-for="service in services"
              :key="service.id"
              class="flex items-center justify-between p-4 bg-black border border-zinc-800 rounded-lg hover:border-zinc-700 transition-colors"
            >
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-zinc-800 flex items-center justify-center text-xs font-mono text-gray-400">
                  SVC
                </div>
                <div>
                  <p class="font-medium text-white">{{ service.name }}</p>
                  <p class="text-xs text-gray-400">{{ service.replicas }} replicas</p>
                </div>
              </div>
              <div
                class="w-2 h-2 rounded-full"
                :class="service.status === 'degraded' ? 'bg-amber-500' : 'bg-emerald-500'"
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Stats Overview -->
    <div class="mt-6 grid gap-6 md:grid-cols-3" v-if="isClusterUp">
      <Card class="bg-zinc-900 border-zinc-800">
        <CardContent class="pt-6">
          <div class="text-sm text-gray-400 mb-1">Total Nodes</div>
          <div class="text-3xl font-bold text-white">{{ totalNodes }}</div>
        </CardContent>
      </Card>
      <Card class="bg-zinc-900 border-zinc-800">
        <CardContent class="pt-6">
          <div class="text-sm text-gray-400 mb-1">Healthy Nodes</div>
          <div class="text-3xl font-bold text-emerald-500">{{ healthyNodes }}</div>
        </CardContent>
      </Card>
      <Card class="bg-zinc-900 border-zinc-800">
        <CardContent class="pt-6">
          <div class="text-sm text-gray-400 mb-1">Services</div>
          <div class="text-3xl font-bold text-white">{{ services.length }}</div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>

<style scoped>
@keyframes ping {
  75%,
  100% {
    transform: scale(2);
    opacity: 0;
  }
}

.animate-ping {
  animation: ping 1.5s cubic-bezier(0, 0, 0.2, 1) infinite;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
