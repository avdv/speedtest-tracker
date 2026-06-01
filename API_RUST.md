# API Endpoints (Rust Implementation)

## Health Check
```bash
GET /api/healthcheck
```
Returns server health status.

**Response:**
```json
{
  "message": "Speedtest Tracker is running!"
}
```

## Legacy Endpoint (v0)
```bash
GET /api/speedtest/latest
```
Backwards compatible endpoint for Homepage and Organizr dashboards.

**Response:**
```json
{
  "message": "ok",
  "data": {
    "id": 1,
    "ping": 12.5,
    "download": 100.5,
    "upload": 50.2,
    "scheduled": true,
    "failed": false,
    "created_at": "2026-05-31T18:33:02",
    "updated_at": "2026-05-31T18:33:02"
  }
}
```

## API v1 Endpoints

### List Results
```bash
GET /api/v1/results?page=1&per_page=25
```
Get paginated list of all speedtest results.

**Query Parameters:**
- `page` (optional, default: 1) - Page number
- `per_page` (optional, default: 25) - Results per page

**Response:**
```json
{
  "data": [
    {
      "id": 1,
      "service": "ookla",
      "ping": 12.5,
      "download": 123456789,
      "upload": 12345678,
      "status": "completed",
      "scheduled": true,
      "created_at": "2026-05-31 18:33:02",
      "updated_at": "2026-05-31 18:33:02"
    }
  ],
  "page": 1,
  "per_page": 25,
  "total": 100
}
```

### Get Latest Result
```bash
GET /api/v1/results/latest
```
Get the most recent speedtest result.

**Response:**
```json
{
  "data": {
    "id": 1,
    "service": "ookla",
    "ping": 12.5,
    "download": 123456789,
    "upload": 12345678,
    "status": "completed",
    "scheduled": true,
    "created_at": "2026-05-31 18:33:02",
    "updated_at": "2026-05-31 18:33:02"
  },
  "message": "Success"
}
```

### Get Single Result
```bash
GET /api/v1/results/{id}
```
Get a specific speedtest result by ID.

**Response:**
```json
{
  "data": {
    "id": 1,
    "service": "ookla",
    "ping": 12.5,
    "download": 123456789,
    "upload": 12345678,
    "status": "completed",
    "scheduled": true,
    "created_at": "2026-05-31 18:33:02",
    "updated_at": "2026-05-31 18:33:02"
  },
  "message": "Success"
}
```

### Get Statistics
```bash
GET /api/v1/stats
```
Get aggregated statistics for all speedtest results.

**Response:**
```json
{
  "data": {
    "total_results": 100,
    "avg_ping": 12.5,
    "avg_download": 150.5,
    "avg_upload": 75.2,
    "min_ping": 8.0,
    "min_download": 50.0,
    "min_upload": 20.0,
    "max_ping": 25.0,
    "max_download": 300.0,
    "max_upload": 150.0
  },
  "message": "Success"
}
```

### List Ookla Servers
```bash
GET /api/v1/ookla/list-servers
```
Get a list of available Ookla speedtest servers.

**Response:**
```json
{
  "data": [
    {
      "id": 12345,
      "host": "speedtest.example.com",
      "name": "Example ISP",
      "location": "New York, NY",
      "country": "United States"
    }
  ],
  "message": "Speedtest servers fetched successfully."
}
```

## Testing

Start the server:
```bash
cargo run
```

Test endpoints:
```bash
# Health check
curl http://localhost:3000/api/healthcheck

# Get latest result (legacy)
curl http://localhost:3000/api/speedtest/latest

# List results
curl http://localhost:3000/api/v1/results

# Get latest result
curl http://localhost:3000/api/v1/results/latest

# Get specific result
curl http://localhost:3000/api/v1/results/1

# Get statistics
curl http://localhost:3000/api/v1/stats

# List Ookla servers
curl http://localhost:3000/api/v1/ookla/list-servers
```

## Notes

- All download/upload values in Mbps for legacy endpoint
- Download/upload in bytes for v1 endpoints (matching PHP version)
- No authentication implemented yet (TODO: add Sanctum token validation)
- Filtering and sorting not yet implemented (TODO)
