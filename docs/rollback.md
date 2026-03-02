# Wrangler Rollback Procedure

This document describes how to roll back a Cloudflare Worker deployment using Wrangler CLI.

## Quick Reference

```bash
# List recent deployments
wrangler deployments list

# Rollback to previous version
wrangler rollback

# Rollback to specific version
wrangler rollback --version <deployment-id>
```

## When to Rollback

Rollback should be performed when:
- A production deployment causes critical errors
- API endpoints are returning unexpected errors
- The Worker is unresponsive or timing out
- A security vulnerability is discovered in deployed code

## Pre-Rollback Checklist

1. **Verify the issue**: Confirm the issue is caused by the recent deployment, not an external dependency
2. **Check deployment history**: Review recent deployments to identify the stable version
3. **Notify team**: Alert relevant team members about the rollback
4. **Document the issue**: Record what went wrong for post-mortem

## Rollback Steps

### 1. List Recent Deployments

```bash
# List recent deployments for production
wrangler deployments list

# For staging environment
wrangler deployments list --env staging
```

Example output:
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Deployment ID: 373305c6-56f3-4fc7-8540-1e1e1e1e1e1e                          │
│ Created on:    2024-03-15T10:30:00.000000Z                                   │
│ Author:        ci-user                                                       │
│ Source:        Upload                                                        │
│ Version:       1.2.3                                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Identify Target Version

Find the deployment ID of the stable version you want to roll back to. This is typically the deployment immediately before the problematic one.

### 3. Execute Rollback

```bash
# Rollback to the most recent previous deployment
wrangler rollback

# Or specify a specific deployment
wrangler rollback --version <deployment-id>

# For staging environment
wrangler rollback --env staging --version <deployment-id>
```

### 4. Verify Rollback

```bash
# Check deployment status
wrangler deployments list

# Test the worker endpoint
curl https://spikes.sh/health

# Check logs for errors
wrangler tail --format json
```

### 5. Post-Rollback Actions

1. **Monitor**: Watch logs for any issues
2. **Fix the issue**: Address the root cause in a branch
3. **Test thoroughly**: Ensure the fix works in staging
4. **Re-deploy**: Create a new deployment with the fix

## Environment-Specific Rollback

### Production Rollback

```bash
# Production uses the default environment or --env production
wrangler rollback
# or
wrangler rollback --env production
```

### Staging Rollback

```bash
wrangler rollback --env staging
```

## D1 Database Considerations

**IMPORTANT**: Rollback only affects the Worker code, not the D1 database.

If a deployment included schema migrations:
1. **Do NOT rollback the Worker** if the migration cannot be reversed
2. Create a forward-fix migration instead
3. If the migration is reversible, run the down migration first:

```bash
# Apply down migration (if available)
wrangler d1 execute spikes-sh-db --file=migrations/down.sql
```

## R2 Bucket Considerations

Worker rollback does not affect R2 bucket contents. If assets were corrupted:
1. Restore from backup if available
2. Re-upload correct assets:

```bash
# Re-upload widget
wrangler r2 object put spikes-hosted-assets/widget/spikes.js \
  --file public/spikes.js --content-type application/javascript
```

## Emergency Contacts

If rollback fails or you need assistance:
1. Check Cloudflare status page
2. Review Cloudflare dashboard for alerts
3. Contact Cloudflare support if the issue is platform-related

## Version Control Integration

After rollback, ensure the git repository reflects the correct state:

```bash
# Tag the current stable version if not already tagged
git tag -a v1.2.2-stable -m "Stable version after rollback"

# Create a branch from the stable version for the fix
git checkout -b fix/deployment-issue v1.2.2-stable
```

## CI/CD Integration

After an emergency rollback, the next push to main will trigger a new staging deployment. To prevent re-deploying the broken version:

1. Fix the issue in a branch
2. Test in staging
3. Merge to main
4. Create a new release tag for production

## Monitoring Post-Rollback

After rollback, monitor for:
- Error rates in Worker logs (`wrangler tail`)
- Response times from health endpoints
- User reports of issues
- Stripe webhook delivery status
- D1 query performance
