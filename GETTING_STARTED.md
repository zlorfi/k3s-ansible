# Getting Started with Compute Blade Agent Deployment

This document provides a quick guide to deploy your k3s-ansible cluster with Compute Blade Agent support.

## Prerequisites

- Ansible installed on your control machine
- SSH access to all nodes configured
- Raspberry Pi CM4/CM5 modules with Raspberry Pi OS installed

## Quick Start (5 minutes)

### Step 1: Review Configuration

```bash
cat inventory/hosts.ini
```

Verify:

- Master nodes are correct (cm4-01, cm4-02, cm4-03)
- Worker node IP is correct (cm4-04)
- `enable_compute_blade_agent=true` is set (optional for masters)

### Step 2: Test Connectivity

```bash
ansible all -m ping
```

All nodes should respond with `pong`.

### Step 3: Deploy

```bash
ansible-playbook site.yml
```

This will:

1. Prepare all nodes (10-15 minutes)
2. Install K3s server on master (5 minutes)
3. Install K3s agents on workers (5 minutes)
4. Install compute-blade-agent on workers (2-3 minutes per node)
5. Deploy test application (1 minute)

**Total time**: ~30-45 minutes

### Step 4: Verify Cluster

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl get nodes
```

You should see all 4 nodes ready (3 masters + 1 worker):

```bash
NAME     STATUS   ROLES                       AGE   VERSION
cm4-01   Ready    control-plane,etcd,master   5m    v1.35.0+k3s1
cm4-02   Ready    control-plane,etcd          3m    v1.35.0+k3s1
cm4-03   Ready    control-plane,etcd          3m    v1.35.0+k3s1
cm4-04   Ready    <none>                      3m    v1.35.0+k3s1
```

## Configuration

### Enable/Disable Agent

To enable agent on all workers (default):

```ini
[k3s_cluster:vars]
enable_compute_blade_agent=true
```

To disable agent:

```ini
enable_compute_blade_agent=false
```

To enable/disable on specific nodes:

```ini
[worker]
cm4-02 ansible_host=192.168.30.102 ansible_user=pi enable_compute_blade_agent=true
cm4-03 ansible_host=192.168.30.103 ansible_user=pi enable_compute_blade_agent=false
cm4-04 ansible_host=192.168.30.104 ansible_user=pi
```

## Deployment Options

### Option 1: Full Deployment (Recommended for new clusters)

```bash
ansible-playbook site.yml
```

Deploys K3s + compute-blade-agent + test application

### Option 2: Skip Test Application (Faster)

```bash
ansible-playbook site.yml --skip-tags test
```

Useful if cluster already has applications

### Option 3: Agent Only (Existing K3s cluster)

```bash
ansible-playbook site.yml --tags compute-blade-agent
```

Deploy agent to existing K3s cluster

### Option 4: Skip Agent

```bash
ansible-playbook site.yml --skip-tags compute-blade-agent
```

Deploy K3s without agent

## Verification

### Check Agent Status

```bash
# From control machine
bash scripts/verify-compute-blade-agent.sh

# On a worker node
ssh pi@192.168.30.102
sudo systemctl status compute-blade-agent
```

### View Logs

```bash
ssh pi@192.168.30.102
sudo journalctl -u compute-blade-agent -f
```

Press `Ctrl+C` to exit logs.

### Check Binary

```bash
ssh pi@192.168.30.102
/usr/local/bin/compute-blade-agent --version
```

## What Was Installed

### On Each Worker Node

- **Binary**: `/usr/local/bin/compute-blade-agent`
- **CLI Tool**: `/usr/local/bin/bladectl`
- **Config**: `/etc/compute-blade-agent/config.yaml`
- **Service**: `compute-blade-agent.service` (auto-start)

### Features Enabled

- Hardware monitoring (temperature, fan speed, buttons)
- Critical mode protection (overheat response)
- LED identification (blade location)
- Prometheus metrics export
- Auto-restart on node reboot

## Troubleshooting

### Service Not Running

```bash
ssh pi@192.168.30.102
sudo systemctl status compute-blade-agent
sudo journalctl -u compute-blade-agent -n 50
```

### Re-run Deployment

```bash
ansible-playbook site.yml --tags compute-blade-agent
```

### Check Installation on Node

```bash
ssh pi@192.168.30.102
ls -la /usr/local/bin/compute-blade-agent
ls -la /etc/compute-blade-agent/
sudo systemctl status compute-blade-agent
```

## Next Steps

1. **Review Documentation**
   - `COMPUTE_BLADE_AGENT.md` - Quick reference
   - `DEPLOYMENT_CHECKLIST.md` - Detailed steps
   - `README.md` - Full guide

2. **Configure Monitoring** (Optional)

   ```bash
   kubectl apply -f manifests/compute-blade-agent-daemonset.yaml
   ```

3. **Access Cluster** (If deployed K3s)

   ```bash
   export KUBECONFIG=$(pwd)/kubeconfig
   kubectl get nodes
   ```

4. **Customize Configuration** (If needed)
   - Edit `inventory/hosts.ini` for deployment options
   - Edit `/etc/compute-blade-agent/config.yaml` on nodes

## Common Tasks

### Check Cluster Status

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl get nodes
kubectl get pods --all-namespaces
```

### Access Any Master Node

```bash
# Access cm4-01
ssh pi@192.168.30.101

# Or access cm4-02 (backup master)
ssh pi@192.168.30.102

# Or access cm4-03 (backup master)
ssh pi@192.168.30.103
```

### Deploy Only to Specific Nodes

```bash
ansible-playbook site.yml --tags compute-blade-agent --limit cm4-04
```

### Disable Agent for Next Deployment

```bash
# Edit inventory/hosts.ini
enable_compute_blade_agent=false

# Then run playbook again
ansible-playbook site.yml --tags compute-blade-agent
```

## Uninstall

### Uninstall Agent (All Workers)

```bash
ansible worker -m shell -a "bash /usr/local/bin/k3s-uninstall-compute-blade-agent.sh" --become
```

### Uninstall K3s (All Nodes)

```bash
ansible all -m shell -a "bash /usr/local/bin/k3s-uninstall.sh" --become
```

## Documentation

- **README.md** - Full guide with all configuration options
- **DEPLOYMENT_CHECKLIST.md** - Step-by-step checklist
- **COMPUTE_BLADE_AGENT.md** - Quick reference for agent deployment
- **MIKROTIK-VIP-SETUP-CUSTOM.md** - Virtual IP failover configuration

## File Locations

```bash
k3s-ansible/
├── inventory/hosts.ini                    ← Configuration
├── site.yml                               ← Main playbook
├── roles/compute-blade-agent/
│   └── tasks/main.yml                    ← Installation logic
├── manifests/
│   └── compute-blade-agent-daemonset.yaml ← K8s resources
├── scripts/
│   └── verify-compute-blade-agent.sh     ← Verification
├── GETTING_STARTED.md                     ← This file
├── COMPUTE_BLADE_AGENT.md                 ← Quick reference
├── DEPLOYMENT_CHECKLIST.md                ← Step-by-step
└── README.md                              ← Full documentation
```

---

**Ready to deploy?** Run this command:

```bash
ansible-playbook site.yml
```

Then verify with:

```bash
bash scripts/verify-compute-blade-agent.sh
```

Good luck! 🚀
