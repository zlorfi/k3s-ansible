# K3s Uninstall and Fresh Installation Guide

## Overview

This guide walks you through completely removing K3s from all nodes in your cluster and performing a clean, fresh installation.

## WARNING ⚠️

**This will:**
- Stop all K3s services
- Delete all Kubernetes data (etcd, volumes, configurations)
- Remove all running containers and pods
- Delete all kubeconfig files
- Clean up systemd service files

**This will NOT:**
- Delete the actual node's operating system
- Delete non-K3s application data outside of K3s directories
- Delete SSH keys or user home directories

**Data Loss:** Any persistent data stored in Kubernetes volumes will be lost. Make backups if needed.

## Prerequisites

- SSH access to all nodes
- Ansible installed and configured
- Inventory file properly configured
- All nodes running and accessible

## Step 1: Backup Important Data (Optional but Recommended)

If you have critical data in the cluster, back it up first:

```bash
# Backup Prometheus data if installed
kubectl --kubeconfig=./kubeconfig exec -n monitoring prometheus-prometheus-kube-prometheus-prometheus-0 -- tar czf /tmp/prometheus-backup.tar.gz -C /prometheus .

# Extract from pod
kubectl --kubeconfig=./kubeconfig cp monitoring/prometheus-prometheus-kube-prometheus-prometheus-0:/tmp/prometheus-backup.tar.gz ./prometheus-backup.tar.gz
```

## Step 2: Uninstall K3s from All Nodes

Run the uninstall playbook:

```bash
# Uninstall K3s from all nodes
ansible-playbook uninstall-k3s.yml
```

This will:
1. Stop K3s services (k3s and k3s-agent)
2. Run K3s uninstall scripts (/usr/local/bin/k3s-uninstall.sh)
3. Kill any remaining K3s/containerd processes
4. Remove all K3s data directories:
   - `/var/lib/rancher/k3s`
   - `/etc/rancher/k3s`
   - `/var/lib/cni`
   - `/var/log/pods`
   - `/var/log/containers`
   - `/var/run/k3s`
   - `/var/cache/k3s`
5. Remove K3s binaries and scripts
6. Remove systemd service files
7. Reload systemd daemon

## Step 3: Verify Cleanup

Check that K3s is completely removed:

```bash
# Verify on a node (via SSH)
ssh pi@192.168.30.101 "ls /var/lib/rancher/ 2>&1 || echo 'Directory removed successfully'"
ssh pi@192.168.30.101 "systemctl list-unit-files | grep k3s || echo 'No k3s services found'"
ssh pi@192.168.30.101 "which k3s || echo 'k3s binary removed'"

# Verify kubeconfig is gone
ls -la ./kubeconfig && echo "WARNING: kubeconfig still exists" || echo "kubeconfig removed"
```

## Step 4: Fresh K3s Installation

Once all nodes are clean, perform a fresh installation:

```bash
# Full fresh install
ansible-playbook site.yml --tags k3s-server,k3s-agent

# Or install just masters
ansible-playbook site.yml --tags k3s-server

# Or install just workers
ansible-playbook site.yml --tags k3s-agent
```

## Step 5: Verify Fresh Installation

Check that K3s is running correctly:

```bash
# Check cluster nodes
kubectl --kubeconfig=./kubeconfig get nodes

# Output should show:
# NAME     STATUS   ROLES                       AGE   VERSION
# cm4-01   Ready    control-plane,etcd,master   0m    v1.35.0+k3s1
# cm4-02   Ready    control-plane,etcd          0m    v1.35.0+k3s1
# cm4-03   Ready    control-plane,etcd          0m    v1.35.0+k3s1
# cm4-04   Ready    <none>                      0m    v1.35.0+k3s1

# Check cluster info
kubectl --kubeconfig=./kubeconfig cluster-info

# Check system pods
kubectl --kubeconfig=./kubeconfig get pods -n kube-system
```

## Step 6: Reinstall Optional Components

After fresh K3s installation, reinstall optional components:

```bash
# Install Prometheus Operator
ansible-playbook site.yml --tags prometheus-operator

# Configure Traefik
ansible-playbook site.yml --tags traefik-config

# Install compute-blade-agent
ansible-playbook site.yml --tags compute-blade-agent
```

Or reinstall everything:

```bash
# Full fresh installation with all components
ansible-playbook site.yml
```

## Selective Uninstall (Optional)

If you only want to remove specific components:

### Remove Prometheus Operator Only

```bash
kubectl --kubeconfig=./kubeconfig delete namespace monitoring
```

### Remove Traefik Configuration Only

```bash
kubectl --kubeconfig=./kubeconfig delete helmchart traefik -n kube-system
kubectl --kubeconfig=./kubeconfig delete helmchart traefik-crd -n kube-system
```

### Remove compute-blade-agent Only

```bash
kubectl --kubeconfig=./kubeconfig delete namespace compute-blade-agent
```

### Reset K3s But Keep the Installation

If you want to keep K3s but reset it:

```bash
# Stop K3s without uninstalling
ansible all -i inventory/hosts.ini -m systemd -a "name=k3s state=stopped" -b

# Manually delete specific data while keeping K3s binary:
ansible all -i inventory/hosts.ini -m shell -a "rm -rf /var/lib/rancher/k3s/server/db /var/lib/rancher/k3s/server/tls /var/lib/rancher/k3s/server/token" -b

# Restart
ansible all -i inventory/hosts.ini -m systemd -a "name=k3s state=started" -b
```

## Troubleshooting

### Uninstall Script Fails

If `/usr/local/bin/k3s-uninstall.sh` fails:

```bash
# Manually on the node:
ssh pi@192.168.30.101

# Kill processes
sudo pkill -9 k3s
sudo pkill -9 containerd

# Remove directories
sudo rm -rf /var/lib/rancher/k3s
sudo rm -rf /etc/rancher/k3s
sudo rm -rf /var/lib/cni
sudo rm -rf /var/run/k3s

# Remove binaries
sudo rm -f /usr/local/bin/k3s*
sudo rm -f /usr/local/bin/kubectl
sudo rm -f /usr/local/bin/crictl

# Remove services
sudo rm -f /etc/systemd/system/k3s*
sudo systemctl daemon-reload
```

### K3s Won't Start After Fresh Install

Clear caches and restart:

```bash
# On affected node
sudo systemctl stop k3s
sudo rm -rf /var/lib/rancher/k3s/agent/containerd
sudo systemctl start k3s

# Check logs
sudo journalctl -u k3s -n 50 -f
```

### Cluster Won't Form After Fresh Install

Reset etcd and rejoin masters:

```bash
# Stop all masters
ansible master -i inventory/hosts.ini -m systemd -a "name=k3s state=stopped" -b

# On primary master only, remove etcd
ssh pi@192.168.30.101 "sudo rm -rf /var/lib/rancher/k3s/server/db"

# Start primary master first
ansible cm4-01 -i inventory/hosts.ini -m systemd -a "name=k3s state=started" -b

# Wait for it to be ready (30 seconds)
sleep 30

# Start additional masters
ansible cm4-02,cm4-03 -i inventory/hosts.ini -m systemd -a "name=k3s state=started" -b
```

## Quick Reference Commands

```bash
# Full uninstall
ansible-playbook uninstall-k3s.yml

# Fresh install after uninstall
ansible-playbook site.yml

# Verify cluster
kubectl --kubeconfig=./kubeconfig get nodes

# View cluster info
kubectl --kubeconfig=./kubeconfig cluster-info

# Check K3s version
kubectl --kubeconfig=./kubeconfig version
```

## FAQ

**Q: Can I uninstall specific nodes only?**

A: Yes, use the `--limit` flag:
```bash
ansible-playbook uninstall-k3s.yml --limit cm4-04
```

**Q: How long does uninstall take?**

A: Usually 2-5 minutes depending on the amount of data to clean up.

**Q: Can I keep etcd data?**

A: Manually edit the uninstall playbook and remove the `/var/lib/rancher/k3s` deletion, but this is not recommended for a fresh install.

**Q: What if nodes are offline?**

A: Skip them with `--limit` and clean them up manually after they come back online.

**Q: Do I need to uninstall on workers before masters?**

A: Order doesn't matter. All nodes can be uninstalled simultaneously.

## References

- [K3s Uninstall Documentation](https://docs.k3s.io/installation/uninstall)
- [K3s Data Directory Structure](https://docs.k3s.io/storage)
- [Ansible Documentation](https://docs.ansible.com/)