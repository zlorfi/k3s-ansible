# K3s Ansible Deployment for Raspberry Pi CM4/CM5

Ansible playbook to deploy a k3s Kubernetes cluster on Raspberry Pi Compute Module 4 and 5 devices.

## Prerequisites

- Raspberry Pi CM4/CM5 modules running Raspberry Pi OS (64-bit recommended)
- SSH access to all nodes
- Ansible installed on your control machine
- SSH key-based authentication configured

## Project Structure

```bash
k3s-ansible/
├── ansible.cfg                  # Ansible configuration
├── site.yml                     # Main playbook
├── inventory/
│   └── hosts.ini               # Inventory file
├── manifests/
│   └── nginx-test-deployment.yaml  # Test nginx deployment
└── roles/
    ├── prereq/                 # Prerequisites role
    │   └── tasks/
    │       └── main.yml
    ├── k3s-server/            # K3s master/server role
    │   └── tasks/
    │       └── main.yml
    ├── k3s-agent/             # K3s worker/agent role
    │   └── tasks/
    │       └── main.yml
    └── k3s-deploy-test/       # Test deployment role
        └── tasks/
            └── main.yml
```

## Configuration

### 1. Update Inventory

Edit `inventory/hosts.ini` and add your Raspberry Pi nodes:

```ini
[master]
cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true
cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false
cm4-03 ansible_host=192.168.30.103 ansible_user=pi k3s_server_init=false

[worker]
cm4-04 ansible_host=192.168.30.104 ansible_user=pi
```

### 2. Configure Variables

In `inventory/hosts.ini`, you can customize:

- `k3s_version`: K3s version to install (default: v1.35.0+k3s1)
- `extra_server_args`: Additional arguments for k3s server
- `extra_agent_args`: Additional arguments for k3s agent
- `extra_packages`: List of additional packages to install on all nodes

### 3. Customize Extra Packages (Optional)

The playbook can install additional system utilities on all nodes. Edit the `extra_packages` variable in `inventory/hosts.ini`:

```ini
# Comma-separated list of packages
extra_packages=btop,vim,tmux,net-tools,dnsutils,iotop,ncdu,tree,jq
```

**Included packages:**

- `btop` - Better top, modern system monitor
- `vim` - Text editor
- `tmux` - Terminal multiplexer
- `net-tools` - Network tools (ifconfig, netstat, etc.)
- `dnsutils` - DNS utilities (dig, nslookup)
- `iotop` - I/O monitor
- `ncdu` - Disk usage analyzer
- `tree` - Directory tree viewer
- `jq` - JSON processor

To add packages, append them to the comma-separated list. To disable extra packages entirely, comment out or remove the `extra_packages` line.

## Usage

### Test Connectivity

Basic connectivity test:

```bash
ansible all -m ping
```

### Gather Node Information

Display critical information from all nodes (uptime, temperature, memory, disk usage, load average):

### Deploy Telegraf for Metrics Collection

Stream system metrics from all nodes to InfluxDB using Telegraf client.

**Prerequisites:**

- InfluxDB instance running and accessible
- API token with write permissions to your bucket

**Setup:**

1. Configure your InfluxDB credentials in `.env` file (already created):

```bash
# .env file (keep this secret, never commit!)
INFLUXDB_HOST=192.168.10.10
INFLUXDB_PORT=8086
INFLUXDB_ORG=family
INFLUXDB_BUCKET=rpi-cluster
INFLUXDB_TOKEN=your-api-token-here
```

2. Deploy Telegraf to all nodes:

```bash
ansible-playbook telegraf.yml
```

Or deploy to specific nodes:

```bash
# Only worker nodes
ansible-playbook telegraf.yml --limit worker

# Only master nodes
ansible-playbook telegraf.yml --limit master

# Specific node
ansible-playbook telegraf.yml --limit cm4-02
```

**Metrics Collected:**

- **System**: CPU (per-core and total), memory, swap, processes, system load
- **Disk**: Disk I/O, disk usage, inodes
- **Network**: Network interfaces, packets, errors
- **Thermal**: CPU temperature (Raspberry Pi specific)
- **K3s**: Process metrics for k3s components

**Verify Installation:**

Check Telegraf status on a node:

```bash
ssh pi@<node-ip>
sudo systemctl status telegraf
sudo journalctl -u telegraf -f
```

**View Metrics in InfluxDB:**

Once configured, metrics will appear in your InfluxDB instance under the `rpi-cluster` bucket with tags for each node hostname and node type (master/worker).

### Monitoring Dashboards

Two pre-built dashboards are available for visualizing your cluster metrics:

#### Grafana Dashboard

A comprehensive Grafana dashboard with interactive visualizations:

- CPU usage across all nodes
- Memory usage (percentage)
- CPU temperature (Raspberry Pi specific)
- System load averages

**Import to Grafana:**

1. Open Grafana and go to **Dashboards** → **New** → **Import**
2. Upload the dashboard file: `grafana/rpi-cluster-dashboard.json`
3. Your InfluxDB datasource (named `influxdb`) will be automatically selected
4. Click **Import**

**Customize the Grafana Dashboard:**

You can modify the dashboard after import to:

- Adjust time ranges (default: last 6 hours)
- Add alerts for high CPU/temperature/memory
- Add more panels for additional metrics
- Create node-specific views using Grafana variables

#### InfluxDB Dashboard

A native InfluxDB 2.x dashboard with built-in gauges and time series:

- CPU usage gauge (average)
- Memory usage gauge (average)
- CPU usage time series (6-hour view)
- Memory usage time series (6-hour view)
- CPU temperature trend
- System load trend

**Import to InfluxDB 2.8:**

**Via UI (Recommended):**

1. Open InfluxDB UI at `http://your-influxdb-host:8086`
2. Go to **Dashboards** (left sidebar)
3. Click **Create Dashboard** → **From a Template**
4. Click **Paste JSON**
5. Copy and paste the contents of `influxdb/rpi-cluster-dashboard-v2.json`
6. Click **Create Dashboard**

**Via CLI:**

```bash
influx dashboard import \
  --org family \
  --file influxdb/rpi-cluster-dashboard-v2.json
```

**Benefits of InfluxDB Dashboard:**

- Native integration - no external datasource configuration needed
- Built-in alert support
- Real-time data without polling delays
- Direct access to raw data and queries
- InfluxDB 2.8 compatible

### Deploy K3s Cluster

```bash
ansible-playbook site.yml
```

This will deploy the full k3s cluster with the test nginx application.

### Deploy Without Test Application

To skip the test deployment:

```bash
ansible-playbook site.yml --skip-tags test
```

### Deploy Only the Test Application

If the cluster is already running and you just want to deploy the test app:

```bash
ansible-playbook site.yml --tags deploy-test
```

### Deploy Only Prerequisites

```bash
ansible-playbook site.yml --tags prereq
```

## What the Playbook Does

### Prerequisites Role (`prereq`)

- Sets hostname on each node
- Updates and upgrades system packages
- Installs required packages (curl, wget, git, iptables, etc.)
- Enables cgroup memory and swap in boot config
- Configures legacy iptables (required for k3s on ARM)
- Disables swap
- Reboots if necessary

### K3s Server Role (`k3s-server`)

- Installs k3s in server mode on master node(s)
- Configures k3s with Flannel VXLAN backend (optimized for ARM)
- Retrieves and stores the node token for workers
- Copies kubeconfig to master node user
- Fetches kubeconfig to local machine for kubectl access

### K3s Agent Role (`k3s-agent`)

- Installs k3s in agent mode on worker nodes
- Joins workers to the cluster using the master's token
- Configures agents to connect to the master

### K3s Deploy Test Role (`k3s-deploy-test`)

- Waits for all cluster nodes to be ready
- Deploys the nginx test application with 5 replicas
- Verifies deployment is successful
- Displays pod distribution across nodes

## Post-Installation

After successful deployment:

1. The kubeconfig file will be saved to `./kubeconfig`
2. Use it with kubectl:

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl get nodes
```

You should see all your nodes in Ready state:

```bash
NAME     STATUS   ROLES                       AGE   VERSION
cm4-01   Ready    control-plane,etcd,master   5m    v1.35.0+k3s1
cm4-02   Ready    control-plane,etcd          3m    v1.35.0+k3s1
cm4-03   Ready    control-plane,etcd          3m    v1.35.0+k3s1
cm4-04   Ready    <none>                      3m    v1.35.0+k3s1
```

## Accessing the Cluster

### From Master Node

SSH into a master node and use kubectl:

```bash
ssh pi@192.168.30.101
kubectl get nodes
```

### From Your Local Machine

The playbook automatically fetches the kubeconfig to `./kubeconfig`. You have several options to use it:

#### Option 1: Temporary Access (Environment Variable)

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl get nodes
kubectl get pods --all-namespaces
```

#### Option 2: Merge into ~/.kube/config (Recommended)

This allows you to manage multiple clusters and switch between them:

```bash
# Backup your existing config
cp ~/.kube/config ~/.kube/config.backup

# Merge the k3s config into your existing config
KUBECONFIG=~/.kube/config:$(pwd)/kubeconfig kubectl config view --flatten > ~/.kube/config.tmp
mv ~/.kube/config.tmp ~/.kube/config

# Rename the context to something meaningful
kubectl config rename-context default k3s-pi-cluster

# View all contexts
kubectl config get-contexts

# Switch to k3s context
kubectl config use-context k3s-pi-cluster

# Switch back to other clusters
kubectl config use-context <other-context-name>
```

#### Option 3: Direct Usage

Use the kubeconfig file directly without setting environment variables:

```bash
kubectl --kubeconfig=./kubeconfig get nodes
kubectl --kubeconfig=./kubeconfig get pods --all-namespaces
```

## Ingress Setup

K3s comes with **Traefik ingress controller** pre-installed by default, which allows you to expose your applications via HTTP/HTTPS with domain names.

### How It Works

- Traefik listens on ports **80 (HTTP)** and **443 (HTTPS)** on all nodes
- Ingress rules route traffic based on hostname to different services
- Multiple applications can share the same IP using different hostnames
- No additional setup required - Traefik is ready to use after cluster deployment

### Verify Traefik is Running

```bash
kubectl --kubeconfig=./kubeconfig get pods -n kube-system -l app.kubernetes.io/name=traefik
kubectl --kubeconfig=./kubeconfig get svc -n kube-system traefik
```

### View Ingress Resources

```bash
kubectl --kubeconfig=./kubeconfig get ingress
kubectl --kubeconfig=./kubeconfig describe ingress nginx-test
```

## Testing the Cluster

A sample nginx deployment with 5 replicas and ingress is provided to test your cluster.

### Automated Deployment (via Ansible)

The test application is automatically deployed with ingress when you run the full playbook:

```bash
ansible-playbook site.yml
```

Or deploy it separately after the cluster is up:

```bash
ansible-playbook site.yml --tags deploy-test
```

The Ansible role will:

- Wait for all nodes to be ready
- Deploy the nginx application with ingress
- Wait for all pods to be running
- Show deployment status, pod distribution, ingress details, and access instructions

### Manual Deployment (via kubectl)

Deploy using kubectl:

```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl apply -f manifests/nginx-test-deployment.yaml
```

This deploys:

- Nginx deployment with 5 replicas
- ClusterIP service
- Ingress resource for domain-based access

### Verify the Deployment

Check that all 5 replicas are running:

```bash
kubectl --kubeconfig=./kubeconfig get deployments
kubectl --kubeconfig=./kubeconfig get pods -o wide
kubectl --kubeconfig=./kubeconfig get ingress
```

You should see output similar to:

```bash
NAME         READY   UP-TO-DATE   AVAILABLE   AGE
nginx-test   5/5     5            5           1m

NAME                          READY   STATUS    RESTARTS   AGE   NODE
nginx-test-7d8f4c9b6d-2xk4p   1/1     Running   0          1m    pi-worker-1
nginx-test-7d8f4c9b6d-4mz9r   1/1     Running   0          1m    pi-worker-2
nginx-test-7d8f4c9b6d-7w3qs   1/1     Running   0          1m    pi-worker-3
nginx-test-7d8f4c9b6d-9k2ln   1/1     Running   0          1m    pi-worker-1
nginx-test-7d8f4c9b6d-xr5wp   1/1     Running   0          1m    pi-worker-2
```

### Access via Ingress

Add your master node IP to /etc/hosts:

```bash
# Replace with any master or worker node IP
192.168.30.101  nginx-test.local nginx.pi.local
192.168.30.102  nginx-test.local nginx.pi.local
192.168.30.103  nginx-test.local nginx.pi.local
192.168.30.104  nginx-test.local nginx.pi.local
```

Then access via browser:

- <http://nginx-test.local>
- <http://nginx.pi.local>

Or test with curl:

```bash
# Test with any cluster node IP (master or worker)
curl -H "Host: nginx-test.local" http://192.168.30.101
curl -H "Host: nginx-test.local" http://192.168.30.102
```

### Scale the Deployment

Test scaling:

```bash
# Scale up to 10 replicas
kubectl scale deployment nginx-test --replicas=10

# Scale down to 3 replicas
kubectl scale deployment nginx-test --replicas=3

# Watch the pods being created/terminated
kubectl get pods -w
```

### Clean Up Test Deployment

When you're done testing:

```bash
kubectl delete -f manifests/nginx-test-deployment.yaml
```

## High Availability - Multi-Master Cluster

This deployment supports a **3-node highly available Kubernetes cluster** with multiple control-plane nodes for redundancy.

### Current Setup

The cluster is configured with:

- **Master Nodes (Control-Plane)**: cm4-01, cm4-02, cm4-03
- **Worker Nodes**: cm4-04
- **Virtual IP (VIP)**: 192.168.30.100 (via MikroTik router)

### Why Multi-Master?

With 3 control-plane nodes:

- **No Single Point of Failure**: If one master fails, the cluster continues operating
- **High Availability**: Automatic failover between masters
- **Better Uptime**: Can perform maintenance on one master while others serve the cluster
- **Load Distribution**: API server and etcd are distributed across 3 nodes

### How It Works

1. **Primary Master (cm4-01)**:
   - Initializes the cluster and creates the token
   - All other nodes use its token to join

2. **Additional Masters (cm4-02, cm4-03)**:
   - Join the cluster using the token from the primary master
   - Automatically become part of the control-plane
   - Synchronized with the primary master

3. **Worker Nodes (cm4-04)**:
   - Join the cluster as worker nodes
   - Can handle workload and are not part of control-plane

4. **Virtual IP (192.168.30.100)**:
   - MikroTik router provides a single entry point to the cluster
   - Automatically routes to available control-plane nodes
   - DNS points to this VIP for seamless failover

### Promoting Additional Masters

To add more masters or promote a worker to master:

1. Edit `inventory/hosts.ini` and move the node to `[master]` group:

   ```ini
   [master]
   cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true
   cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false
   cm4-03 ansible_host=192.168.30.103 ansible_user=pi k3s_server_init=false
   # To promote cm4-04 to master:
   # cm4-04 ansible_host=192.168.30.104 ansible_user=pi k3s_server_init=false

   [worker]
   # Workers only
   ```

2. Run the deployment playbook:

   ```bash
   ansible-playbook site.yml --tags k3s-server
   ```

   The playbook automatically:
   - Installs k3s server on the new master
   - Joins it to the existing cluster
   - Synchronizes with other control-plane nodes

### Monitoring Master Health

Check the status of all control-plane nodes:

```bash
kubectl get nodes -o wide | grep control-plane
# or
kubectl get nodes -L node-role.kubernetes.io/control-plane
```

To see which nodes are control-plane:

```bash
kubectl get nodes --show-labels | grep control-plane
```

Monitor etcd status across masters:

```bash
# Connect to any master
ssh pi@192.168.30.101

# Check etcd status
sudo /var/lib/rancher/k3s/data/*/bin/kubectl get nodes -n kube-system
```

### Master Failover

If a master node fails:

1. The cluster detects the failure within ~30 seconds
2. etcd automatically removes the failed node
3. Remaining masters continue operating
4. New pods are scheduled on healthy nodes

To see the status:

```bash
kubectl get nodes -o wide
```

To recover a failed master, simply:

```bash
# On the failed node, reset it
ssh pi@<failed-master-ip>
sudo /usr/local/bin/k3s-uninstall.sh

# Then re-run the playbook to rejoin it
ansible-playbook site.yml --tags k3s-server --limit <failed-master>
```

### Demoting a Master to Worker

To remove a master from control-plane and make it a worker (note: this reduces HA from 3-node to 2-node):

1. Edit `inventory/hosts.ini`:

   ```ini
   [master]
   cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true
   cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false

   [worker]
   cm4-03 ansible_host=192.168.30.103 ansible_user=pi
   cm4-04 ansible_host=192.168.30.104 ansible_user=pi
   ```

   **Warning**: This reduces your cluster to 2 master nodes. With only 2 masters, you lose quorum (require 2/3, have only 1/2 if one fails).

2. Drain the node:

   ```bash
   kubectl drain cm4-03 --ignore-daemonsets --delete-emptydir-data
   ```

3. Reset the node:

   ```bash
   ssh pi@192.168.30.103
   sudo /usr/local/bin/k3s-uninstall.sh
   ```

4. Re-run the deployment:

   ```bash
   ansible-playbook site.yml --tags k3s-agent --limit cm4-03
   ```

## Maintenance

### Updating the Cluster

K3s updates are handled automatically through the system package manager. There are several ways to update your cluster:

#### Option 1: Automatic Updates (Recommended)

K3s can automatically update itself. To enable automatic updates on all nodes:

1. Add the following to your inventory `hosts.ini`:

```ini
[k3s_cluster:vars]
k3s_version=latest
```

1. Re-run the k3s installation playbook:

```bash
ansible-playbook site.yml --tags k3s-server,k3s-agent
```

K3s will then automatically apply updates when new versions are available (typically patched versions).

#### Option 2: Manual Update to Specific Version

To update to a specific k3s version:

1. Update the `k3s_version` variable in `inventory/hosts.ini`:

```ini
[k3s_cluster:vars]
k3s_version=v1.36.0+k3s1
```

1. Run the k3s playbook to update all nodes:

```bash
# Update master first (required to generate token for agents)
ansible-playbook site.yml --tags k3s-server,k3s-agent
```

**Important:** Always update master nodes before workers. Workers need the token from the master to rejoin the cluster.

#### Option 3: Update via K3s Release Script

For more control, you can manually update k3s on individual nodes:

```bash
# SSH into a node
ssh pi@<node-ip>

# Download and install specific version
curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION=v1.36.0+k3s1 sh -

# Restart k3s
sudo systemctl restart k3s        # On master
sudo systemctl restart k3s-agent  # On workers
```

#### Checking Current K3s Version

To see the current k3s version running on your cluster:

```bash
kubectl version --short
# or
kubectl get nodes -o wide
```

To check versions on specific nodes:

```bash
ssh pi@<node-ip>
k3s --version

# Or via Ansible
ansible all -m shell -a "k3s --version" --become
```

#### Update Telegraf

To update Telegraf metrics collection to the latest version:

```bash
# Update Telegraf on all nodes
ansible-playbook telegraf.yml

# Update only specific nodes
ansible-playbook telegraf.yml --limit worker
```

#### Post-Update Verification

After updating, verify your cluster is healthy:

```bash
# Check all nodes are ready
kubectl get nodes

# Check pod status
kubectl get pods --all-namespaces

# Check cluster info
kubectl cluster-info

# View recent events
kubectl get events --all-namespaces --sort-by='.lastTimestamp'
```

#### Rollback (if needed)

If an update causes issues, you can rollback to a previous version:

```bash
# Update inventory with previous version
# [k3s_cluster:vars]
# k3s_version=v1.35.0+k3s1

# Re-run the playbook
ansible-playbook site.yml --tags k3s-server,k3s-agent
```

### Rebooting Cluster Nodes

A dedicated playbook is provided to safely reboot all cluster nodes:

```bash
ansible-playbook reboot.yml
```

This playbook will:

1. Reboot worker nodes first (one at a time, serially)
2. Wait for each worker to come back online and k3s-agent to be running
3. Reboot master nodes (one at a time, serially)
4. Wait for each master to come back online and k3s to be running
5. Verify the cluster status and show all nodes are ready

The serial approach ensures that only one node reboots at a time, maintaining cluster availability.

### Reboot Only Workers

```bash
ansible-playbook reboot.yml --limit worker
```

### Reboot Only Masters

```bash
ansible-playbook reboot.yml --limit master
```

### Reboot a Specific Node

```bash
ansible-playbook reboot.yml --limit cm4-04
```

## Troubleshooting

### Check k3s service status

On master:

```bash
sudo systemctl status k3s
sudo journalctl -u k3s -f
```

On workers:

```bash
sudo systemctl status k3s-agent
sudo journalctl -u k3s-agent -f
```

### Reset a node

If you need to reset a node and start over:

```bash
# On the node
/usr/local/bin/k3s-uninstall.sh          # For server
/usr/local/bin/k3s-agent-uninstall.sh    # For agent
```

### Common Issues

1. **Nodes not joining**: Check firewall rules. K3s requires port 6443 open on the master.
2. **Memory issues**: Ensure cgroup memory is enabled (the playbook handles this).
3. **Network issues**: The playbook uses VXLAN backend which works better on ARM devices.

## Customization

### Add More Master Nodes (HA Setup)

For a high-availability setup, you can add more master nodes:

```ini
[master]
pi-master-1 ansible_host=192.168.30.100 ansible_user=pi
pi-master-2 ansible_host=192.168.30.101 ansible_user=pi
pi-master-3 ansible_host=192.168.30.102 ansible_user=pi
```

You'll need to configure an external database (etcd or PostgreSQL) for HA.

### Custom K3s Arguments

Modify `extra_server_args` or `extra_agent_args` in the inventory:

```ini
[k3s_cluster:vars]
extra_server_args="--flannel-backend=vxlan --disable traefik --disable servicelb"
extra_agent_args="--node-label foo=bar"
```

## Compute Blade Agent Deployment

The playbook includes automatic deployment of the Compute Blade Agent, a system service for managing Compute Blade hardware (Raspberry Pi CM4/CM5 modules). The agent monitors hardware states, reacts to temperature changes and button presses, and exposes metrics via Prometheus.

### Components

1. **compute-blade-agent**: Daemon that monitors hardware and manages blade operations
2. **bladectl**: Command-line tool for local/remote interaction with the agent
3. **fanunit.uf2**: Firmware for the fan unit microcontroller

### Configuration

The compute-blade-agent deployment is controlled by the `enable_compute_blade_agent` variable in `inventory/hosts.ini`:

```ini
# Enable/disable compute-blade-agent on all nodes (control-plane and workers)
enable_compute_blade_agent=true
```

To disable on specific nodes, add an override:

```ini
[master]
cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true enable_compute_blade_agent=true
cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false enable_compute_blade_agent=false
cm4-03 ansible_host=192.168.30.103 ansible_user=pi k3s_server_init=false enable_compute_blade_agent=false

[worker]
cm4-04 ansible_host=192.168.30.104 ansible_user=pi enable_compute_blade_agent=true
```

### Deployment

The compute-blade-agent is automatically deployed as part of the main playbook:

```bash
ansible-playbook site.yml
```

Or deploy only the compute-blade-agent on all nodes:

```bash
ansible-playbook site.yml --tags compute-blade-agent
```

### Verification

Check the agent status on any node:

```bash
# SSH into any node
ssh pi@192.168.30.101

# Check service status
sudo systemctl status compute-blade-agent

# View logs
sudo journalctl -u compute-blade-agent -f

# Check binary installation
/usr/bin/compute-blade-agent --version
```

### Configuration Files

The compute-blade-agent creates its configuration at:

```yaml
/etc/compute-blade-agent/config.yaml
```

Configuration can also be controlled via environment variables prefixed with `BLADE_`.

### Metrics and Monitoring

The compute-blade-agent exposes Prometheus metrics. To monitor the agents:

1. **Optional Kubernetes resources** are available in `manifests/compute-blade-agent-daemonset.yaml`

2. Deploy the optional monitoring resources (requires Prometheus):

```bash
kubectl apply -f manifests/compute-blade-agent-daemonset.yaml
```

### Features

- **Hardware Monitoring**: Tracks temperature, fan speed, and button events
- **Critical Mode**: Automatically enters maximum fan speed + red LED during overheating
- **Identification**: Locate specific blades via LED blinking
- **Metrics Export**: Prometheus-compatible metrics endpoint

### Troubleshooting compute-blade-agent

#### Service fails to start

Check the installer output:

```bash
sudo journalctl -u compute-blade-agent -n 50
```

#### Agent not detecting hardware

Verify the Compute Blade hardware is properly connected. The agent logs detailed information:

```bash
sudo journalctl -u compute-blade-agent -f
```

#### Re-run installation

To reinstall compute-blade-agent:

```bash
# SSH into the node
ssh pi@<node-ip>

# Uninstall
sudo /usr/local/bin/k3s-uninstall-compute-blade-agent.sh 2>/dev/null || echo "Not found, continuing"

# Remove from Ansible to reinstall
# Then re-run the playbook
ansible-playbook site.yml --tags compute-blade-agent
```

## External DNS Configuration

To use external domains (like `test.zlor.fi`) with your k3s cluster ingress, you need to configure DNS. Your cluster uses a Virtual IP (192.168.30.100) via MikroTik for high availability.

### Step 1: Configure DNS Server Records

On your DNS server, add **A records** pointing to your k3s cluster nodes:

#### Option A: Virtual IP (VIP) via MikroTik - Recommended for HA

Use your MikroTik router's Virtual IP (192.168.30.100) for high availability:

```dns
test.zlor.fi  A  192.168.30.100
```

**Pros:**

- Single IP for entire cluster
- Hardware-based failover (more reliable)
- Better performance
- No additional software needed
- Automatically routes to available masters

See [MIKROTIK-VIP-SETUP-CUSTOM.md](MIKROTIK-VIP-SETUP-CUSTOM.md) for detailed setup instructions.

#### Option B: Multiple Records (Load Balanced)

If your DNS supports multiple A records, point to all cluster nodes:

```dns
test.zlor.fi  A  192.168.30.101
test.zlor.fi  A  192.168.30.102
test.zlor.fi  A  192.168.30.103
test.zlor.fi  A  192.168.30.104
```

**Pros:** Load balanced, automatic failover
**Cons:** Requires DNS server support for multiple A records

#### Option C: Single Master Node (No Failover)

For simple setups without redundancy:

```dns
test.zlor.fi  A  192.168.30.101
```

**Pros:** Simple, works with any DNS server
**Cons:** No failover if that node is down (not recommended for HA clusters)

### Step 2: Configure Cluster Nodes for External DNS

K3s nodes need to be able to resolve external DNS queries. Update the DNS resolver on all nodes:

#### Option A: Ansible Playbook (Recommended)

Create a new playbook `dns-config.yml`:

```yaml
---
- name: Configure external DNS resolver
  hosts: all
  become: true
  tasks:
    - name: Update /etc/resolv.conf with custom DNS
      copy:
        content: |
          nameserver 8.8.8.8
          nameserver 8.8.4.4
          nameserver 192.168.1.1
        dest: /etc/resolv.conf
        owner: root
        group: root
        mode: '0644'
      notify: Update systemd-resolved

    - name: Make resolv.conf immutable
      file:
        path: /etc/resolv.conf
        attributes: '+i'
        state: file

    - name: Configure systemd-resolved for external DNS
      copy:
        content: |
          [Resolve]
          DNS=8.8.8.8 8.8.4.4 192.168.1.1
          FallbackDNS=8.8.8.8
          DNSSECNegativeTrustAnchors=zlor.fi
        dest: /etc/systemd/resolved.conf
        owner: root
        group: root
        mode: '0644'
      notify: Restart systemd-resolved

  handlers:
    - name: Update systemd-resolved
      systemd:
        name: systemd-resolved
        state: restarted
        daemon_reload: yes
```

Apply the playbook:

```bash
ansible-playbook dns-config.yml
```

#### Option B: Manual Configuration on Each Node

SSH into each node and update DNS:

```bash
ssh pi@192.168.30.101
sudo nano /etc/systemd/resolved.conf
```

Add or modify:

```ini
[Resolve]
DNS=8.8.8.8 8.8.4.4 192.168.1.1
FallbackDNS=8.8.8.8
DNSSECNegativeTrustAnchors=zlor.fi
```

Save and restart:

```bash
sudo systemctl restart systemd-resolved
```

Verify DNS is working:

```bash
nslookup test.zlor.fi
dig test.zlor.fi
```

### Step 3: Update Ingress Configuration

Your nginx-test deployment has already been updated to include `test.zlor.fi`. Verify the ingress:

```bash
kubectl get ingress nginx-test -o yaml
```

You should see:

```yaml
spec:
  rules:
  - host: test.zlor.fi
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

### Step 4: Test External Domain Access

Once DNS is configured, test access from your local machine:

```bash
# Test DNS resolution
nslookup test.zlor.fi

# Test HTTP access
curl http://test.zlor.fi

# With verbose output
curl -v http://test.zlor.fi

# Test from all cluster IPs
for ip in 192.168.30.{101..104}; do
  echo "Testing $ip:"
  curl -H "Host: test.zlor.fi" http://$ip
done
```

### Troubleshooting DNS

#### DNS Resolution Failing

Check if systemd-resolved is running:

```bash
systemctl status systemd-resolved
```

Test DNS from a node:

```bash
ssh pi@192.168.30.101
nslookup test.zlor.fi
dig test.zlor.fi @8.8.8.8
```

#### Ingress Not Responding

Check if Traefik is running:

```bash
kubectl get pods -n kube-system -l app.kubernetes.io/name=traefik
```

Check ingress status:

```bash
kubectl get ingress
kubectl describe ingress nginx-test
```

#### Request Timing Out

Verify network connectivity:

```bash
# From your machine
ping 192.168.30.101
ping 192.168.30.102

# From a cluster node
ssh pi@192.168.30.101
ping test.zlor.fi
curl -v http://test.zlor.fi
```

### Adding More Domains

To add additional domains (e.g., `api.zlor.fi`, `admin.zlor.fi`):

1. Add DNS A records for each domain pointing to your cluster nodes
1. Update the ingress YAML with new rules:

```yaml
spec:
  rules:
  - host: test.zlor.fi
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: nginx-test
            port:
              number: 80
  - host: api.zlor.fi
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-service
            port:
              number: 8080
```

1. Apply the updated manifest:

```bash
kubectl apply -f manifests/nginx-test-deployment.yaml
```

## Uninstall

To completely remove k3s from all nodes:

```bash
# Create an uninstall playbook or run manually on each node
ansible all -m shell -a "/usr/local/bin/k3s-uninstall.sh" --become
ansible workers -m shell -a "/usr/local/bin/k3s-agent-uninstall.sh" --become
```

To uninstall compute-blade-agent:

```bash
# Uninstall from all worker nodes
ansible worker -m shell -a "bash /usr/local/bin/k3s-uninstall-compute-blade-agent.sh" --become
```

## License

MIT

## References

- [K3s Documentation](https://docs.k3s.io/)
- [K3s on Raspberry Pi](https://docs.k3s.io/installation/requirements)
