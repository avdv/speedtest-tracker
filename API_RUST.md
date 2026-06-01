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

**Authentication Required:** All API v1 endpoints require a valid Bearer token in the Authorization header.

### Authentication

API v1 endpoints use Laravel Sanctum-compatible token authentication. Include your API token in the Authorization header:

```bash
Authorization: Bearer YOUR_API_TOKEN
```

**Creating an API Token:**
1. Log in to the web interface
2. Navigate to `/admin/api-tokens`
3. Create a new token with the required abilities
4. Copy the generated token (shown only once!)

**Example Request:**
```bash
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
  http://localhost:3000/api/v1/results
```

**Unauthorized Response:**
```json
{
  "message": "Unauthenticated."
}
```

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
    "download": {
      "avg": 23071216,
      "avg_bits": 184569725,
      "avg_bits_human": "184.57 Mbps",
      "max": 24506775,
      "max_bits": 196054200,
      "max_bits_human": "196.05 Mbps",
      "min": 3493746,
      "min_bits": 27949968,
      "min_bits_human": "27.95 Mbps"
    },
    "ping": {
      "avg": 17.65,
      "max": 133.93,
      "min": 3.37
    },
    "total_results": 2422,
    "upload": {
      "avg": 4756434,
      "avg_bits": 38051473,
      "avg_bits_human": "38.05 Mbps",
      "max": 4971944,
      "max_bits": 39775552,
      "max_bits_human": "39.78 Mbps",
      "min": 986955,
      "min_bits": 7895640,
      "min_bits_human": "7.90 Mbps"
    }
  },
  "message": "ok"
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
# Health check (public)
curl http://localhost:3000/api/healthcheck

# Get latest result (legacy, public)
curl http://localhost:3000/api/speedtest/latest

# List results (requires auth)
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
  http://localhost:3000/api/v1/results

# Get latest result (requires auth)
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
  http://localhost:3000/api/v1/results/latest

# Get specific result (requires auth)
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
  http://localhost:3000/api/v1/results/1

# Get statistics (requires auth)
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
  http://localhost:3000/api/v1/stats

# List Ookla servers (requires auth)
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
  http://localhost:3000/api/v1/ookla/list-servers
```

## Notes

- All download/upload values in Mbps for legacy endpoint
- Download/upload in bytes for v1 endpoints (matching PHP version)
- **Authentication:** API v1 endpoints require Bearer token authentication using Laravel Sanctum-compatible tokens
- Token validation uses SHA-256 hashing (same as PHP Sanctum implementation)
- Tokens are validated against the `personal_access_tokens` table
- Token expiration and last_used_at tracking is supported
- Public endpoints: `/api/healthcheck` and `/api/speedtest/latest`
- Filtering and sorting not yet implemented (TODO)
