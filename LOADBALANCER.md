# K3s LoadBalancer and Load Balancing Guide

## Overview

K3s includes **ServiceLB** (formerly known as Klipper LoadBalancer), a lightweight load balancer that automatically exposes services on all nodes. This guide explains how it works and how to configure round-robin load balancing.

## How K3s LoadBalancer Works

### Architecture

K3s LoadBalancer is fundamentally different from cloud provider load balancers (AWS, GCP, Azure). Instead of a centralized load balancer, K3s uses **local load balancing** on each node:

1. When you create a `Service` with `type: LoadBalancer`, K3s creates a DaemonSet called `svclb-<service-name>`
2. This DaemonSet runs a small load balancer pod on **every node** in the cluster
3. Each node's load balancer listens on the service's external port
4. Traffic hitting any node is distributed to the service's backend pods

### Service Types in K3s

```
┌─────────────────────────────────────────────────────────────┐
│ Service Types                                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ClusterIP (default)                                         │
│ ├─ Only accessible within cluster                           │
│ ├─ Internal DNS: nginx-test.default.svc.cluster.local      │
│ └─ No external access                                       │
│                                                             │
│ NodePort                                                    │
│ ├─ Exposes service on all nodes at a high port (30000+)    │
│ ├─ Accessible via <node-ip>:<node-port>                    │
│ └─ Used for external access without LoadBalancer           │
│                                                             │
│ LoadBalancer (uses ServiceLB)                               │
│ ├─ Exposes service on all nodes at standard ports (80, 443)│
│ ├─ Accessible via <node-ip>:<port>                         │
│ ├─ Creates DaemonSet load balancer pods on all nodes       │
│ └─ Best for production external access                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Current Setup Analysis

In your `nginx-test-deployment.yaml`:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nginx-test
spec:
  type: ClusterIP    # ← Internal only, no external load balancing
  ports:
  - port: 80
    targetPort: 80
  selector:
    app: nginx-test
```

**Current flow:**
1. Request comes to Traefik (Ingress controller)
2. Traefik forwards to ClusterIP service `nginx-test`
3. Kubernetes distributes to backend pods using **round-robin by default**

**Note:** You're already using round-robin! Kubernetes services default to round-robin load balancing at the service level.

## Load Balancing Methods in Kubernetes

### 1. Service-Level Load Balancing (Default: Round-Robin)

**How it works:**
- When a pod connects to a service, the endpoint controller maintains a list of all healthy pods
- Kubernetes distributes connections **round-robin** across backend pods
- This is handled by kube-proxy in iptables/IPVS mode

**Verify current method:**
```bash
# Check kube-proxy mode (usually iptables or ipvs)
kubectl get daemonset -n kube-system kube-proxy -o yaml | grep -A 5 "mode:"

# Check available endpoints
kubectl get endpoints nginx-test
```

### 2. Traefik-Level Load Balancing

Since your traffic goes through Traefik (Ingress), Traefik also applies load balancing.

**Traefik load balancing strategy:**
- Traefik uses round-robin by default
- Distributes across all service endpoints
- Can be configured with Middleware for different strategies

## Changing Load Balancing Strategy

### Option 1: Configure Traefik Load Balancer (Recommended)

Add a Middleware to explicitly configure round-robin with different settings:

```yaml
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: sticky-sessions
  namespace: default
spec:
  sticky:
    cookie:
      name: sticky
      secure: true
      sameSite: lax
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: nginx-test
  annotations:
    traefik.ingress.kubernetes.io/router.middlewares: default-sticky-sessions@kubernetescrd
spec:
  ingressClassName: traefik
  rules:
  - host: nginx-test.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: nginx-test
            port:
              number: 80
```

### Option 2: Use LoadBalancer Service Type

Replace ClusterIP with LoadBalancer:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nginx-test
  namespace: default
spec:
  type: LoadBalancer              # ← Changed from ClusterIP
  sessionAffinity: None           # Round-robin (default)
  # sessionAffinity: ClientIP    # Sticky sessions (same client → same pod)
  ports:
  - port: 80
    targetPort: 80
    protocol: TCP
    name: http
  selector:
    app: nginx-test
```

**sessionAffinity options:**
- `None` - Pure round-robin, each new request goes to next pod
- `ClientIP` - Sticky: same client IP always routes to same pod
- `ClientIP` with `sessionAffinityConfig` - Sticky with timeout

### Option 3: Verify Current Round-Robin

Test that round-robin is working:

```bash
# Get service endpoints
kubectl get endpoints nginx-test
# Shows all pod IPs that nginx-test routes to

# Test round-robin with multiple requests
for i in {1..10}; do
  curl -s -H "Host: nginx-test.local" http://192.168.30.101 | grep "pod-name"
done

# You should see different pods in the response (showing round-robin is active)
```

## Practical Configuration for Round-Robin

### Change Current Service to Explicit Round-Robin

Edit your deployment:

```bash
kubectl patch service nginx-test -p '{"spec":{"sessionAffinity":"None"}}'
```

Or update the YAML:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nginx-test
  namespace: default
  labels:
    app: nginx-test
spec:
  type: LoadBalancer              # Enable external load balancing
  sessionAffinity: None           # Explicit round-robin
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 0           # No timeout (stateless)
  ports:
  - port: 80
    targetPort: 80
    protocol: TCP
    name: http
  selector:
    app: nginx-test
```

### Enable Sticky Sessions (Optional)

If you want the same client to always reach the same pod:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nginx-test
  namespace: default
spec:
  type: LoadBalancer
  sessionAffinity: ClientIP       # Sticky sessions
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800       # 3 hours timeout
  ports:
  - port: 80
    targetPort: 80
  selector:
    app: nginx-test
```

## Monitoring Load Distribution

### Check Current Load Balancing

```bash
# Watch which pods are receiving traffic
kubectl get pods -l app=nginx-test -w

# Check logs to see traffic distribution
kubectl logs -f -l app=nginx-test --all-containers=true | grep -E "GET|POST"

# See traffic pattern with timestamps
for i in {1..20}; do
  curl -s -H "Host: nginx-test.local" http://192.168.30.101 | \
    grep -o "nginx-test-[^<]*" | head -1
  echo ""
done
```

### Metrics from Prometheus

If you have Prometheus installed:

```promql
# Requests per pod
sum(rate(nginx_requests_total[1m])) by (pod)

# Pod latency
histogram_quantile(0.95, rate(nginx_request_duration_seconds_bucket[5m])) by (pod)
```

## ServiceLB Details

### View ServiceLB DaemonSet

```bash
# When you create a LoadBalancer service, K3s creates:
kubectl get daemonset -n kube-system | grep svclb

# Example output:
# NAME                                 DESIRED   CURRENT   READY   ...
# svclb-nginx-test-xxxxx              4         4         4       ...

# This means the load balancer pod is running on all 4 nodes
```

### ServiceLB Pod Details

```bash
# Check ServiceLB pods
kubectl get pods -n kube-system -l app=svclb

# Check which node runs the load balancer
kubectl get pods -n kube-system -l app=svclb -o wide

# View load balancer logs
kubectl logs -n kube-system -l app=svclb
```

## Comparison: Current vs LoadBalancer Service

### Current Setup (ClusterIP + Traefik Ingress)

```
External Request (http://192.168.30.101)
  ↓
Traefik Pod (runs on one node)
  ↓
ClusterIP Service (internal DNS resolution)
  ↓
Load balancing: Round-robin at kube-proxy level
  ↓
Backend Pod (any of 15 nginx pods)
```

**Pros:**
- Simple single ingress controller
- Works well for HTTP/HTTPS
- Traefik handles SSL/TLS termination

**Cons:**
- Traffic must go through Traefik first
- Traefik becomes a potential bottleneck

### LoadBalancer Service

```
External Request (http://192.168.30.101:80)
  ↓
ServiceLB Pod (runs on every node)
  ↓
Load balancing: Round-robin at each node
  ↓
Backend Pod (any of 15 nginx pods)
```

**Pros:**
- Distributed load balancing (one per node)
- Lower latency (no Traefik hop)
- Direct service access

**Cons:**
- Requires NodePort-like external access
- Different pod on each node adds resources
- No SSL/TLS termination (unless added)

## Recommended Configuration for Production

### For HTTP Services (using Traefik)

Keep your current setup - it's optimal for HTTP/HTTPS with ingress.

```bash
# Your current setup is already using round-robin!
# To verify:
kubectl get service nginx-test -o yaml | grep sessionAffinity
# Output: sessionAffinity: None  (= round-robin)
```

### For TCP/UDP Services

Use LoadBalancer type:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-tcp-service
spec:
  type: LoadBalancer
  sessionAffinity: None  # Round-robin
  ports:
  - port: 5000
    targetPort: 5000
    protocol: TCP
  selector:
    app: my-tcp-app
```

## Testing Round-Robin

### Test Script

```bash
#!/bin/bash

# Function to get pod name from response
get_pod_name() {
  curl -s -H "Host: nginx-test.local" http://192.168.30.101 | \
    grep -o "nginx-test-[^<]*" | head -1
}

# Send 30 requests and count which pods receive them
declare -A pod_counts

for i in {1..30}; do
  pod=$(get_pod_name)
  ((pod_counts[$pod]++))
  echo "Request $i: $pod"
done

# Print distribution
echo ""
echo "Distribution Summary:"
for pod in "${!pod_counts[@]}"; do
  echo "  $pod: ${pod_counts[$pod]} requests"
done
```

### Expected Output (Perfect Round-Robin)

If you have 15 replicas and send 30 requests:
```
nginx-test-abc: 2 requests
nginx-test-def: 2 requests
nginx-test-ghi: 2 requests
... (each pod gets ~2 requests)
```

## Quick Reference

### Check Current Load Balancing Strategy

```bash
kubectl get service nginx-test -o jsonpath='{.spec.sessionAffinity}'
# Output: None (round-robin) or ClientIP (sticky)
```

### Change to Round-Robin

```bash
kubectl patch service nginx-test -p '{"spec":{"sessionAffinity":"None"}}'
```

### Change to Sticky Sessions

```bash
kubectl patch service nginx-test -p '{"spec":{"sessionAffinity":"ClientIP","sessionAffinityConfig":{"clientIP":{"timeoutSeconds":10800}}}}'
```

### Switch Service Type

```bash
# From ClusterIP to LoadBalancer
kubectl patch service nginx-test -p '{"spec":{"type":"LoadBalancer"}}'

# Back to ClusterIP
kubectl patch service nginx-test -p '{"spec":{"type":"ClusterIP"}}'
```

## Summary

Your current setup already uses **round-robin load balancing** at multiple levels:

1. **Service Level** - Kubernetes kube-proxy distributes to pods round-robin
2. **Traefik Level** - Traefik also uses round-robin across endpoints
3. **This is the recommended configuration** for HTTP/HTTPS services

If you want more direct control or non-HTTP protocols, switch to LoadBalancer type, but for your nginx-test deployment, you're already optimally configured!
