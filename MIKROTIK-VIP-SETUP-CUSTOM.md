# MikroTik Virtual IP Setup for Your K3s Cluster

Customized setup guide for your MikroTik RouterOS configuration.

## Your Current Setup

```
Uplink Network:     192.168.1.0/24  (br-uplink - WAN/External)
LAB Network:        192.168.30.0/24 (br-lab - K3s Cluster)

K3s Nodes:
  cm4-01: 192.168.30.101 (Master)
  cm4-02: 192.168.30.102 (Worker)
  cm4-03: 192.168.30.103 (Worker)
  cm4-04: 192.168.30.104 (Worker)

Virtual IP to Create:
  192.168.30.100/24 (on br-lab bridge)
```

## Step 1: Add Virtual IP Address on MikroTik

Since your K3s nodes are on the `br-lab` bridge, add the VIP there:

### Via Command Line (Recommended)

```mikrotik
/ip/address/add address=192.168.30.100/24 interface=br-lab comment="K3s-Cluster-VIP"
```

### Verify VIP is Added

```mikrotik
/ip/address/print detail where comment~"K3s"
```

You should see:
```
0   address=192.168.30.100/24 interface=br-lab disabled=no comment="K3s-Cluster-VIP"
```

## Step 2: Create NAT Rules for Traffic Routing

Your VIP will route traffic to the master node by default. Create NAT rules to handle port 80 (HTTP) and 443 (HTTPS).

### HTTP (Port 80)

```mikrotik
/ip/firewall/nat/add chain=dstnat dst-address=192.168.30.100 dst-port=80 protocol=tcp action=dstnat to-addresses=192.168.30.101 to-ports=80 comment="K3s-VIP-HTTP"
```

### HTTPS (Port 443)

```mikrotik
/ip/firewall/nat/add chain=dstnat dst-address=192.168.30.100 dst-port=443 protocol=tcp action=dstnat to-addresses=192.168.30.101 to-ports=443 comment="K3s-VIP-HTTPS"
```

### Verify NAT Rules

```mikrotik
/ip/firewall/nat/print detail where comment~"K3s-VIP"
```

## Step 3: Add Static Routes (Optional but Recommended)

Ensure the K3s cluster nodes can reach each other through br-lab:

```mikrotik
/ip/route/add dst-address=192.168.30.0/24 gateway=192.168.30.1 \
  comment="K3s-Cluster-Network"
```

## Step 4: Configure Firewall Rules

Make sure your firewall allows traffic on ports 80 and 443 to the VIP:

### Allow Ingress to VIP on Port 80

```mikrotik
/ip/firewall/filter/add \
  chain=forward \
  dst-address=192.168.30.100 \
  dst-port=80 \
  protocol=tcp \
  action=accept \
  comment="Allow-HTTP-to-VIP"
```

### Allow Ingress to VIP on Port 443

```mikrotik
/ip/firewall/filter/add \
  chain=forward \
  dst-address=192.168.30.100 \
  dst-port=443 \
  protocol=tcp \
  action=accept \
  comment="Allow-HTTPS-to-VIP"
```

## Step 5: Test the VIP

### Test from MikroTik Router

```mikrotik
/tool/ping 192.168.30.100 count=5
```

Expected output: All 5 pings should succeed

### Test from Your Local Machine

```bash
# Ping the VIP
ping 192.168.30.100

# Test HTTP access
curl http://192.168.30.100

# Test HTTPS access (ignore cert warnings for now)
curl -k https://192.168.30.100
```

### Test from a Cluster Node

```bash
# SSH to any cluster node
ssh pi@192.168.30.101

# From the node, test local connectivity
curl http://192.168.30.100
```

## Step 6: Configure DNS

Add an A record in your DNS server:

```dns
test.zlor.fi  A  192.168.30.100
```

## Step 7: Verify End-to-End

```bash
# Test DNS resolution
nslookup test.zlor.fi
dig test.zlor.fi

# Test HTTP access via domain
curl http://test.zlor.fi

# Test HTTPS access via domain
curl -k https://test.zlor.fi
```

## Step 8: Optional - Add Health Check Script

For automatic failover, create a health check script that monitors the master node and updates NAT rules if it goes down.

### Create Health Check Script

```mikrotik
/system/script/add name=k3s-vip-health-check source={
  :local masterIP "192.168.30.101"
  :local vipAddr "192.168.30.100"
  :local allWorkers {192.168.30.102; 192.168.30.103; 192.168.30.104}

  # Log current status
  :put "[$(date)] Checking K3s cluster health..."

  # Check if master is reachable
  :if ([/ping address=$masterIP count=2] > 0) do={
    :put "[$(date)] Master $masterIP is UP - keeping NAT rules as-is"
  } else={
    :put "[$(date)] Master $masterIP is DOWN - checking workers..."

    # Find first available worker
    :local backupIP ""
    :foreach worker in=$allWorkers do={
      :if ([/ping address=$worker count=2] > 0) do={
        :set $backupIP $worker
        :put "[$(date)] Using worker $backupIP as backup"
        :break
      }
    }

    # If a worker is available, update NAT rules
    :if ($backupIP != "") do={
      /ip/firewall/nat/set [find comment~"K3s-VIP-HTTP"] to-addresses=$backupIP
      /ip/firewall/nat/set [find comment~"K3s-VIP-HTTPS"] to-addresses=$backupIP
      :put "[$(date)] NAT rules updated to point to $backupIP"
    } else={
      :put "[$(date)] ERROR: No worker nodes available!"
    }
  }
}
```

### Schedule Health Check

```mikrotik
/system/scheduler/add \
  name=k3s-vip-health-check \
  on-event=k3s-vip-health-check \
  interval=30s \
  disabled=no \
  comment="Monitor K3s cluster and update VIP routes"
```

### View Health Check Logs

```mikrotik
/system/script/job/print
/log/print where topics~"k3s"
```

## Verification Checklist

- [ ] VIP address (192.168.30.100) added to br-lab
- [ ] NAT rules for port 80 and 443 created
- [ ] Firewall rules allow traffic to VIP
- [ ] Ping 192.168.30.100 succeeds
- [ ] curl http://192.168.30.100 returns nginx page
- [ ] DNS A record added: test.zlor.fi → 192.168.30.100
- [ ] curl http://test.zlor.fi works
- [ ] Health check script created (optional)
- [ ] Health check scheduled (optional)

## Troubleshooting

### VIP Not Reachable

1. Verify VIP is on correct interface:
```mikrotik
/ip/address/print detail where address~"192.168.30.100"
```

2. Verify NAT rules exist:
```mikrotik
/ip/firewall/nat/print detail where comment~"K3s-VIP"
```

3. Check firewall rules are not blocking:
```mikrotik
/ip/firewall/filter/print detail where comment~"VIP\|K3s"
```

4. Check br-lab bridge is up:
```mikrotik
/interface/bridge/print detail where name=br-lab
```

### Failover Not Working

1. Check health check script is running:
```mikrotik
/system/scheduler/print where name~"k3s-vip"
/system/script/job/print
```

2. Run health check manually:
```mikrotik
/system/script/run k3s-vip-health-check
```

3. View logs:
```mikrotik
/log/print where topics~"k3s" or message~"K3s"
```

### Traffic Not Routing Correctly

1. Test NAT rule directly:
```bash
# From a machine on 192.168.1.0/24 network
curl -v http://192.168.30.100:80

# Check what port it's reaching on master
ssh pi@192.168.30.101
sudo netstat -tulpn | grep :80
```

2. Check if traffic is reaching the cluster:
```bash
# SSH to master and monitor traffic
ssh pi@192.168.30.101
sudo tcpdump -i eth0 'tcp port 80'

# Make a request from another machine
curl http://192.168.30.100
```

## Complete Command Sequence

If you want to run all commands in one go, here's the complete sequence:

```mikrotik
# Add VIP address
/ip/address/add address=192.168.30.100/24 interface=br-lab comment="K3s-Cluster-VIP"

# Add HTTP NAT rule
/ip/firewall/nat/add chain=dstnat dst-address=192.168.30.100 dst-port=80 protocol=tcp action=dstnat to-addresses=192.168.30.101 to-ports=80 comment="K3s-VIP-HTTP"

# Add HTTPS NAT rule
/ip/firewall/nat/add chain=dstnat dst-address=192.168.30.100 dst-port=443 protocol=tcp action=dstnat to-addresses=192.168.30.101 to-ports=443 comment="K3s-VIP-HTTPS"

# Add static route
/ip/route/add dst-address=192.168.30.0/24 gateway=192.168.30.1 comment="K3s-Cluster-Network"

# Verify
/ip/address/print detail
/ip/firewall/nat/print detail where comment~"K3s"
```

## Remove VIP (If Needed)

If you need to remove the VIP setup:

```mikrotik
# Remove VIP address
/ip/address/remove [find comment~"K3s-Cluster-VIP"]

# Remove NAT rules
/ip/firewall/nat/remove [find comment~"K3s-VIP"]

# Remove firewall filter rules
/ip/firewall/filter/remove [find comment~"VIP\|K3s"]

# Remove health check
/system/script/remove [find name="k3s-vip-health-check"]
/system/scheduler/remove [find name="k3s-vip-health-check"]
```

## Summary

Your VIP is now configured on MikroTik:

```
External Traffic
    ↓
192.168.30.100:80 (VIP on br-lab)
    ↓
NAT Rule Routes to 192.168.30.101:80
    ↓
K3s Master Node (cm4-01)
    ↓
If Master Down → Failover to Worker
    (Optional with health check script)
```

DNS: `test.zlor.fi → 192.168.30.100`

Single IP for your entire cluster with automatic failover! ✅
