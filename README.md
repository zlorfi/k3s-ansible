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
pi-master ansible_host=192.168.30.100 ansible_user=pi

[worker]
pi-worker-1 ansible_host=192.168.30.102 ansible_user=pi
pi-worker-2 ansible_host=192.168.30.103 ansible_user=pi
pi-worker-3 ansible_host=192.168.30.104 ansible_user=pi
```

### 2. Configure Variables

In `inventory/hosts.ini`, you can customize:

- `k3s_version`: K3s version to install (default: v1.28.3+k3s1)
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

```bash
ansible all -m ping
```

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
NAME          STATUS   ROLES                  AGE   VERSION
pi-master     Ready    control-plane,master   5m    v1.28.3+k3s1
pi-worker-1   Ready    <none>                 3m    v1.28.3+k3s1
pi-worker-2   Ready    <none>                 3m    v1.28.3+k3s1
```

## Accessing the Cluster

### From Master Node

SSH into the master node and use kubectl:

```bash
ssh pi@pi-master
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
# Replace 192.168.30.101 with your master node IP
192.168.30.101  nginx-test.local nginx.pi.local
```

Then access via browser:

- <http://nginx-test.local>
- <http://nginx.pi.local>

Or test with curl:

```bash
# Replace with your master node IP
curl -H "Host: nginx-test.local" http://192.168.30.101
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

## Maintenance

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
ansible-playbook reboot.yml --limit pi-worker-1
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
# Enable/disable compute-blade-agent on all worker nodes
enable_compute_blade_agent=true
```

To disable on specific nodes, add an override:

```ini
[worker]
cb-02 ansible_host=192.168.30.102 ansible_user=pi enable_compute_blade_agent=false
cb-03 ansible_host=192.168.30.103 ansible_user=pi
cb-04 ansible_host=192.168.30.104 ansible_user=pi
```

### Deployment

The compute-blade-agent is automatically deployed as part of the main playbook:

```bash
ansible-playbook site.yml
```

Or deploy only the compute-blade-agent on worker nodes:

```bash
ansible-playbook site.yml --tags compute-blade-agent
```

### Verification

Check the agent status on a worker node:

```bash
# SSH into a worker node
ssh pi@192.168.30.102

# Check service status
sudo systemctl status compute-blade-agent

# View logs
sudo journalctl -u compute-blade-agent -f

# Check binary installation
/usr/local/bin/compute-blade-agent --version
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
