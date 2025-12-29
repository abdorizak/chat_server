# Chat Server Deployment Guide

## Local Docker Setup (Testing)

### Prerequisites
- Docker Desktop installed
- Docker Compose installed

### Step 1: Build and Run Locally

```bash
# 1. Make sure you're in the project directory
cd /Users/xman/Developer/chat_server

# 2. Build and start all services (DB + App)
docker compose up --build

# This will:
# - Build the Rust application
# - Start PostgreSQL database
# - Run migrations automatically
# - Expose app on port 8080
```

### Step 2: Test Locally

```bash
# In a new terminal, test the health endpoint
curl http://localhost:8080/health

# Test user registration
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123",
    "first_name": "Test",
    "last_name": "User"
  }'

# Test WebSocket connection
# Use a WebSocket client like wscat:
# npm install -g wscat
# wscat -c "ws://localhost:8080/ws?userId=1"
```

### Step 3: Access Database (Port Forwarding)

```bash
# The PostgreSQL database is accessible on localhost:5432
# Connect using any PostgreSQL client:

psql -h localhost -p 5432 -U postgres -d chat_db
# Password: postgres (from docker-compose.yml)

# Or use a GUI tool like:
# - pgAdmin: localhost:5432
# - DBeaver: localhost:5432
# - TablePlus: localhost:5432
```

---

## Production Server Deployment

### Option A: Docker Compose on Server

**1. Copy files to server:**
```bash
# On your local machine
scp -r /Users/xman/Developer/chat_server user@your-server-ip:/home/user/

# Or use rsync (better for updates)
rsync -avz --exclude 'target' \
  /Users/xman/Developer/chat_server/ \
  user@your-server-ip:/home/user/chat_server/
```

**2. On the server:**
```bash
# SSH into server
ssh user@your-server-ip

# Navigate to project
cd /home/user/chat_server

# Create production .env file
cp .env.example .env
nano .env  # Edit with production values

# Build and run
docker compose up -d --build

# Check logs
docker compose logs -f chat_server
```

**3. Configure firewall:**
```bash
# Allow HTTP/HTTPS and WebSocket
sudo ufw allow 8080/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### Option B: Separate Database Server (Recommended for Production)

**Server Architecture:**
- **Server 1 (Database):** PostgreSQL in Docker
- **Server 2 (App):** Chat server in Docker

**Database Server Setup:**
```bash
# On DB server
docker run -d \
  --name postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=STRONG_PASSWORD_HERE \
  -e POSTGRES_DB=chat_db \
  -p 5432:5432 \
  -v postgres_data:/var/lib/postgresql/data \
  --restart unless-stopped \
  postgres:16-alpine

# For security, only allow connections from app server
sudo ufw allow from APP_SERVER_IP to any port 5432
```

**App Server Setup:**
```bash
# Update .env to point to DB server
DATABASE_URL=postgresql://postgres:PASSWORD@DB_SERVER_IP:5432/chat_db

# Build and run app
docker build -t chat_server .
docker run -d \
  --name chat_server \
  -p 8080:8080 \
  --env-file .env \
  --restart unless-stopped \
  chat_server
```

---

## Port Forwarding & Database Access

### Local Development

**Access remote database from local machine:**
```bash
# SSH tunnel method (most secure)
ssh -L 5432:localhost:5432 user@your-server-ip

# Now you can connect to localhost:5432 on your machine
# and it will forward to the server's PostgreSQL
psql -h localhost -p 5432 -U postgres -d chat_db
```

**DBeaver/pgAdmin Setup:**
1. Create new connection
2. Host: `localhost`
3. Port: `5432` (while SSH tunnel is active)
4. Database: `chat_db`
5. Username: `postgres`
6. Use SSH tunnel in connection settings

### Production Database Security

**DO NOT expose PostgreSQL directly to internet. Use:**

1. **SSH Tunnel** (recommended)
2. **VPN** connection
3. **IP Whitelisting** if you must expose it

```bash
# If you must allow remote access, whitelist specific IPs:
sudo ufw allow from YOUR_IP to any port 5432
```

---

## Monitoring & Logs

```bash
# View application logs
docker compose logs -f chat_server

# View database logs
docker compose logs -f db

# Check running containers
docker ps

# Check resource usage
docker stats
```

---

## Updating/Redeploying

```bash
# Pull latest changes
git pull origin main  # if using git

# Rebuild and restart
docker compose down
docker compose up -d --build

# Or for zero-downtime:
docker compose up -d --build --no-deps chat_server
```

---

## Scaling (Future)

For production scale:
1. Use **Nginx** as reverse proxy
2. Add **SSL/TLS** with Let's Encrypt
3. Use **Redis** for session management
4. Load balance multiple app instances
5. Use managed PostgreSQL (AWS RDS, etc.)

---

## Troubleshooting

**App won't connect to DB:**
```bash
# Check if DB is running
docker compose ps

# Check DB logs
docker compose logs db

# Test DB connection manually
docker compose exec db psql -U postgres -d chat_db
```

**Ports already in use:**
```bash
# Find what's using port 8080
lsof -i :8080

# Kill the process or change port in docker-compose.yml
```

**Migrations fail:**
```bash
# Run migrations manually
docker compose exec chat_server ls migrations/
# Check if files are present, then restart
docker compose restart chat_server
```
