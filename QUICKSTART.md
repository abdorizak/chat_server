# Quick Start Commands

## Local Development

```bash
# Start in development mode
./scripts/build-local.sh

# Or manually:
docker compose up --build

# Stop
docker compose down

# View logs
docker compose logs -f chat_server
```

## Access Database

```bash
# Using psql
psql -h localhost -p 5432 -U postgres -d chat_db
# Password: postgres

# Using Docker
docker-compose exec db psql -U postgres -d chat_db
```

## Test API

```bash
# Health check
curl http://localhost:8080/health

# Register user
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john",
    "email": "john@example.com",
    "password": "password123",
    "first_name": "John",
    "last_name": "Doe"
  }'

# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "password123"
  }'
```

## Production Deployment

```bash
# On your server
cd /home/user/chat_server
cp .env.production .env
# Edit .env with actual values
nano .env

# Or manually:
docker compose up -d --build
```

## Database Backup

```bash
# Backup
docker compose exec db pg_dump -U postgres chat_db > backup.sql

# Restore
cat backup.sql | docker compose exec -T db psql -U postgres chat_db
```

## SSH Tunnel for Remote DB Access

```bash
# Forward remote DB to local port
ssh -L 5432:localhost:5432 user@your-server-ip

# Now connect to localhost:5432
psql -h localhost -p 5432 -U postgres -d chat_db
```
