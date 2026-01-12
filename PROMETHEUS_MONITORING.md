# Prometheus Operator & Monitoring Guide

Complete guide for deploying and managing monitoring infrastructure with Prometheus Operator, Grafana, and AlertManager in your k3s-ansible cluster.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Configuration](#configuration)
- [Accessing Components](#accessing-components)
- [Monitoring compute-blade-agent](#monitoring-compute-blade-agent)
- [Custom ServiceMonitors](#custom-servicemonitors)
- [Alerting](#alerting)
- [Grafana Dashboards](#grafana-dashboards)
- [Troubleshooting](#troubleshooting)

## Overview

The Prometheus Operator installation includes:

- **Prometheus**: Time-series database and scraping engine
- **Grafana**: Visualization and dashboarding platform
- **AlertManager**: Alert routing and management
- **Node Exporter**: Hardware and OS metrics
- **kube-state-metrics**: Kubernetes cluster metrics
- **Prometheus Operator**: CRD controller for managing Prometheus resources

### Architecture

```
┌─────────────────────────────────────────┐
│         Prometheus Operator             │
│  (CRD Controller - monitoring namespace)│
└─────────────────────────────────────────┘
                    ↓
    ┌───────────────┼───────────────┐
    ↓               ↓               ↓
Prometheus      Grafana        AlertManager
  (9090)         (3000)           (9093)
    ↓               ↓               ↓
    └───────────────┼───────────────┘
                    ↓
        ┌───────────────────────┐
        │  ServiceMonitors      │
        │  PrometheusRules      │
        │  AlertingRules        │
        └───────────────────────┘
                    ↓
    ┌───────────────┼───────────────┐
    ↓               ↓               ↓
 Scrape         Scrape          Scrape
Targets      Targets         Targets
```

## Quick Start

### Deploy Everything

```bash
# 1. Enable Prometheus Operator in inventory
# Edit inventory/hosts.ini
[k3s_cluster:vars]
enable_prometheus_operator=true
enable_compute_blade_agent=true

# 2. Run Ansible playbook
ansible-playbook site.yml --tags prometheus-operator

# 3. Wait for components to be ready
kubectl wait --for=condition=ready pod \
  -l app.kubernetes.io/name=prometheus \
  -n monitoring --timeout=300s

# 4. Access Prometheus
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090

# 5. Access Grafana
kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80
```

Then open:
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (default: admin/admin)
- AlertManager: http://localhost:9093

### Disable Prometheus Operator

```bash
# Edit inventory/hosts.ini
[k3s_cluster:vars]
enable_prometheus_operator=false

# Re-run playbook (Prometheus stack won't be installed)
ansible-playbook site.yml --tags prometheus-operator
```

## Installation

### Prerequisites

- K3s cluster already deployed with k3s-ansible
- kubectl access to the cluster
- Helm 3.x installed on the control machine

### Step-by-Step Installation

#### 1. Configure Inventory

Edit `inventory/hosts.ini`:

```ini
[k3s_cluster:vars]
# Enable Prometheus Operator
enable_prometheus_operator=true

# (Optional) Set Grafana admin password
grafana_admin_password=SecurePassword123!

# Enable compute-blade-agent monitoring
enable_compute_blade_agent=true
```

#### 2. Run the Playbook

```bash
# Install only Prometheus Operator
ansible-playbook site.yml --tags prometheus-operator

# Or deploy everything including K3s
ansible-playbook site.yml
```

#### 3. Verify Installation

```bash
# Check if monitoring namespace exists
kubectl get namespace monitoring

# Check Prometheus Operator deployment
kubectl get deployment -n monitoring

# Check all monitoring components
kubectl get all -n monitoring

# Wait for all pods to be ready
kubectl wait --for=condition=ready pod --all -n monitoring --timeout=300s
```

Expected output:

```
NAME                                                    READY   STATUS    RESTARTS   AGE
pod/prometheus-operator-5f8d4b5c7d-x9k2l               1/1     Running   0          2m
pod/prometheus-kube-prometheus-prometheus-0            2/2     Running   0          1m
pod/prometheus-kube-state-metrics-7c9d5f8c4-m2k9n     1/1     Running   0          2m
pod/prometheus-node-exporter-pz8kl                     1/1     Running   0          2m
pod/prometheus-grafana-5f8d7b5c9e-z1q3x                3/3     Running   0          1m
pod/prometheus-kube-alertmanager-0                     2/2     Running   0          1m
```

## Configuration

### Environment Variables

Configure via `inventory/hosts.ini`:

```ini
[k3s_cluster:vars]
# Enable/disable monitoring stack
enable_prometheus_operator=true

# Grafana configuration
grafana_admin_password=SecurePassword123!
grafana_admin_user=admin
grafana_storage_size=5Gi

# Prometheus configuration
prometheus_retention_days=7
prometheus_storage_size=10Gi
prometheus_scrape_interval=30s
prometheus_scrape_timeout=10s

# AlertManager configuration
alertmanager_storage_size=5Gi

# Component flags
enable_grafana=true
enable_alertmanager=true
enable_prometheus_node_exporter=true
enable_kube_state_metrics=true
```

### Per-Node Configuration

To restrict Prometheus to specific nodes:

```ini
[k3s_cluster:vars]
prometheus_node_selector={"node-type": "monitoring"}
```

Or via inventory host vars:

```ini
[master]
cm4-01 ansible_host=192.168.30.101 ansible_user=pi enable_prometheus_operator=true
cm4-02 ansible_host=192.168.30.102 ansible_user=pi enable_prometheus_operator=false
```

### Resource Limits

Control resource usage in `inventory/hosts.ini`:

```ini
prometheus_cpu_request=250m
prometheus_cpu_limit=500m
prometheus_memory_request=512Mi
prometheus_memory_limit=1Gi

grafana_cpu_request=100m
grafana_cpu_limit=200m
grafana_memory_request=256Mi
grafana_memory_limit=512Mi
```

## Accessing Components

### Prometheus Web UI

```bash
# Port-forward to localhost
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090

# Access at: http://localhost:9090
```

**Available from within cluster:**
```
http://prometheus-kube-prometheus-prometheus.monitoring.svc.cluster.local:9090
```

**Features:**
- Query builder
- Target health status
- Alert rules
- Service discovery
- Graph visualization

### Grafana Dashboards

```bash
# Port-forward to localhost
kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80

# Access at: http://localhost:3000
# Default credentials: admin / admin (or your configured password)
```

**Available from within cluster:**
```
http://prometheus-grafana.monitoring.svc.cluster.local:80
```

**Pre-installed Dashboards:**
1. Kubernetes / Cluster Monitoring
2. Kubernetes / Nodes
3. Kubernetes / Pods
4. Kubernetes / Deployments Statefulsets Daemonsets
5. Node Exporter for Prometheus Dashboard

**Custom Dashboards:**
- Import from grafana.com
- Create custom dashboards
- Connect to Prometheus data source

### AlertManager

```bash
# Port-forward to localhost
kubectl port-forward -n monitoring svc/prometheus-kube-alertmanager 9093:9093

# Access at: http://localhost:9093
```

**Available from within cluster:**
```
http://prometheus-kube-alertmanager.monitoring.svc.cluster.local:9093
```

**Features:**
- Alert grouping and deduplication
- Alert routing rules
- Notification management
- Silence alerts

### Verify Network Connectivity

```bash
# Test from within the cluster
kubectl run debug --image=busybox -it --rm -- sh

# Inside the pod:
wget -O- http://prometheus-kube-prometheus-prometheus.monitoring.svc:9090/-/healthy
wget -O- http://prometheus-grafana.monitoring.svc:80/api/health
wget -O- http://prometheus-kube-alertmanager.monitoring.svc:9093/-/healthy
```

## Monitoring compute-blade-agent

### Automatic Integration

When both are enabled, the Ansible role automatically:

1. Creates the `compute-blade-agent` namespace
2. Deploys ServiceMonitor for metrics scraping
3. Deploys PrometheusRule for alerting
4. Configures Prometheus scrape targets

### Verify compute-blade-agent Monitoring

```bash
# Check if ServiceMonitor is created
kubectl get servicemonitor -n compute-blade-agent

# Check if metrics are being scraped
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090

# Then in Prometheus UI:
# 1. Go to Status → Targets
# 2. Look for "compute-blade-agent" targets
# 3. Should show "UP" status
```

### Available Metrics

```
# Temperature monitoring
compute_blade_temperature_celsius

# Fan monitoring
compute_blade_fan_rpm
compute_blade_fan_speed_percent

# Power monitoring
compute_blade_power_watts

# Status indicators
compute_blade_status
compute_blade_led_state
```

### Create Custom Dashboard for compute-blade-agent

In Grafana:

1. Create new dashboard
2. Add panel with query:
   ```
   compute_blade_temperature_celsius{job="compute-blade-agent"}
   ```
3. Set visualization type to "Gauge" or "Graph"
4. Save dashboard

## Custom ServiceMonitors

### Create a ServiceMonitor

Create `custom-servicemonitor.yml`:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: my-app-monitoring
  namespace: my-app
  labels:
    app: my-app
    release: prometheus
spec:
  selector:
    matchLabels:
      app: my-app
  endpoints:
    - port: metrics
      interval: 30s
      path: /metrics
      scrapeTimeout: 10s
```

Deploy it:

```bash
kubectl apply -f custom-servicemonitor.yml
```

### Create a PrometheusRule

Create `custom-alerts.yml`:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: my-app-alerts
  namespace: my-app
  labels:
    prometheus: kube-prometheus
spec:
  groups:
    - name: my-app.rules
      interval: 30s
      rules:
        - alert: MyAppHighErrorRate
          expr: |
            rate(my_app_errors_total[5m]) > 0.05
          for: 5m
          labels:
            severity: warning
            app: my-app
          annotations:
            summary: "High error rate in my-app"
            description: "Error rate is {{ $value }} on {{ $labels.instance }}"

        - alert: MyAppDown
          expr: |
            up{job="my-app"} == 0
          for: 5m
          labels:
            severity: critical
            app: my-app
          annotations:
            summary: "my-app is down"
            description: "my-app on {{ $labels.instance }} is unreachable"
```

Deploy it:

```bash
kubectl apply -f custom-alerts.yml
```

## Alerting

### Pre-configured Alerts for compute-blade-agent

Automatically deployed when compute-blade-agent monitoring is enabled:

1. **ComputeBladeAgentHighTemperature** (Warning)
   - Triggers when temp > 80°C for 5 minutes
   
2. **ComputeBladeAgentCriticalTemperature** (Critical)
   - Triggers when temp > 95°C for 2 minutes
   
3. **ComputeBladeAgentDown** (Critical)
   - Triggers when agent unreachable for 5 minutes
   
4. **ComputeBladeAgentFanFailure** (Warning)
   - Triggers when fan RPM = 0 for 5 minutes
   
5. **ComputeBladeAgentHighFanSpeed** (Warning)
   - Triggers when fan speed > 90% for 10 minutes

### View Active Alerts

```bash
# In Prometheus UI:
# 1. Go to Alerts
# 2. See all active and pending alerts

# Or query:
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090
# Then visit: http://localhost:9090/alerts
```

### Configure AlertManager Routing

Edit AlertManager configuration:

```bash
# Get current config
kubectl get secret -n monitoring alertmanager-kube-prometheus-alertmanager -o jsonpath='{.data.alertmanager\.yaml}' | base64 -d

# Edit configuration
kubectl edit secret -n monitoring alertmanager-kube-prometheus-alertmanager
```

Example routing configuration:

```yaml
global:
  resolve_timeout: 5m

route:
  receiver: 'default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  
  routes:
    - match:
        severity: critical
      receiver: 'critical'
      group_wait: 0s
      group_interval: 1m
      repeat_interval: 30m
      
    - match:
        severity: warning
      receiver: 'default'
      group_wait: 1m

receivers:
  - name: 'default'
    # Add email, slack, webhook, etc.
    
  - name: 'critical'
    # Add urgent notifications
```

## Grafana Dashboards

### Import Pre-built Dashboards

1. Open Grafana: http://localhost:3000
2. Click "+" → Import
3. Enter dashboard ID from grafana.com
4. Select Prometheus data source
5. Click Import

**Recommended Dashboards:**

| ID | Name |
|----|------|
| 1860 | Node Exporter for Prometheus Dashboard |
| 6417 | Kubernetes Cluster Monitoring |
| 8588 | Kubernetes Deployment Statefulset Daemonset |
| 11074 | Node Exporter - Nodes |
| 12114 | Kubernetes cluster monitoring |

### Create Custom Dashboard

1. Click "+" → Dashboard
2. Click "Add new panel"
3. Configure query:
   - Data source: Prometheus
   - Query: `up{job="compute-blade-agent"}`
4. Set visualization (Graph, Gauge, Table, etc.)
5. Click Save

### Export Dashboard

```bash
# Get dashboard JSON
curl http://admin:password@localhost:3000/api/dashboards/db/my-dashboard > my-dashboard.json

# Import elsewhere
curl -X POST -H "Content-Type: application/json" \
  -d @my-dashboard.json \
  http://admin:password@localhost:3000/api/dashboards/db
```

## Troubleshooting

### Prometheus Not Scraping Targets

```bash
# Check Prometheus targets
kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090

# Visit: http://localhost:9090/targets
# Look for failed targets

# Check ServiceMonitor
kubectl get servicemonitor --all-namespaces

# Check Prometheus config
kubectl get prometheus -n monitoring -o yaml

# View Prometheus logs
kubectl logs -n monitoring -l app.kubernetes.io/name=prometheus --tail=50 -f
```

### Grafana Data Source Not Working

```bash
# Check Prometheus connectivity from Grafana
kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80

# In Grafana:
# 1. Configuration → Data Sources
# 2. Click Prometheus
# 3. Check Status (green = working)
# 4. If red, check URL and credentials

# Or check logs
kubectl logs -n monitoring -l app.kubernetes.io/name=grafana --tail=50 -f
```

### AlertManager Not Sending Notifications

```bash
# Check AlertManager configuration
kubectl get secret -n monitoring alertmanager-kube-prometheus-alertmanager -o jsonpath='{.data.alertmanager\.yaml}' | base64 -d

# Restart AlertManager to apply changes
kubectl rollout restart statefulset -n monitoring prometheus-kube-alertmanager

# Check logs
kubectl logs -n monitoring -l app.kubernetes.io/name=alertmanager --tail=50 -f

# Test webhook
kubectl port-forward -n monitoring svc/prometheus-kube-alertmanager 9093:9093
# Visit: http://localhost:9093 to see alerts
```

### Disk Space Issues

```bash
# Check Prometheus PVC usage
kubectl get pvc -n monitoring

# View disk usage
kubectl exec -n monitoring prometheus-kube-prometheus-prometheus-0 -- df -h /prometheus

# Increase storage
kubectl patch pvc -n monitoring prometheus-kube-prometheus-prometheus-db-prometheus-kube-prometheus-prometheus-0 \
  -p '{"spec":{"resources":{"requests":{"storage":"20Gi"}}}}'
```

### High Memory/CPU Usage

```bash
# Check resource usage
kubectl top pod -n monitoring

# Reduce retention period
kubectl edit prometheus -n monitoring

# Update in spec:
# retention: 3d  # Reduce from 7d to 3d

# Or reduce scrape interval
# serviceMonitorSelectorNilUsesHelmValues: false
# scrapeInterval: 60s  # Increase from 30s to 60s
```

### ServiceMonitor Not Being Picked Up

```bash
# Check if labels match
kubectl get servicemonitor --all-namespaces -o yaml | grep -A5 "release: prometheus"

# Prometheus selector config
kubectl get prometheus -n monitoring -o yaml | grep -A5 "serviceMonitorSelector"

# Restart Prometheus if config changed
kubectl rollout restart statefulset -n monitoring prometheus-kube-prometheus-prometheus

# Check Prometheus logs for errors
kubectl logs -n monitoring prometheus-kube-prometheus-prometheus-0 -c prometheus --tail=100 | grep -i "error\|failed"
```

## Maintenance

### Backup Grafana Dashboards

```bash
# Export all dashboards
for dashboard in $(curl -s http://admin:password@localhost:3000/api/search | jq -r '.[] | .uri'); do
  name=$(echo $dashboard | cut -d'/' -f2)
  curl -s http://admin:password@localhost:3000/api/dashboards/$dashboard > ${name}.json
done

# Or use backup tool
kubectl exec -n monitoring prometheus-grafana-0 -- grafana-cli admin export-dashboard all ./backups
```

### Update Prometheus Retention

```bash
# Edit Prometheus resource
kubectl edit prometheus -n monitoring

# Update retention field
spec:
  retention: "30d"  # Change from 7d to 30d

# Changes apply automatically
```

### Scale Prometheus Resources

```bash
# For high-load environments, increase resources
kubectl patch prometheus -n monitoring kube-prometheus -p '{"spec":{"resources":{"requests":{"cpu":"500m","memory":"1Gi"},"limits":{"cpu":"1000m","memory":"2Gi"}}}}'
```

## Best Practices

1. **Security**
   - Change default Grafana password immediately
   - Restrict AlertManager webhook URLs to known services
   - Use network policies to limit access

2. **Performance**
   - Monitor Prometheus disk usage regularly
   - Adjust scrape intervals based on needs
   - Use recording rules for complex queries

3. **Reliability**
   - Enable persistent storage (PVC)
   - Configure alert routing and escalation
   - Regular backup of Grafana dashboards and configs

4. **Organization**
   - Label all ServiceMonitors with `release: prometheus`
   - Use consistent naming conventions
   - Document custom dashboards and alerts

5. **Cost Optimization**
   - Remove unused scrape targets
   - Tune scrape intervals (don't scrape more than needed)
   - Set appropriate retention periods

## Support

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Prometheus Operator GitHub](https://github.com/prometheus-operator/prometheus-operator)
- [Grafana Documentation](https://grafana.com/docs/)
- [AlertManager Documentation](https://prometheus.io/docs/alerting/latest/alertmanager/)
