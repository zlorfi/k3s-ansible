# Traefik Basic Authentication for Dashboard and API

## Overview

This guide explains how to enable basic authentication for the Traefik dashboard (`/dashboard`) and API (`/api`) endpoints in your Helm-deployed Traefik instance.

## Prerequisites

- Traefik deployed via Helm with IngressRoute enabled
- `htpasswd` utility (usually comes with Apache utilities)
- `kubectl` configured and access to your cluster

## Step 1: Generate Credentials

### Option 1: Using htpasswd (Recommended)

Generate a hashed password for your basic auth user:

```bash
# Install htpasswd if not already installed
# macOS:
brew install httpd

# Linux:
sudo apt-get install apache2-utils

# Create credentials file (replace 'admin' with your desired username)
htpasswd -c auth admin
# You'll be prompted to enter a password

# View the generated credentials
cat auth
# Output example: admin:$apr1$H6uskkkW$FTbVJriq9dMj0O/3YbCzC0
```

### Option 2: Using Python

```bash
python3 << 'EOF'
import bcrypt
import base64

username = "admin"
password = "your_secure_password"

# Hash password using bcrypt
hashed = bcrypt.hashpw(password.encode(), bcrypt.gensalt(rounds=12))

# Format for htpasswd
htpasswd_line = f"{username}:{hashed.decode()}"
print(htpasswd_line)
EOF
```

## Step 2: Create Kubernetes Secret

Once you have the credentials, create a Kubernetes Secret:

```bash
# Using the auth file from htpasswd
kubectl create secret generic traefik-basic-auth \
  --from-file=users=auth \
  -n traefik
```

Or create it directly without a file:

```bash
# Replace 'admin' and hashed_password with your actual values
kubectl create secret generic traefik-basic-auth \
  --from-literal=users='admin:$apr1$H6uskkkW$FTbVJriq9dMj0O/3YbCzC0' \
  -n traefik
```

### Verify Secret Creation

```bash
kubectl get secret traefik-basic-auth -n traefik
kubectl get secret traefik-basic-auth -n traefik -o yaml
```

## Step 3: Create BasicAuth Middleware

Create a Traefik BasicAuth middleware that references the secret:

```yaml
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: traefik-basic-auth
  namespace: kube-system
spec:
  basicAuth:
    secret: traefik-basic-auth
    removeHeader: true  # Remove Authorization header after auth
    headerField: Authorization  # Standard HTTP auth header
```

Save this to a file and apply it:

```bash
kubectl apply -f - <<'EOF'
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: traefik-basic-auth
  namespace: traefik
spec:
  basicAuth:
    secret: traefik-basic-auth
    removeHeader: true
EOF
```

## Step 4: Update IngressRoute with Middleware

Now update your Traefik IngressRoute to use the BasicAuth middleware:

```yaml
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: traefik-dashboard
  namespace: traefik
spec:
  entryPoints:
    - traefik
  routes:
    - match: PathPrefix(`/dashboard`) || PathPrefix(`/api`)
      kind: Rule
      services:
        - kind: TraefikService
          name: api@internal
      middlewares:
        - name: traefik-basic-auth
          namespace: traefik
  tls: {}
```

### Apply Updated IngressRoute

```bash
kubectl apply -f - <<'EOF'
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: traefik-dashboard
  namespace: traefik
spec:
  entryPoints:
    - traefik
  routes:
    - match: PathPrefix(`/dashboard`) || PathPrefix(`/api`)
      kind: Rule
      services:
        - kind: TraefikService
          name: api@internal
      middlewares:
        - name: traefik-basic-auth
          namespace: kube-system
  tls: {}
EOF
```

## Step 5: Test Basic Auth

### Test Dashboard Access

```bash
# Port-forward to Traefik
kubectl port-forward -n traefik svc/traefik 8080:8080 &

# Test without credentials (should fail with 401)
curl -i http://localhost:8080/dashboard/
# Output: HTTP/1.1 401 Unauthorized

# Test with correct credentials
curl -i -u admin:your_password http://localhost:8080/dashboard/
# Output: HTTP/1.1 200 OK

# Test with wrong credentials (should fail)
curl -i -u admin:wrong_password http://localhost:8080/dashboard/
# Output: HTTP/1.1 401 Unauthorized
```

### Test API Access

```bash
# Without auth
curl http://localhost:8080/api/http/routers

# With auth
curl -u admin:your_password http://localhost:8080/api/http/routers
```

## Using with Helm Chart Values

If you want to configure this permanently via Helm, add to your values.yaml:

```yaml
ingressRoute:
  dashboard:
    enabled: true
    entryPoints:
      - traefik
    matchRule: PathPrefix(`/dashboard`) || PathPrefix(`/api`)
    middlewares:
      - name: traefik-basic-auth
        namespace: traefik
    services:
      - kind: TraefikService
        name: api@internal
    tls: {}

# Create BasicAuth middleware as an extra object
extraObjects:
  - apiVersion: v1
    kind: Secret
    metadata:
      name: traefik-basic-auth
      namespace: traefik
    type: Opaque
    stringData:
      users: |
        admin:$apr1$H6uskkkW$FTbVJriq9dMj0O/3YbCzC0

  - apiVersion: traefik.io/v1alpha1
    kind: Middleware
    metadata:
      name: traefik-basic-auth
      namespace: traefik
    spec:
      basicAuth:
        secret: traefik-basic-auth
        removeHeader: true
```

Then update via Helm:

```bash
helm upgrade traefik traefik/traefik \
  -n traefik \
  -f values.yaml
```

## Multiple Users

To add multiple users to the basic auth secret:

```bash
# Create auth file with multiple users
htpasswd -c auth admin
htpasswd -B auth user2
htpasswd -B auth user3

# View the file
cat auth
# Output:
# admin:$apr1$H6uskkkW$FTbVJriq9dMj0O/3YbCzC0
# user2:$2y$05$...
# user3:$2y$05$...

# Create secret
kubectl create secret generic traefik-basic-auth \
  --from-file=users=auth \
  -n traefik \
  -o yaml --dry-run=client | kubectl apply -f -
```

## Rotate Credentials

To update credentials without recreating the middleware:

```bash
# Delete old secret
kubectl delete secret traefik-basic-auth -n traefik

# Create new secret with updated credentials
htpasswd -c auth admin  # Enter new password
kubectl create secret generic traefik-basic-auth \
  --from-file=users=auth \
  -n kube-system
```

Traefik will automatically reload the secret without needing a pod restart.

## Security Best Practices

1. **Use Strong Passwords**: Generate strong random passwords
   ```bash
   openssl rand -base64 16
   # Generate: dRw3k8F9P2mL7qX1nV4bZ6
   ```

2. **Use HTTPS/TLS**: Configure TLS for the dashboard in production
   ```yaml
   tls:
     secretName: traefik-tls  # Your TLS certificate secret
   ```

3. **Restrict Access**: Use IP-based access restrictions
   ```yaml
   middlewares:
     - name: traefik-basic-auth
       namespace: kube-system
     - name: ip-whitelist
       namespace: kube-system
   ---
   apiVersion: traefik.io/v1alpha1
   kind: Middleware
   metadata:
     name: ip-whitelist
     namespace: kube-system
   spec:
     ipWhiteList:
       sourceRange:
         - 192.168.1.0/24  # Your IP range
   ```

4. **Rotate Credentials Regularly**: Update passwords every 90 days

5. **Use Network Policies**: Restrict access to Traefik API port
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: traefik-api-restrict
     namespace: kube-system
   spec:
     podSelector:
       matchLabels:
         app.kubernetes.io/name: traefik
     policyTypes:
       - Ingress
     ingress:
       - from:
           - namespaceSelector:
               matchLabels:
                 name: kube-system
         ports:
           - protocol: TCP
             port: 8080
   ```

## Troubleshooting

### Secret not found error

```
Error: secret "traefik-basic-auth" not found
```

**Solution:**
```bash
# Verify secret exists in correct namespace
kubectl get secret traefik-basic-auth -n kube-system

# If missing, recreate it
kubectl create secret generic traefik-basic-auth \
  --from-file=users=auth \
  -n traefik
```

### 401 Unauthorized even with correct credentials

**Possible causes:**
- Secret format is wrong
- Middleware not applied to IngressRoute
- Password hash algorithm mismatch

**Debug:**
```bash
# Check secret content
kubectl get secret traefik-basic-auth -n traefik -o yaml

# Check IngressRoute
kubectl get ingressroute traefik-dashboard -n traefik -o yaml

# Check Traefik logs
kubectl logs -n traefik -l app.kubernetes.io/name=traefik -f | grep -i auth
```

### Htpasswd format issues

Valid formats:
- **APR1**: `$apr1$...` (from `htpasswd -b`)
- **Bcrypt**: `$2y$...` (from `htpasswd -B`)
- **SHA**: `{SHA}...` (from `htpasswd -s`)

Use APR1 or Bcrypt for best compatibility.

## Quick Reference Commands

```bash
# Generate credentials
htpasswd -c auth admin

# Create secret
kubectl create secret generic traefik-basic-auth --from-file=users=auth -n traefik

# Create middleware
kubectl apply -f - <<'EOF'
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: traefik-basic-auth
  namespace: traefik
spec:
  basicAuth:
    secret: traefik-basic-auth
    removeHeader: true
EOF

# Update IngressRoute
kubectl patch ingressroute traefik-dashboard -n traefik --type merge \
  -p '{"spec":{"routes":[{"middlewares":[{"name":"traefik-basic-auth","namespace":"traefik"}]}]}}'

# Test access
curl -u admin:password http://localhost:8080/dashboard/

# Verify middleware is applied
kubectl get middleware -n traefik
kubectl describe middleware traefik-basic-auth -n traefik
```

## References

- [Traefik BasicAuth Middleware](https://doc.traefik.io/traefik/middlewares/http/basicauth/)
- [Traefik Helm Chart](https://github.com/traefik/traefik-helm-chart)
- [Kubernetes Secrets Documentation](https://kubernetes.io/docs/concepts/configuration/secret/)