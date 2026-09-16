# Ironcage Operational Deployment & Backstage Integration

## Production Setup (Oracle Free Tier 2 OCPU / 12 GB)

### 1. Deploy Infrastructure

```bash
# Load Docker images from CI artifacts
./bin/deploy-images.sh

# Push to Docker Hub
docker push YOUR_USER/ironcage-kernel:v0.1.0
docker push YOUR_USER/ironcage-api:v0.1.0

# Update k8s/deployment.yaml image references
sed -i 's/docker.io\/chidionyema/docker.io\/YOUR_USER/g' k8s/deployment.yaml

# Deploy to K8s
kubectl apply -f k8s/deployment.yaml

# Verify ready
kubectl get pods -n ironcage -o wide
```

### 2. Backstage Integration

**Register with Backstage:**

```bash
# Copy catalog-info.yaml to Backstage locations repo
# Add to your Backstage config:
catalog:
  locations:
    - type: github
      target: https://github.com/chidionyema/ironcage/blob/main/catalog-info.yaml
```

**Access Research Component:**
- Navigate to Backstage catalog
- Search: "ironcage-research"
- Click component to view:
  - Repository link
  - Deployed API endpoint  
  - Real-time service status

### 3. Conduct Research

**Via Ironcage UI (10/10 UX):**

1. Port-forward API:
```bash
kubectl port-forward -n ironcage svc/ironcage-api 3001:3001 &
```

2. Open UI: http://localhost:3001

3. Submit research claim in form:
   - **Claim**: "P=NP via X approximation..."
   - **Domain**: math (optional)
   - Click **Submit Claim**

4. Watch real-time MCTS tree:
   - Graph shows exploration nodes
   - Green = verified, Red = unverified
   - Click nodes for details

**Via API directly:**

```bash
curl -X POST http://localhost:3001/research/claim \
  -H "Content-Type: application/json" \
  -d '{
    "claim": "Riemann Hypothesis...",
    "evidence": "Proof sketch...",
    "domain": "math"
  }'
```

**WebSocket live updates:**

```bash
wscat -c ws://localhost:3001/ws
# Streams: {"node_id": "...", "visits": 123, "reward": 0.85, "verified": true}
```

### 4. Pipeline Execution

**Full Research Workflow:**

1. **Hypothesis Submission** → claim recorded to ledger
2. **Evidence Generation** → inference engine generates candidates
3. **Formal Verification** → Z3 verifier checks claim
4. **Tree Exploration** → MCTS expands high-confidence branches
5. **Tamper-Evident Record** → SHA3-256 ledger immutably stores results

### 5. Monitoring & Maintenance

**Health Check:**
```bash
curl http://localhost:3001/health
# Returns: "ok"
```

**Metrics (Prometheus format):**
```bash
curl http://localhost:3001/metrics
# Returns: ironcage_api_version{} 1
```

**View Ledger:**
```bash
kubectl exec -it -n ironcage <pod-name> -- sqlite3 /ledger/research.db \
  "SELECT * FROM ledger LIMIT 10;"
```

### 6. Troubleshooting

**Pods not starting:**
```bash
kubectl describe pod -n ironcage <pod-name>
# Check: ImagePullBackOff → update image references
#        CrashLoopBackOff → check logs
```

**No WebSocket connection:**
```bash
kubectl logs -n ironcage <api-pod> | grep -i websocket
# Verify network policy and firewall
```

**Verification failures:**
```bash
# Check Z3 solver output in logs
kubectl logs -n ironcage <kernel-pod> | grep -i "verification"
```

## Scale-Out Strategy

For production research at scale:

1. **Multi-region**: Deploy additional Ironcage instances per research domain
2. **Sharded ledger**: Split by claim domain for parallel verification  
3. **Caching layer**: Add Redis for frequent claim results
4. **Analytics**: Connect to data warehouse for research trends

## Security Hardening

✅ Already configured:
- Non-root containers (uid 65532)
- Read-only root filesystem
- Drop ALL capabilities  
- seccomp RuntimeDefault
- K8s network policies via Kyverno
- SHA3-256 immutable ledger

Additional steps:
- Enable audit logging
- Configure RBAC roles
- Set resource quotas per namespace
- Monitor pod logs for anomalies

## Support & Operations

**On-call Procedures:**
- Monitor metrics dashboard in Backstage
- Alert on failed verification rates > 10%
- Check ledger for integrity violations
- Escalate to research team if P=NP detection triggered 🔥

**Operational Runbooks:**
See `/docs/runbooks/` for detailed procedures:
- `ledger-recovery.md` - Handle ledger corruption
- `kernel-restart.md` - Graceful kernel shutdown
- `scaling.md` - Horizontal pod autoscaling
