# Compute Blade Agent Deployment Checklist

## Pre-Deployment

- [ ] Review inventory configuration: `cat inventory/hosts.ini`
- [ ] Verify SSH access to all worker nodes: `ansible all -m ping`
- [ ] Review Compute Blade Agent documentation: `cat COMPUTE_BLADE_AGENT.md`
- [ ] Check that compute-blade-agent is enabled: `grep enable_compute_blade_agent inventory/hosts.ini`

## Deployment

### Option 1: Full Stack Deployment (Recommended for new clusters)

```bash
ansible-playbook site.yml
```

This will:

1. Prepare all nodes (prerequisites)
2. Install K3s server on master
3. Install K3s agents on workers
4. Install compute-blade-agent on workers
5. Deploy test nginx application

- [ ] Start full deployment
- [ ] Wait for completion (typically 10-20 minutes)
- [ ] Check for any errors in output

### Option 2: Skip Test Application

```bash
ansible-playbook site.yml --skip-tags test
```

- [ ] Start deployment without test app
- [ ] Faster deployment, suitable if cluster already has applications

### Option 3: Deploy Only Compute Blade Agent

```bash
ansible-playbook site.yml --tags compute-blade-agent
```

- [ ] Use on existing K3s cluster
- [ ] Deploy agent to all configured workers
- [ ] Verify with verification script

## Post-Deployment Verification

### 1. Check Cluster Status

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl get nodes
```

- [ ] All master and worker nodes should show "Ready"

### 2. Run Verification Script

```bash
bash scripts/verify-compute-blade-agent.sh
```

- [ ] All worker nodes pass connectivity check
- [ ] Binary is installed at `/usr/local/bin/compute-blade-agent`
- [ ] Service status shows "Running"
- [ ] Config file exists at `/etc/compute-blade-agent/config.yaml`

### 3. Manual Verification on a Master Node

```bash
# Connect to any master (cm4-01, cm4-02, or cm4-03)
ssh pi@192.168.30.101
kubectl get nodes
```

- [ ] All 3 masters show as "Ready"
- [ ] Worker node (cm4-04) shows as "Ready"

### 4. Check Etcd Quorum

```bash
ssh pi@192.168.30.101
sudo /var/lib/rancher/k3s/data/*/bin/etcdctl member list
```

- [ ] All 3 etcd members show as active
- [ ] Cluster has quorum (2/3 minimum for failover)

### 5. Verify Kubeconfig

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl config get-contexts
```

- [ ] Shows contexts: cm4-01, cm4-02, cm4-03, and default
- [ ] All contexts point to correct control-plane nodes

## Optional: Kubernetes Monitoring Setup

### Deploy Monitoring Resources

```bash
kubectl apply -f manifests/compute-blade-agent-daemonset.yaml
```

- [ ] Check namespace creation: `kubectl get namespace compute-blade-agent`
- [ ] Check DaemonSet: `kubectl get daemonset -n compute-blade-agent`
- [ ] Check service: `kubectl get service -n compute-blade-agent`

### Enable Prometheus Monitoring

1. Edit `manifests/compute-blade-agent-daemonset.yaml`
2. Uncomment the ServiceMonitor section
3. Apply: `kubectl apply -f manifests/compute-blade-agent-daemonset.yaml`

- [ ] ServiceMonitor created (if Prometheus operator installed)
- [ ] Prometheus scrape targets added (visible in Prometheus UI)

## Troubleshooting

### Service Not Running

- [ ] Check status: `sudo systemctl status compute-blade-agent`
- [ ] Check logs: `sudo journalctl -u compute-blade-agent -f`
- [ ] Check if binary exists: `ls -la /usr/local/bin/compute-blade-agent`
- [ ] Check systemd unit: `cat /etc/systemd/system/compute-blade-agent.service`

### Installation Failed

- [ ] Re-run Ansible playbook: `ansible-playbook site.yml --tags compute-blade-agent`
- [ ] Check for network connectivity during installation
- [ ] Verify sufficient disk space on nodes
- [ ] Check /tmp directory permissions

### Hardware Not Detected

- [ ] Verify physical hardware connection
- [ ] Check dmesg: `sudo dmesg | grep -i compute`
- [ ] Check hardware info: `lspci` or `lsusb`
- [ ] Review compute-blade-agent logs for detection messages

## Configuration

### Global Configuration

To enable/disable on all workers, edit `inventory/hosts.ini`:

```ini
[k3s_cluster:vars]
enable_compute_blade_agent=true  # or false
```

- [ ] Configuration reviewed and correct
- [ ] Saved inventory file

### Per-Node Configuration

Note: cm4-02 and cm4-03 are now **master nodes**, not workers. To enable/disable compute-blade-agent on specific nodes:

```ini
[master]
cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true enable_compute_blade_agent=false
cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false enable_compute_blade_agent=false
cm4-03 ansible_host=192.168.30.103 ansible_user=pi k3s_server_init=false enable_compute_blade_agent=false

[worker]
cm4-04 ansible_host=192.168.30.104 ansible_user=pi enable_compute_blade_agent=true
```

- [ ] Per-node settings configured as needed
- [ ] Master nodes typically don't need compute-blade-agent
- [ ] Saved inventory file
- [ ] Re-run playbook if changes made

### Agent Configuration

Edit configuration on the node:

```bash
ssh pi@<worker-ip>
sudo vi /etc/compute-blade-agent/config.yaml
sudo systemctl restart compute-blade-agent
```

- [ ] Configuration customized (if needed)
- [ ] Service restarted successfully

## Maintenance

### Restart Service

```bash
ssh pi@<worker-ip>
sudo systemctl restart compute-blade-agent
```

- [ ] Service restarted
- [ ] Service is still running

### View Real-time Logs

```bash
ssh pi@<worker-ip>
sudo journalctl -u compute-blade-agent -f
```

- [ ] Monitor for any issues
- [ ] Press Ctrl+C to exit

### Check Service on All Workers

```bash
ansible worker -m shell -a "systemctl status compute-blade-agent" --become
```

- [ ] All workers show active status

## HA Cluster Maintenance

### Testing Failover

Your 3-node HA cluster can handle one master going down (maintains 2/3 quorum):

```bash
# Reboot one master while monitoring cluster
ssh pi@192.168.30.101
sudo reboot

# From another terminal, watch cluster status
watch kubectl get nodes
```

- [ ] Cluster remains operational with 2/3 masters
- [ ] Pods continue running
- [ ] Can still kubectl from cm4-02 or cm4-03 context

## Uninstall (if needed)

### Uninstall K3s from All Nodes

```bash
ansible all -m shell -a "bash /usr/local/bin/k3s-uninstall.sh" --become
ansible worker -m shell -a "bash /usr/local/bin/k3s-agent-uninstall.sh" --become
```

- [ ] All K3s services stopped
- [ ] Cluster data cleaned up

### Disable in Future Deployments

Edit `inventory/hosts.ini`:

```ini
enable_compute_blade_agent=false
```

- [ ] Setting disabled
- [ ] Won't be deployed on next playbook run

## Documentation References

- [ ] Read README.md compute-blade-agent section
- [ ] Read COMPUTE_BLADE_AGENT.md quick reference
- [ ] Check GitHub repo: [compute-blade-agent](https://github.com/compute-blade-community/compute-blade-agent)
- [ ] Review Ansible role: `cat roles/compute-blade-agent/tasks/main.yml`

## Completion

- [ ] All deployment steps completed
- [ ] All verification checks passed
- [ ] Documentation reviewed
- [ ] Team notified of deployment
- [ ] Monitoring configured (optional)
- [ ] Backup of configuration taken

## Notes

Document any issues, customizations, or special configurations here:

```text
[Add notes here]
```

---

**Last Updated**: 2025-11-24
**Status**: Ready for Deployment
