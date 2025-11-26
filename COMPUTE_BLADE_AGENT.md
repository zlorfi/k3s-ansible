# Compute Blade Agent Deployment Guide

Quick reference for deploying and managing the Compute Blade Agent in your k3s-ansible cluster.

## Quick Start

### Deploy Everything

```bash
ansible-playbook site.yml
```

### Deploy Only Compute Blade Agent

```bash
ansible-playbook site.yml --tags compute-blade-agent
```

### Skip Compute Blade Agent

```bash
ansible-playbook site.yml --skip-tags compute-blade-agent
```

## Configuration

### Enable/Disable Globally

Edit `inventory/hosts.ini`:

```ini
[k3s_cluster:vars]
enable_compute_blade_agent=true    # Set to false to disable
```

### Enable/Disable Per-Node

Edit `inventory/hosts.ini`:

```ini
[worker]
cm4-02 ansible_host=192.168.30.102 ansible_user=pi enable_compute_blade_agent=true
cm4-03 ansible_host=192.168.30.103 ansible_user=pi enable_compute_blade_agent=false
cm4-04 ansible_host=192.168.30.104 ansible_user=pi
```

## Verification

### Check Service Status

```bash
ssh pi@<worker-ip>
sudo systemctl status compute-blade-agent
```

### View Logs

```bash
ssh pi@<worker-ip>
sudo journalctl -u compute-blade-agent -f
```

### Check Installation

```bash
ssh pi@<worker-ip>
/usr/local/bin/compute-blade-agent --version
ls -la /etc/compute-blade-agent/
```

## File Locations

- **Binary**: `/usr/local/bin/compute-blade-agent`
- **Config**: `/etc/compute-blade-agent/config.yaml`
- **Systemd Service**: `/etc/systemd/system/compute-blade-agent.service`
- **Logs**: `journalctl -u compute-blade-agent`

## Environment Variables

Configure via `BLADE_` prefixed environment variables:

```bash
export BLADE_CONFIG_PATH=/etc/compute-blade-agent/config.yaml
/usr/local/bin/compute-blade-agent
```

## Monitoring

### Optional Kubernetes Resources

Deploy monitoring components:

```bash
kubectl apply -f manifests/compute-blade-agent-daemonset.yaml
```

This creates:

- Namespace: `compute-blade-agent`
- ConfigMap: `compute-blade-agent-config`
- DaemonSet: `compute-blade-agent-exporter`
- Service: `compute-blade-agent-metrics`

### Prometheus Integration

To enable Prometheus scraping, uncomment the ServiceMonitor in the manifest:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: compute-blade-agent
  namespace: compute-blade-agent
spec:
  selector:
    matchLabels:
      app: compute-blade-agent
  endpoints:
    - port: metrics
      interval: 30s
      path: /metrics
```

## Troubleshooting

### Service Not Running

```bash
# Check service status
sudo systemctl status compute-blade-agent

# View error logs
sudo journalctl -u compute-blade-agent -n 50

# Try restarting
sudo systemctl restart compute-blade-agent
```

### Hardware Not Detected

1. Verify hardware is connected
2. Check logs for hardware errors
3. Ensure systemd service started correctly

### Reinstall

```bash
# SSH to node
ssh pi@<worker-ip>

# Check if uninstall script exists
ls -la /usr/local/bin/*compute-blade*

# Run uninstall if it exists
sudo bash /usr/local/bin/k3s-uninstall-compute-blade-agent.sh 2>/dev/null || true

# Re-run Ansible
ansible-playbook site.yml --tags compute-blade-agent
```

## Uninstall

### From Single Node

```bash
ssh pi@<worker-ip>
sudo bash /usr/local/bin/k3s-uninstall-compute-blade-agent.sh
```

### From All Worker Nodes

```bash
ansible worker -m shell -a "bash /usr/local/bin/k3s-uninstall-compute-blade-agent.sh" --become
```

## Features

- **Hardware Monitoring**: Temperature, fan speed, button events
- **Automatic Scaling**: Critical mode on overheat (max fan + red LED)
- **Identification**: LED blinking for locating blades in racks
- **Metrics Export**: Prometheus-compatible metrics
- **CLI Tool**: `bladectl` for local/remote interaction

## Files in This Repository

- `roles/compute-blade-agent/tasks/main.yml` - Installation role
- `manifests/compute-blade-agent-daemonset.yaml` - Kubernetes monitoring resources
- `site.yml` - Playbook that runs the role
- `inventory/hosts.ini` - Configuration variables
- `README.md` - Full documentation

## Additional Resources

- [Compute Blade Agent Repository](https://github.com/compute-blade-community/compute-blade-agent)
- [Compute Blade Community](https://github.com/compute-blade-community)
