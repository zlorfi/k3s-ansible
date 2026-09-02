# K3s Ansible Deployment for Raspberry Pi CM4/CM5

🚀 **Production-ready Kubernetes cluster automation** for Raspberry Pi Compute Module 4/5 hardware with built-in monitoring, high availability, and hardware management.

## ✨ Features

- **🔄 3-node HA control plane** with automatic failover
- **📊 Comprehensive monitoring** (Telegraf → InfluxDB → Grafana)
- **🌐 Traefik ingress** with automatic TLS via Let's Encrypt + Hetzner DNS-01
- **🖥️ Compute Blade Agent** for hardware monitoring
- **📈 Prometheus metrics** with custom dashboards
- **🔧 One-command deployment** and maintenance

## 📋 Prerequisites

- **Hardware**: Raspberry Pi CM4/CM5 modules
- **OS**: Raspberry Pi OS (64-bit recommended)
- **Network**: SSH access to all nodes
- **Control machine**: Ansible installed
- **Authentication**: SSH key-based configured

## 🏗️ Project Structure

```bash
k3s-ansible/
├── 📄 ansible.cfg                    # Ansible configuration
├── 📄 site.yml                       # Main deployment playbook
├── 📁 inventory/
│   └── 📄 hosts.ini                  # Cluster inventory
├── 📁 manifests/                     # Kubernetes manifests
│   └── 📄 nginx-test-deployment.yaml # Test application
├── 📁 roles/                         # Ansible roles
│   ├── 📁 prereq/                    # System preparation
│   ├── 📁 k3s-server/               # Control-plane setup
│   ├── 📁 k3s-agent/                # Worker node setup
│   ├── 📁 k3s-deploy-test/          # Test deployment
│   ├── 📁 compute-blade-agent/      # Hardware monitoring
│   ├── 📁 prometheus-operator/      # Monitoring stack
│   ├── 📁 telegraf/                 # Metrics collection
│   ├── 📁 traefik-config/           # Traefik ACME/TLS configuration
│   └── 📁 vaultwarden/              # Vaultwarden password manager
├── 📁 grafana/                       # Grafana dashboards
├── 📁 influxdb/                      # InfluxDB dashboards
└── 📄 telegraf.yml                  # Metrics deployment
```

## ⚙️ Quick Setup

### 1. Configure Inventory

Edit `inventory/hosts.ini` with your node details:

```ini
[master]
cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true
cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false
cm4-03 ansible_host=192.168.30.103 ansible_user=pi k3s_server_init=false

[worker]
cm4-04 ansible_host=192.168.30.104 ansible_user=pi
```

### 2. Key Configuration Options

```ini
[k3s_cluster:vars]
k3s_version=v1.35.0+k3s1              # K3s version
extra_packages=btop,vim,tmux,net-tools # System utilities
enable_compute_blade_agent=true        # Hardware monitoring
enable_prometheus_operator=true        # Monitoring stack
```

### 3. Setup Environment Variables

Create a `.env` file in the repository root with your credentials:

```bash
cat > .env << EOF
# InfluxDB / Telegraf metrics
INFLUXDB_HOST=192.168.10.10
INFLUXDB_PORT=8086
INFLUXDB_ORG=family
INFLUXDB_BUCKET=rpi-cluster
INFLUXDB_TOKEN=your-influxdb-api-token-here

# Traefik ACME / Let's Encrypt via Hetzner DNS-01
ACME_EMAIL=you@yourdomain.com
HETZNER_API_TOKEN=your-hetzner-dns-api-token-here

# Vaultwarden
ADMIN_TOKEN=your-vaultwarden-admin-token-here

# Grafana (optional — falls back to 'admin' if unset)
GRAFANA_ADMIN_PASSWORD=your-grafana-admin-password-here
EOF
```

**Hetzner DNS API Token requirements**: Create a token in the [Hetzner DNS Console](https://dns.hetzner.com/settings/api-token) → **API tokens** → **Create access token**. The token grants read/write access to all DNS zones in that Hetzner DNS account, so use an account that manages the zones you issue certificates for (`zlor.fi`).

> Note: this is a **Hetzner DNS Console** token (`dns.hetzner.com`), *not* a Hetzner Cloud API token from `console.hetzner.cloud`. They are not interchangeable.

**⚠️ Security Note:** This file is ignored by Git (`.gitignore`) and should never be committed. Keep actual tokens secure and only on your local machine.

### 4. Test Connectivity

```bash
ansible all -m ping
```

## 🚀 Deployment Commands

**Prerequisites:** Make sure your `inventory/hosts.ini` is configured and `.env` file is created (see Setup steps above).

### Full Cluster Deployment
```bash
ansible-playbook site.yml
```

### Component-Specific Deployment
```bash
# Prepare nodes only
ansible-playbook site.yml --tags prereq

# Deploy monitoring
ansible-playbook telegraf.yml

# Configure Traefik ACME/TLS only (on already-running cluster)
ansible-playbook site.yml --tags traefik-config

# Deploy Vaultwarden only
ansible-playbook site.yml --tags vaultwarden

# Deploy test application only
ansible-playbook site.yml --tags deploy-test

# Skip test deployment
ansible-playbook site.yml --skip-tags test
```

## 📊 Monitoring Setup

### Telegraf Metrics Collection

**1. Configure InfluxDB credentials** in `.env`:

```bash
INFLUXDB_HOST=192.168.10.10
INFLUXDB_PORT=8086
INFLUXDB_ORG=family
INFLUXDB_BUCKET=rpi-cluster
INFLUXDB_TOKEN=your-api-token-here
```

**2. Deploy Telegraf**:

```bash
ansible-playbook telegraf.yml
```

**Metrics Collected:**
- 🖥️ **System**: CPU, memory, processes, load
- 💾 **Disk**: I/O, usage, inodes  
- 🌐 **Network**: Interfaces, packets, errors
- 🌡️ **Thermal**: CPU temperature (Pi-specific)
- ⚙️ **K3s**: Process metrics

### Dashboard Options

#### 📈 Grafana Dashboard
```bash
# Import: grafana/rpi-cluster-dashboard.json
# Features: Interactive visualizations, alerts, node-specific views
```

#### 📊 InfluxDB Dashboard  
```bash
# Import: influxdb/rpi-cluster-dashboard-v2.json
# Features: Native integration, real-time data, built-in alerts
```

## 🎯 What Gets Deployed

### 📋 System Preparation (`prereq`)
- ✅ Hostname configuration
- ✅ System updates & package installation
- ✅ cgroup memory & swap configuration
- ✅ Legacy iptables setup (ARM requirement)
- ✅ Swap disabling

### 🎯 Control Plane (`k3s-server`)
- ✅ K3s server installation
- ✅ Flannel VXLAN networking (ARM optimized)
- ✅ Cluster token management
- ✅ Kubeconfig generation & retrieval

### 👥 Worker Nodes (`k3s-agent`)
- ✅ K3s agent installation
- ✅ Cluster joining via master token
- ✅ Network configuration

### 🧪 Test Application (`k3s-deploy-test`)
- ✅ Nginx deployment (5 replicas)
- ✅ Ingress configuration
- ✅ Health verification
- ✅ Pod distribution analysis

## 🎉 Post-Installation

### Access Your Cluster

**📁 Kubeconfig Location**: `./kubeconfig`

**🔧 Quick Setup**:
```bash
export KUBECONFIG=$(pwd)/kubeconfig
kubectl get nodes
```

**Expected Output**:
```bash
NAME     STATUS   ROLES                       AGE   VERSION
cm4-01   Ready    control-plane,etcd,master   5m    v1.35.0+k3s1
cm4-02   Ready    control-plane,etcd          3m    v1.35.0+k3s1
cm4-03   Ready    control-plane,etcd          3m    v1.35.0+k3s1
cm4-04   Ready    <none>                      3m    v1.35.0+k3s1
```

### Access Options

#### 🌐 Local Machine Access
```bash
# Option 1: Environment variable
export KUBECONFIG=$(pwd)/kubeconfig

# Option 2: Merge with existing config
KUBECONFIG=~/.kube/config:$(pwd)/kubeconfig kubectl config view --flatten > ~/.kube/config.tmp
mv ~/.kube/config.tmp ~/.kube/config
kubectl config rename-context default k3s-pi-cluster

# Option 3: Direct usage
kubectl --kubeconfig=./kubeconfig get nodes
```

#### 🖥️ Direct SSH Access
```bash
ssh pi@192.168.30.101
kubectl get nodes
```

## 🌐 Ingress & Networking

### Traefik Ingress Controller
**✅ Pre-installed** by K3s and configured for automatic TLS.

**How it works:**
- Listens on port 80 (HTTP) and 443 (HTTPS)
- Routes traffic by hostname to the correct service
- Multiple apps share the same IP via different domains
- HTTP traffic is automatically redirected to HTTPS

**Verify Traefik:**
```bash
kubectl get pods -n kube-system -l app.kubernetes.io/name=traefik
kubectl get svc -n kube-system traefik
kubectl get ingress --all-namespaces
```

### TLS Certificates — Let's Encrypt via Hetzner DNS-01

Certificates are issued automatically by **Traefik's built-in ACME client** using a **DNS-01 challenge** through the Hetzner DNS API. No cert-manager is required.

**How it works:**
1. When an Ingress with `certresolver: letsencrypt-hetzner` is deployed, Traefik requests a certificate from Let's Encrypt.
2. Traefik creates a `_acme-challenge.<domain>` TXT record via the Hetzner DNS API to prove domain ownership.
3. Let's Encrypt validates the record and issues the certificate.
4. Traefik stores the certificate in `/data/acme.json` (on a PVC) and auto-renews it before expiry.

**The `traefik-config` role** (`roles/traefik-config/`) provisions this by:
- Creating a `traefik-hetzner-token` Kubernetes Secret in `kube-system` from `.env`
- Deleting the legacy `traefik-cloudflare-token` secret if it still exists
- Applying a `HelmChartConfig` CRD that patches the K3s-bundled Traefik Helm release with the ACME resolver and Hetzner provider configuration

Hetzner's DNS propagation is slower than Cloudflare's, so the role sets
`HETZNER_PROPAGATION_TIMEOUT=300`, `HETZNER_POLLING_INTERVAL=5` and `HETZNER_TTL=120`
(tunable in `roles/traefik-config/defaults/main.yml`).

**Deploy or re-apply the configuration:**
```bash
ansible-playbook site.yml --tags traefik-config
```

**Annotate an Ingress to use automatic TLS:**
```yaml
annotations:
  traefik.ingress.kubernetes.io/router.entrypoints: websecure
  traefik.ingress.kubernetes.io/router.tls: "true"
  traefik.ingress.kubernetes.io/router.tls.certresolver: letsencrypt-hetzner
```

**Check certificate status:**
```bash
# View ACME storage (cert state)
kubectl exec -n kube-system deploy/traefik -- cat /data/acme.json | jq '.["letsencrypt-hetzner"].Certificates[].domain'

# Check Traefik logs for ACME activity
kubectl logs -n kube-system deploy/traefik | grep -i acme
```

**Switch to Let's Encrypt staging** (to avoid rate limits during testing):

Edit `roles/traefik-config/defaults/main.yml`:
```yaml
traefik_acme_server: https://acme-staging-v02.api.letsencrypt.org/directory
```
Then re-run `ansible-playbook site.yml --tags traefik-config`.

## 🧪 Test Your Cluster

### Automated Test Deployment
```bash
# Deploy with full cluster
ansible-playbook site.yml

# Deploy test app only
ansible-playbook site.yml --tags deploy-test
```

### Manual Test Deployment
```bash
kubectl apply -f manifests/nginx-test-deployment.yaml
```

### Verify Test Deployment
```bash
kubectl get deployments
kubectl get pods -o wide
kubectl get ingress
```

**Expected Output:**
```bash
NAME         READY   UP-TO-DATE   AVAILABLE   AGE
nginx-test   5/5     5            5           1m

NAME                          READY   STATUS    NODE
nginx-test-7d8f4c9b6d-2xk4p   1/1     Running   cm4-04
nginx-test-7d8f4c9b6d-4mz9r   1/1     Running   cm4-04
nginx-test-7d8f4c9b6d-7w3qs   1/1     Running   cm4-03
nginx-test-7d8f4c9b6d-9k2ln   1/1     Running   cm4-03
nginx-test-7d8f4c9b6d-xr5wp   1/1     Running   cm4-02
```

### Access Test Application

**1. Add to /etc/hosts:**
```bash
192.168.30.101  nginx-test.local
192.168.30.102  nginx-test.local
192.168.30.103  nginx-test.local
192.168.30.104  nginx-test.local
```

**2. Access via browser:**
- 🌐 http://nginx-test.local

**3. Test with curl:**
```bash
curl -H "Host: nginx-test.local" http://192.168.30.101
```

### Scale Test
```bash
# Scale up/down
kubectl scale deployment nginx-test --replicas=10
kubectl scale deployment nginx-test --replicas=3

# Watch scaling
kubectl get pods -w
```

### Cleanup
```bash
kubectl delete -f manifests/nginx-test-deployment.yaml
```

## 🛡️ High Availability

### 3-Node Control Plane
**✅ Production-ready HA setup**

**Architecture:**
- 🎯 **Control Plane**: cm4-01, cm4-02, cm4-03
- 👥 **Workers**: cm4-04  
- 🌐 **Virtual IP**: 192.168.30.100 (MikroTik)

**Benefits:**
- 🚫 **No SPOF** - Cluster survives master failures
- 🔄 **Auto failover** - Seamless master switching
- ⚡ **Load distribution** - API server & etcd spread across nodes
- 🔧 **Zero downtime maintenance** - Update masters one-by-one

### Master Management

**🔍 Monitor Master Health:**
```bash
kubectl get nodes -L node-role.kubernetes.io/control-plane
kubectl get nodes --show-labels | grep control-plane
```

**⬆️ Promote Worker to Master:**
```ini
# Edit inventory/hosts.ini
[master]
cm4-01 ansible_host=192.168.30.101 ansible_user=pi k3s_server_init=true
cm4-02 ansible_host=192.168.30.102 ansible_user=pi k3s_server_init=false
cm4-03 ansible_host=192.168.30.103 ansible_user=pi k3s_server_init=false
cm4-04 ansible_host=192.168.30.104 ansible_user=pi k3s_server_init=false  # Promoted

[worker]
# Workers only
```

```bash
ansible-playbook site.yml --tags k3s-server
```

**🔄 Master Recovery:**
```bash
# Reset failed master
ssh pi@<failed-master-ip>
sudo /usr/local/bin/k3s-uninstall.sh

# Rejoin cluster
ansible-playbook site.yml --tags k3s-server --limit <failed-master>
```

## 🔧 Maintenance

### Cluster Updates

**🚀 Auto Updates (Recommended):**
```ini
# inventory/hosts.ini
[k3s_cluster:vars]
k3s_version=latest
```
```bash
ansible-playbook site.yml --tags k3s-server,k3s-agent
```

**🎯 Manual Version Update:**
```ini
# inventory/hosts.ini
k3s_version=v1.36.0+k3s1
```
```bash
# ⚠️ Update masters first!
ansible-playbook site.yml --tags k3s-server,k3s-agent
```

**📊 Check Versions:**
```bash
kubectl version --short
kubectl get nodes -o wide
ansible all -m shell -a "k3s --version" --become
```

**✅ Post-Update Verification:**
```bash
kubectl get nodes
kubectl get pods --all-namespaces
kubectl cluster-info
```

**🔄 Rollback if Needed:**
```bash
# Set previous version in inventory
k3s_version=v1.35.0+k3s1
ansible-playbook site.yml --tags k3s-server,k3s-agent
```

### Safe Reboots

**🔄 Full Cluster Reboot:**
```bash
ansible-playbook reboot.yml
```
*Reboots workers first, then masters (serially)*

**🎯 Selective Reboots:**
```bash
ansible-playbook reboot.yml --limit worker    # Workers only
ansible-playbook reboot.yml --limit master    # Masters only  
ansible-playbook reboot.yml --limit cm4-04     # Specific node
```

## 🐛 Troubleshooting

### Service Status
```bash
# Master nodes
sudo systemctl status k3s
sudo journalctl -u k3s -f

# Worker nodes  
sudo systemctl status k3s-agent
sudo journalctl -u k3s-agent -f
```

### Node Reset
```bash
# Reset server
/usr/local/bin/k3s-uninstall.sh

# Reset agent
/usr/local/bin/k3s-agent-uninstall.sh
```

### TLS / Certificate Issues

**Certificate not issued (stays at self-signed):**
```bash
# Check Traefik logs for ACME errors
kubectl logs -n kube-system deploy/traefik | grep -iE "acme|error|hetzner"

# Verify the Hetzner secret exists
kubectl get secret traefik-hetzner-token -n kube-system

# Verify the HelmChartConfig was applied
kubectl get helmchartconfig traefik -n kube-system
```

**Hetzner DNS API token errors:**
- Confirm the token was created in the **Hetzner DNS Console** (`dns.hetzner.com`), not Hetzner Cloud.
- Confirm the Hetzner DNS account actually holds the zone (`zlor.fi`) and that the domain's nameservers point at `ns1.first-ns.de` / `robotns2.second-ns.de` / `robotns3.second-ns.com` (or the NS set shown in the Hetzner zone).
- Confirm the token is correctly set in `.env` (no trailing whitespace or newlines).
- Re-run `ansible-playbook site.yml --tags traefik-config` after correcting the token.

**DNS-01 challenge times out:**
- Raise `hetzner_propagation_timeout` in `roles/traefik-config/defaults/main.yml` and re-run the role.
- Verify propagation manually: `dig +short TXT _acme-challenge.safe.zlor.fi @ns1.first-ns.de`

**Let's Encrypt rate limit hit:**
- Switch to the staging server in `roles/traefik-config/defaults/main.yml` (`traefik_acme_server`), re-run the role, verify the flow works, then switch back to production and delete `acme.json` to force re-issuance:
```bash
kubectl exec -n kube-system deploy/traefik -- rm /data/acme.json
kubectl rollout restart deploy/traefik -n kube-system
```

### Common Issues
- 🔥 **Nodes not joining**: Check firewall (port 6443)
- 💾 **Memory issues**: Verify cgroup memory enabled
- 🌐 **Network issues**: VXLAN backend optimized for ARM

## 🎛️ Customization

### Add More Masters
```ini
[master]
pi-master-1 ansible_host=192.168.30.100 ansible_user=pi
pi-master-2 ansible_host=192.168.30.101 ansible_user=pi
pi-master-3 ansible_host=192.168.30.102 ansible_user=pi
```

### Custom K3s Args
```ini
[k3s_cluster:vars]
extra_server_args="--flannel-backend=vxlan --disable traefik --disable servicelb"
extra_agent_args="--node-label foo=bar"
```

## 🖥️ Compute Blade Agent

**🔧 Hardware monitoring for Compute Blade systems**

### Components
- 🖥️ **compute-blade-agent**: Hardware monitoring daemon
- 🛠️ **bladectl**: CLI tool for agent interaction  
- ⚡ **fanunit.uf2**: Fan controller firmware

### Configuration
```ini
# Enable/disable in inventory/hosts.ini
enable_compute_blade_agent=true

# Per-node override
cm4-01 ansible_host=192.168.30.101 enable_compute_blade_agent=true
cm4-02 ansible_host=192.168.30.102 enable_compute_blade_agent=false
```

### Deployment
```bash
# Auto-deployed with main playbook
ansible-playbook site.yml

# Deploy only blade agent
ansible-playbook site.yml --tags compute-blade-agent
```

### Verification
```bash
# Check service status
sudo systemctl status compute-blade-agent
sudo journalctl -u compute-blade-agent -f

# Check binary
/usr/bin/compute-blade-agent --version
```

### Features
- 🌡️ **Hardware monitoring**: Temperature, fans, buttons
- 🚨 **Critical mode**: Auto max fan + red LED on overheating  
- 🔍 **Identification**: LED blade locator
- 📊 **Metrics**: Prometheus endpoint

### Monitoring Setup
```bash
# Deploy Prometheus monitoring
kubectl apply -f manifests/compute-blade-agent-daemonset.yaml
```

## 🌍 External DNS Setup

### DNS Configuration Options

#### 🏆 Option A: Virtual IP (Recommended)
```dns
test.zlor.fi  A  192.168.30.100  # MikroTik VIP
```
**Benefits:** Single IP, hardware failover, best performance

#### ⚖️ Option B: Multiple Records
```dns
test.zlor.fi  A  192.168.30.101
test.zlor.fi  A  192.168.30.102  
test.zlor.fi  A  192.168.30.103
test.zlor.fi  A  192.168.30.104
```
**Benefits:** Load balanced, auto failover

#### 🔧 Option C: Single Node
```dns
test.zlor.fi  A  192.168.30.101
```
**Benefits:** Simple, no failover

### Cluster DNS Configuration

**Configure DNS resolvers on all nodes:**
```bash
# Update systemd-resolved.conf
[Resolve]
DNS=8.8.8.8 8.8.4.4 192.168.1.1
FallbackDNS=8.8.8.8
DNSSECNegativeTrustAnchors=zlor.fi

# Restart service
sudo systemctl restart systemd-resolved
```

### Test External Access
```bash
# Test DNS resolution
nslookup test.zlor.fi

# Test HTTP access  
curl http://test.zlor.fi
curl -v http://test.zlor.fi

# Test from all cluster IPs
for ip in 192.168.30.{101..104}; do
  echo "Testing $ip:"
  curl -H "Host: test.zlor.fi" http://$ip
done
```

### Add More Domains
```yaml
# Update ingress with new hosts
spec:
  rules:
  - host: test.zlor.fi
  - host: api.zlor.fi  
  - host: admin.zlor.fi
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

## 🗑️ Uninstall

### Complete Cluster Removal
```bash
# Remove K3s from all nodes
ansible all -m shell -a "/usr/local/bin/k3s-uninstall.sh" --become
ansible workers -m shell -a "/usr/local/bin/k3s-agent-uninstall.sh" --become

# Remove compute-blade-agent
ansible worker -m shell -a "bash /usr/local/bin/k3s-uninstall-compute-blade-agent.sh" --become
```

## 📄 License

MIT License

## 🔗 References

- [📚 K3s Documentation](https://docs.k3s.io/)
- [🍓 K3s on Raspberry Pi](https://docs.k3s.io/installation/requirements)
- [📊 MikroTik VIP Setup](MIKROTIK-VIP-SETUP-CUSTOM.md)
- [🖥️ Compute Blade Agent](COMPUTE_BLADE_AGENT.md)

---

**🎉 Happy clustering!** 

*For issues or questions, check the troubleshooting section or refer to the documentation links above.*
