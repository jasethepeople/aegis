# Aegis API Reference

## Authentication

### POST /api/v1/auth/login
Login and receive JWT token.

**Request:**
```json
{ "email": "user@example.com", "password": "password" }
```

**Response:**
```json
{ "token": "eyJhbGciOiJIUzI1NiIs..." }
```

## Campaigns

### GET /api/v1/campaigns
List all campaigns.

**Response:**
```json
{ "campaigns": [{ "id": "...", "name": "...", "status": "running", ... }] }
```

### POST /api/v1/campaigns
Create a new campaign.

**Request:**
```json
{ "name": "test", "target_id": "...", "config": { "parallel_workers": 4 } }
```

### GET /api/v1/campaigns/:id
Get campaign details.

### POST /api/v1/campaigns/:id/start
Start a campaign.

### POST /api/v1/campaigns/:id/pause
Pause a running campaign.

### POST /api/v1/campaigns/:id/stop
Stop a campaign.

## Targets

### GET /api/v1/targets
List all targets.

### POST /api/v1/targets
Create a new target.

**Request:**
```json
{ "name": "binary", "type": "binary", "command": "./harness", "input_mode": "stdin" }
```

## Workers

### GET /api/v1/workers
List all workers.

## Crashes

### GET /api/v1/crashes?campaign_id=...
List crashes for a campaign.

## Dashboard

### GET /api/v1/dashboard
Get dashboard statistics.

**Response:**
```json
{ "total_campaigns": 10, "active_campaigns": 3, "total_workers": 5, "active_workers": 4, "total_crashes": 12, "unique_crashes": 8 }
```
