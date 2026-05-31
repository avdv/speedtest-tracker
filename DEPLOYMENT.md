# Quick Deployment Guide

## For Your ARMv7 Device

### 1. Build the Binary

On your development machine:

```bash
# Install cross if you haven't
cargo install cross

# Build for ARMv7
cross build --release --target armv7-unknown-linux-gnueabihf
```

The binary will be at: `target/armv7-unknown-linux-gnueabihf/release/speedtest-admin`

### 2. Copy to Your Device

```bash
# Replace with your device details
scp target/armv7-unknown-linux-gnueabihf/release/speedtest-admin user@your-device:/usr/local/bin/
```

### 3. Set Up on Device

SSH to your device:

```bash
ssh user@your-device

# Make executable
chmod +x /usr/local/bin/speedtest-admin

# Create environment file
cat > /etc/speedtest-admin.env << 'ENVEOF'
DATABASE_URL=sqlite:/path/to/database/database.sqlite
PORT=3000
RUST_LOG=info
ENVEOF

# If using existing PHP database:
# DATABASE_URL=mysql://user:pass@localhost/speedtest
# or
# DATABASE_URL=postgresql://user:pass@localhost/speedtest
```

### 4. Create Systemd Service

```bash
sudo tee /etc/systemd/system/speedtest-admin.service << 'SERVICEEOF'
[Unit]
Description=Speedtest Tracker Admin (Rust)
After=network.target

[Service]
Type=simple
User=nobody
EnvironmentFile=/etc/speedtest-admin.env
ExecStart=/usr/local/bin/speedtest-admin
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SERVICEEOF
```

### 5. Start the Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable speedtest-admin
sudo systemctl start speedtest-admin

# Check status
sudo systemctl status speedtest-admin

# View logs
sudo journalctl -u speedtest-admin -f
```

### 6. Access the Admin Panel

Open your browser to: `http://your-device:3000`

## Using Existing Database

The Rust server can read from the same database as the PHP version:

```bash
# If PHP version used SQLite
DATABASE_URL=sqlite:/path/to/speedtest-tracker/database/database.sqlite

# If PHP version used MySQL
DATABASE_URL=mysql://username:password@localhost:3306/speedtest_tracker

# If PHP version used PostgreSQL
DATABASE_URL=postgresql://username:password@localhost:5432/speedtest_tracker
```

You can run both PHP and Rust servers against the same database (just use different ports).

## Firewall Setup

If you have a firewall:

```bash
# UFW
sudo ufw allow 3000/tcp

# iptables
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
```

## Reverse Proxy (Optional)

### Nginx

```nginx
server {
    listen 80;
    server_name speedtest.yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Caddy

```
speedtest.yourdomain.com {
    reverse_proxy localhost:3000
}
```

## Troubleshooting

**Port already in use:**
```bash
# Change port in environment
PORT=8080 /usr/local/bin/speedtest-admin
```

**Can't connect to database:**
```bash
# Test database connection
sqlite3 /path/to/database.sqlite "SELECT COUNT(*) FROM results;"
# or
mysql -u user -p -e "SELECT COUNT(*) FROM results;" speedtest_tracker
```

**Permission denied:**
```bash
# Check binary permissions
ls -la /usr/local/bin/speedtest-admin

# Check database file permissions (for SQLite)
ls -la /path/to/database/database.sqlite
```

## Binary Size Comparison

```
PHP Docker image: 500MB - 1GB
Rust binary: 5-10MB (200x smaller!)
```

## Memory Usage Comparison

```
PHP-FPM + Nginx: 100-200MB
Rust server: 10-20MB (10x more efficient!)
```

## Performance

The Rust server can handle thousands of requests per second on an ARMv7 device. For an admin interface that's accessed occasionally, this is massive overkill - but it means zero performance concerns.
