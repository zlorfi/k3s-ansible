# MikroTik Virtual IP Setup for K3s Cluster

This guide shows how to configure a Virtual IP on your MikroTik router for high availability across your k3s cluster nodes.

## Overview

Instead of using Keepalived on cluster nodes, MikroTik's built-in VIP feature handles:
- Single virtual IP address (192.168.30.100)
- Automatic failover between cluster nodes
- Health monitoring
- No additional software needed on cluster nodes

## Prerequisites

- MikroTik RouterOS with VIP support (RouterOS 6.x or newer)
- SSH access to MikroTik router
- K3s cluster nodes already deployed
- Master node IP: 192.168.30.101
- Worker node IPs: 192.168.30.102, 192.168.30.103, 192.168.30.104

## MikroTik VIP Configuration

### Option 1: Web Interface (easiest)

1. **Access MikroTik Web Interface**
   - Open `http://<router-ip>:80` in browser
   - Login with admin credentials

2. **Navigate to VIP settings**
   - Go to: **IP** → **Firewall** → **NAT**
   - Or: **IP** → **Addresses** (for simple VIP without NAT)

3. **Create NAT Rule with VIP** (if you want port forwarding)
   - Click **+ New** in NAT rules
   - Set:
     - **Chain**: `dstnat`
     - **In. Interface**: your WAN/LAN interface
     - **Dst. Address**: Your Virtual IP (192.168.30.100)
     - **Protocol**: `tcp/udp`
     - **Dst. Port**: `80` (for HTTP)
     - **To Addresses**: 192.168.30.101 (primary/master)
     - **To Ports**: 80
   - Click **Apply**

4. **Add IP Address on Router** (make router own the VIP)
   - Go to: **IP** → **Addresses**
   - Click **+ New**
   - Set:
     - **Address**: `192.168.30.100/24`
     - **Interface**: Your LAN interface (e.g., `ether2`)
     - **Comment**: `K3s-Cluster-VIP`
   - Click **OK**

### Option 2: Command Line (via SSH)

Connect to your MikroTik router via SSH:

```bash
ssh admin@<mikrotik-ip>
```

#### Add VIP Address on Router

```mikrotik
/ip/address/add address=192.168.30.100/24 interface=ether2 comment="K3s-Cluster-VIP"
```

Replace `ether2` with your LAN interface name.

#### Create NAT Rule for HTTP (if needed)

```mikrotik
/ip/firewall/nat/add chain=dstnat dst-address=192.168.30.100 protocol=tcp \
  dst-port=80 action=dstnat to-addresses=192.168.30.101 to-ports=80 \
  comment="K3s-VIP-HTTP"
```

#### Create NAT Rule for HTTPS (if needed)

```mikrotik
/ip/firewall/nat/add chain=dstnat dst-address=192.168.30.100 protocol=tcp \
  dst-port=443 action=dstnat to-addresses=192.168.30.101 to-ports=443 \
  comment="K3s-VIP-HTTPS"
```

### Option 3: RouterOS Script (Automated)

Create a script on MikroTik to manage VIP and failover.

#### Upload Script to RouterOS

```mikrotik
/system script add name=k3s-health-check source={
  # Check if master is reachable
  :local masterIP "192.168.30.101"
  :local workerIPs ("192.168.30.102" "192.168.30.103" "192.168.30.104")
  :local activeIP $masterIP

  # Try to ping master
  :if ([ping $masterIP count=1] = 0) do={
    :put "Master down, checking workers..."

    # Try each worker
    :foreach worker in=$workerIPs do={
      :if ([ping $worker count=1] > 0) do={
        :set $activeIP $worker
        :put "Worker $worker is up, using as backup"
        :break
      }
    }
  }

  # Update NAT rule to point to active IP
  /ip/firewall/nat/set [find comment~"K3s-VIP"] to-addresses=$activeIP
}
```

#### Schedule Health Check

```mikrotik
/system scheduler add name=k3s-health-check-task \
  on-event=k3s-health-check interval=10s comment="Monitor K3s cluster health"
```

## DNS Configuration

After setting up MikroTik VIP, configure your DNS:

```dns
test.zlor.fi  A  192.168.30.100
```

## Verification

### Test VIP from Your Machine

```bash
# Verify VIP is reachable
ping 192.168.30.100

# Test HTTP access
curl -v http://192.168.30.100

# Test with domain name
curl -v http://test.zlor.fi
```

### Test from MikroTik

SSH into the router and test:

```mikrotik
/tool ping 192.168.30.100 count=5
/tool http-request url=http://192.168.30.100
```

### Verify NAT Rules

Check that NAT rules are correctly configured:

```bash
# SSH to MikroTik
/ip/firewall/nat/print detail
```

## How It Works

### Traffic Flow

```
Internet/External Client
        ↓
   DNS Resolution
   test.zlor.fi → 192.168.30.100
        ↓
  MikroTik Router (VIP 192.168.30.100)
        ↓
   NAT Rule Routes Traffic
        ↓
   Primary: 192.168.30.101 (Master)
   Backup: 192.168.30.102 (if master down)
        ↓
   K3s Cluster
```

### High Availability

- **Default**: All traffic goes to 192.168.30.101 (master)
- **If Master Down**: Health check detects failure
- **Automatic Failover**: NAT rule updates to point to worker node
- **No DNS Changes**: VIP remains the same

## Comparison: MikroTik VIP vs Keepalived

| Feature | MikroTik VIP | Keepalived |
|---------|--------------|-----------|
| **Location** | Router (hardware) | Cluster nodes (software) |
| **Complexity** | Simple | Moderate |
| **Additional Software** | None | Keepalived daemon |
| **Failover Speed** | <1 second | <5 seconds |
| **Health Checks** | Configurable | API server only |
| **Router Dependency** | Required | Not needed |
| **Setup Time** | 5 minutes | 10 minutes |
| **Best For** | Home/small labs | Enterprise |

## Troubleshooting

### VIP Not Responding

1. Check if VIP is added on router:
```mikrotik
/ip/address/print detail where comment~"VIP"
```

2. Verify NAT rules:
```mikrotik
/ip/firewall/nat/print detail where comment~"K3s"
```

3. Check if cluster nodes are reachable:
```bash
for ip in 192.168.30.{101..104}; do
  echo "Testing $ip:"
  ping -c 1 $ip
done
```

### Failover Not Working

1. Check router's health check script:
```mikrotik
/system script print
/system script run k3s-health-check
```

2. View script logs:
```mikrotik
/system script job print
```

3. Test manual failover:
```bash
# Stop the master node temporarily
ssh pi@192.168.30.101 "sudo shutdown -h now"

# Wait 30 seconds, then test if failover occurred
curl http://192.168.30.100

# Boot master back up
# Power on the master node
```

### DNS Not Resolving

1. Verify DNS zone file has the correct A record:
```dns
test.zlor.fi  A  192.168.30.100
```

2. Test DNS resolution:
```bash
nslookup test.zlor.fi
dig test.zlor.fi
```

3. Flush DNS cache if needed:
```bash
# On macOS
sudo dscacheutil -flushcache

# On Linux
sudo systemctl restart systemd-resolved
```

## Removing MikroTik VIP

If you no longer need the VIP:

### Via Web Interface

1. Go to **IP** → **Addresses**
2. Find the VIP address (192.168.30.100)
3. Select it and click **Remove**

4. Go to **IP** → **Firewall** → **NAT**
5. Find rules with "K3s-VIP" in comment
6. Select and click **Remove**

### Via Command Line

```mikrotik
# Remove VIP address
/ip/address/remove [find comment~"K3s-Cluster-VIP"]

# Remove NAT rules
/ip/firewall/nat/remove [find comment~"K3s-VIP"]

# Remove health check script (optional)
/system script remove [find name="k3s-health-check"]
/system scheduler remove [find name="k3s-health-check-task"]
```

## Alternative: ECMP (Equal-Cost Multi-Path) Routing

If you want true load balancing instead of failover, MikroTik also supports ECMP routing.

This distributes traffic equally across all cluster nodes:

```mikrotik
/ip/route/add dst-address=0.0.0.0/0 gateway=192.168.30.101
/ip/route/add dst-address=0.0.0.0/0 gateway=192.168.30.102
/ip/route/add dst-address=0.0.0.0/0 gateway=192.168.30.103
/ip/route/add dst-address=0.0.0.0/0 gateway=192.168.30.104
```

Note: ECMP requires more advanced configuration and works better for internal load balancing.

## Best Practice Recommendations

1. **Use MikroTik VIP for external traffic**
   - Simple and reliable
   - Configured at network edge

2. **Keep Keepalived disabled**
   - Use `/ip/firewall/nat` for VIP
   - No need for cluster-level HA at this point

3. **Monitor the VIP**
   - Test failover monthly
   - Check NAT rule logs

4. **Document the configuration**
   - Export RouterOS config: **Files** → **System** → **Backup**
   - Save a copy of NAT rules for reference

## Support

For more MikroTik documentation:
- [MikroTik NAT Documentation](https://wiki.mikrotik.com/wiki/Manual:IP/Firewall/NAT)
- [MikroTik IP Addresses](https://wiki.mikrotik.com/wiki/Manual:IP/Address)
- [MikroTik Health Check Scripts](https://wiki.mikrotik.com/wiki/Manual:System/Health)
