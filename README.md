# Jira Time Track

A web application to track time spent on Jira tickets easily. It features a Rust backend using Axum and a Vue.js frontend with Tailwind CSS.

## Features

- Track time spent on Jira tickets.
- Manage multiple Jira server configurations.
- Securely store credentials with encryption.
- User authentication with email confirmation and password recovery.
- Reorder tickets for better organization.
- Submit worklogs directly to Jira.

## Tech Stack

### Backend
- **Language:** Rust (Edition 2024)
- **Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Runtime:** [Tokio](https://tokio.rs/)
- **Database:** MySQL 9.5.0 (via [SQLx](https://github.com/launchbadge/sqlx))
- **Email:** [Lettre](https://lettre.at/)
- **Auth:** JWT (JSON Web Tokens) and Argon2 for password hashing.

### Frontend
- **Framework:** [Vue 3](https://vuejs.org/)
- **Build Tool:** [Vite](https://vitejs.dev/)
- **CSS:** [Tailwind CSS](https://tailwindcss.com/)
- **Icons:** [Lucide Vue Next](https://lucide.dev/)
- **Components:** [Vuedraggable](https://github.com/SortableJS/Vue.Draggable)

## Requirements

- **Rust:** 1.75 or newer (Edition 2024 requires a recent version).
- **Node.js & npm:** For frontend development and building.
- **Docker:** To run the application and the MySQL database.
- **SQLx CLI** (Optional): For managing migrations manually.

## Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd jira-time-track
   ```

2. **Environment Variables:**
   Copy the example environment file and fill in the values:
   ```bash
   cp .env.example .env
   ```
   *Note: Ensure `ENCRYPTION_KEY` is a 64-character hex string.*

3. **Start the Application and Database:**
   Use Docker Compose to start both the app and the MySQL service:
   ```bash
   docker-compose up -d --build
   ```
   *Note: Ensure all environment variables in `.env` are set correctly before starting.*

4. **Backend Setup:**
   The backend will automatically run migrations on startup.

5. **Frontend Setup:**
   ```bash
   cd frontend
   npm install
   ```

## Development

### Running the Backend
```bash
cargo run
```
The server will start on `http://localhost:3000` (by default).

### Running the Frontend (with HMR)
```bash
cd frontend
npm run dev
```
The frontend dev server will start (usually on `http://localhost:5173`) and proxy API requests to the backend.

## Building for Production

1. **Build the Frontend:**
   ```bash
   cd frontend
   npm run build
   ```
   This will output the production-ready files to the root `dist/` directory.

2. **Build the Backend:**
   ```bash
   cargo build --release
   ```

3. **Run the Production Server:**
   Ensure your `.env` is configured correctly and run:
   ```bash
   ./target/release/jira-time-track
   ```
   The backend will serve the frontend files from the `dist/` directory.

## Docker

### Running with Docker Compose
The easiest way to run the entire stack (App + MySQL) is using Docker Compose:

1. Create a `.env` file based on `.env.example`.
2. Start the services:
   ```bash
   docker-compose up -d --build
   ```
3. The application will be accessible at `http://localhost:3000`.

### Building the Image Manually
If you want to build the Docker image standalone:

```bash
docker build -t jira-time-track:latest .
```

### Running the Image Standalone
When running the image standalone, you must provide the necessary environment variables:

```bash
docker run -p 3000:3000 --env-file .env jira-time-track:latest
```
Note: Ensure the `DATABASE_URL` in your `.env` points to an accessible MySQL instance.

## Scripts

### Root / Backend
- `cargo run`: Run the development server.
- `cargo test`: Run backend tests.
- `cargo build --release`: Build the backend for production.

### Frontend (`/frontend`)
- `npm install`: Install dependencies.
- `npm run dev`: Start Vite development server.
- `npm run build`: Build frontend for production (outputs to `../dist`).
- `npm run test`: Run frontend unit/component tests with Vitest.
- `npm run preview`: Preview the production build locally.

## Environment Variables

| Variable              | Description                               | Default                 |
|-----------------------|-------------------------------------------|-------------------------|
| `MYSQL_ROOT_PASSWORD` | MySQL root password                       | `mysqlrootpass`         |
| `MYSQL_USER`          | MySQL user                                | `mysqluser`             |
| `MYSQL_PASSWORD`      | MySQL user password                       | `mysqlpassword`         |
| `MYSQL_HOST`          | MySQL host                                | `localhost`             |
| `MYSQL_PORT`          | MySQL port                                | `3306`                  |
| `MYSQL_DBNAME`        | MySQL database name                       | `jira_time_track`       |
| `DATABASE_URL`        | SQLx database connection URL              | `mysql://...`           |
| `JWT_SECRET`          | Secret key for JWT signing                | -                       |
| `ENCRYPTION_KEY`      | 64-character hex key for Jira credentials | -                       |
| `APP_PORT`            | Port for the Axum server                  | `3000`                  |
| `APP_BASE_URL`        | Base URL of the application               | `http://localhost:3000` |
| `SMTP_HOST`           | SMTP server host                          | -                       |
| `SMTP_PORT`           | SMTP server port                          | `587`                   |
| `SMTP_USER`           | SMTP username                             | -                       |
| `SMTP_PASSWORD`       | SMTP password                             | -                       |
| `SMTP_FROM`           | Email address to send from                | -                       |

## Testing

### Backend Tests
```bash
cargo test
```
Includes E2E tests in the `tests/` directory and unit tests within `src/`.

### Frontend Tests
```bash
cd frontend
npm run test
```
Uses Vitest and Vue Test Utils for component and logic testing.

## Project Structure

```text
.
├── Cargo.toml            # Backend configuration
├── docker-compose.yaml   # Docker services (MySQL)
├── dist/                 # Compiled frontend files (Production)
├── frontend/             # Vue.js frontend source
│   ├── src/              # Frontend components, views, assets
│   ├── package.json      # Frontend dependencies and scripts
│   └── vite.config.js    # Vite configuration
├── migrations/           # SQLx database migrations
├── src/                  # Backend source code (Rust)
│   ├── handlers/         # API route handlers
│   ├── middleware/       # Axum middlewares (Auth, etc.)
│   ├── models/           # Database models
│   ├── services/         # Business logic (Jira, Mail)
│   ├── utils/            # Shared utilities
│   ├── lib.rs            # App initialization and routing
│   └── main.rs           # Application entry point
└── tests/                # Backend E2E tests
```

## TODO

- [ ] Add License file.
- [ ] Implement more comprehensive E2E tests for the frontend.
- [ ] CI/CD pipeline configuration.

## License

TODO: Add license information.
