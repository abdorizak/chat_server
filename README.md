# Chat Server - WebSocket Real-Time Messaging

A real-time chat application built with Rust, featuring WebSocket support, PostgreSQL database, JWT authentication, 1-to-1 messaging, and group chats.

## Features

- ✅ **User Authentication** - Register, login with JWT tokens
- ✅ **Friend Management** - Add friends, send/accept friend requests
- 📝 **1-to-1 Chat** - Direct messaging between users
- 👥 **Group Chat** - Create groups, add friends, group messaging
- 🔄 **Real-time** - WebSocket support for instant message delivery
- 🐳 **Docker Ready** - Full Docker and docker-compose configuration
- 🗄️ **PostgreSQL** - Robust database with migrations

## Tech Stack

- **Backend**: Rust + Actix-Web
- **Database**: PostgreSQL 15
- **Authentication**: JWT (jsonwebtoken)
- **WebSocket**: Actix-WS
- **Password Hashing**: bcrypt
- **Deployment**: Docker & Docker Compose

## Quick Start

### Prerequisites

- Rust 1.75+ 
- PostgreSQL 15 (or use Docker)
- Docker & Docker Compose (optional)

### Method 1: Using Docker (Recommended)

1. **Clone and setup**
```bash
cd chat_server
cp .env.example .env
```

2. **Update `.env` file with your configuration** (optional)

3. **Build and run with Docker Compose**
```bash
docker-compose up --build
```

The server will start at `http://localhost:8080`

### Method 2: Local Development

1. **Install PostgreSQL** and create a database:
```bash
createdb chatdb
createuser chatuser
```

2. **Setup environment**
```bash
cp .env.example .env
# Edit .env with your database credentials
```

3. **Build and run**
```bash
cargo build
cargo run
```

## API Endpoints

### Authentication

- `POST /api/auth/register` - Register new user
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "phone": "+1234567890",
  "password": "securepassword123"
}
```

- `POST /api/auth/login` - Login
```json
{
  "email": "john@example.com",
  "password": "securepassword123"
}
```

- `GET /api/auth/me` - Get current user (requires JWT token)

### Health Check

- `GET /health` - Server health check

## Database Schema

The application uses 8 tables:

1. **users** - User accounts
2. **auth_tokens** - JWT token management
3. **contacts** - Friend/contact relationships
4. **conversations** - 1-to-1 chat conversations
5. **messages** - Direct messages
6. **groups** - Group chat information
7. **group_members** - Group membership
8. **group_messages** - Group messages

Migrations are automatically run on server startup.

## Project Structure

```
chat_server/
├── src/
│   ├── common/          # Shared utilities (API responses)
│   ├── db/              # Database connection & migrations
│   ├── logger/          # Logging setup
│   ├── modules/
│   │   ├── auth/        # Authentication module
│   │   ├── users/       # User management (TODO)
│   │   └── ws/          # WebSocket handling (TODO)
│   ├── utils/           # Helper functions (JWT, bcrypt)
│   └── main.rs          # Application entry point
├── migrations/          # SQL migration files
├── Dockerfile           # Docker configuration
├── docker-compose.yml   # Docker Compose setup
└── .env.example         # Environment template
```

## Development

### Running tests
```bash
cargo test
```

### Check code
```bash
cargo check
cargo clippy
```

### Format code
```bash
cargo fmt
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://chatuser:chatpass@localhost:5432/chatdb` |
| `SERVER_HOST` | Server host | `127.0.0.1` |
| `SERVER_PORT` | Server port | `8080` |
| `JWT_SECRET` | Secret key for JWT | Required |
| `JWT_EXPIRATION` | JWT expiration in seconds | `3600` |
| `RUST_LOG` | Log level | `info` |

## Next Steps

This is a learning project. Upcoming features:

- [ ] Complete WebSocket implementation
- [ ] Friend management endpoints
- [ ] 1-to-1 chat endpoints
- [ ] Group chat functionality
- [ ] Message history and pagination
- [ ] Read receipts
- [ ] Typing indicators
- [ ] File uploads
- [ ] Frontend client

## Contributing

This is a learning project. Feel free to fork and experiment!

## License

MIT
