#!/usr/bin/env bash

# Run FundingFlow (Rust backend + Next.js frontend + Postgres) locally without Docker.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="$ROOT_DIR/apps/web"
LOG_DIR="${LOG_DIR:-$ROOT_DIR/log}"

BACKEND_PORT="${BACKEND_PORT:-3001}"
FRONTEND_PORT="${FRONTEND_PORT:-3000}"
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-fundingflow}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-fundingflow}"
POSTGRES_DB="${POSTGRES_DB:-fundingflow}"
DATABASE_URL="${DATABASE_URL:-postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}}"
NEXT_PUBLIC_API_URL="${NEXT_PUBLIC_API_URL:-http://localhost:${BACKEND_PORT}}"
AUTO_INSTALL="${AUTO_INSTALL:-1}"
RUNLOCAL_STATE_DIR="${RUNLOCAL_STATE_DIR:-$ROOT_DIR/.runlocal}"
RUNLOCAL_PID_FILE="${RUNLOCAL_PID_FILE:-$RUNLOCAL_STATE_DIR/pids}"

export DATABASE_URL
export NEXT_PUBLIC_API_URL

log() {
	echo "[runlocal] $*"
}

log "ROOT_DIR: $ROOT_DIR"
log "FRONTEND_DIR: $FRONTEND_DIR"
log "LOG_DIR: $LOG_DIR"

mkdir -p "$LOG_DIR"
mkdir -p "$RUNLOCAL_STATE_DIR"

PIDS=()

usage() {
	cat <<'USAGE'
Usage: ./runlocal.sh [setup|services|backend|frontend|all|migrate|import-<source>|killall|help]

  setup       Install/configure Postgres and create the database + user
  services    Start Postgres
  backend     Build and start the Rust API (axum on BACKEND_PORT)
  frontend    Install npm deps and start Next.js dev server (starts Postgres too)
  all         Run backend and frontend together (default)
  migrate     Run database migrations only
  import-<s>  Import data from <s> (usaspending, lda, fec, sec, rulemaking)
  killall     Stop processes spawned by previous runlocal.sh runs
  help        Show this message

  Environment overrides:
  BACKEND_PORT          API port (default 3001)
  FRONTEND_PORT         Next.js dev port (default 3000)
  POSTGRES_HOST         Postgres hostname (default localhost)
  POSTGRES_PORT         Postgres port (default 5432)
  POSTGRES_USER         Postgres user (default fundingflow)
  POSTGRES_PASSWORD     Postgres password (default fundingflow)
  POSTGRES_DB           Postgres database (default fundingflow)
  DATABASE_URL          Full Postgres connection string
  NEXT_PUBLIC_API_URL   Frontend API base URL (default http://localhost:BACKEND_PORT)
USAGE
}

cleanup() {
	log "Stopping background processes..."
	for pid in "${PIDS[@]}"; do
		if kill -0 "$pid" >/dev/null 2>&1; then
			kill "$pid" >/dev/null 2>&1 || true
		fi
	done
	remove_pids_from_file
	PIDS=()
}

record_pid() {
	local pid="$1"
	local label="$2"

	if [[ -z "$pid" ]]; then
		return 0
	fi

	local timestamp
	timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
	printf "%s|%s|%s\n" "$timestamp" "$pid" "$label" >>"$RUNLOCAL_PID_FILE"
}

remove_pids_from_file() {
	if [[ ! -f "$RUNLOCAL_PID_FILE" ]] || [[ ${#PIDS[@]} -eq 0 ]]; then
		return 0
	fi

	local tmp_file="${RUNLOCAL_PID_FILE}.tmp"
	: >"$tmp_file"

	while IFS= read -r line; do
		local pid
		pid="$(printf "%s" "$line" | cut -d'|' -f2)"
		if [[ -z "$pid" ]]; then
			continue
		fi
		local keep=1
		for target_pid in "${PIDS[@]}"; do
			if [[ "$pid" == "$target_pid" ]]; then
				keep=0
				break
			fi
		done
		if [[ "$keep" -eq 1 ]]; then
			printf "%s\n" "$line" >>"$tmp_file"
		fi
	done <"$RUNLOCAL_PID_FILE"

	mv "$tmp_file" "$RUNLOCAL_PID_FILE" 2>/dev/null || true
}

prune_pid_file() {
	if [[ ! -f "$RUNLOCAL_PID_FILE" ]]; then
		return 0
	fi

	local tmp_file="${RUNLOCAL_PID_FILE}.tmp"
	: >"$tmp_file"

	while IFS='|' read -r timestamp pid label; do
		if [[ -z "$pid" ]]; then
			continue
		fi
		if kill -0 "$pid" >/dev/null 2>&1; then
			printf "%s|%s|%s\n" "$timestamp" "$pid" "$label" >>"$tmp_file"
		fi
	done <"$RUNLOCAL_PID_FILE"

	mv "$tmp_file" "$RUNLOCAL_PID_FILE" 2>/dev/null || true
}

killall_services() {
	if [[ ! -f "$RUNLOCAL_PID_FILE" ]]; then
		log "No runlocal PID file found."
		return 0
	fi

	prune_pid_file

	if [[ ! -s "$RUNLOCAL_PID_FILE" ]]; then
		log "No active runlocal processes found."
		rm -f "$RUNLOCAL_PID_FILE"
		return 0
	fi

	log "Stopping recorded runlocal processes..."
	while IFS='|' read -r _ pid label; do
		if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
			log "Stopping ${label:-process} (pid ${pid})"
			kill "$pid" >/dev/null 2>&1 || true
		fi
	done <"$RUNLOCAL_PID_FILE"

	sleep 0.3

	while IFS='|' read -r _ pid label; do
		if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
			log "Force-stopping ${label:-process} (pid ${pid})"
			kill -9 "$pid" >/dev/null 2>&1 || true
		fi
	done <"$RUNLOCAL_PID_FILE"

	rm -f "$RUNLOCAL_PID_FILE"
}

free_port() {
	local port="$1"
	if command -v ss >/dev/null 2>&1; then
		local pid
		pid="$(ss -lptn "sport = :${port}" 2>/dev/null | awk -F'pid=' 'NR>1 {print $2}' | awk -F',' '{print $1}' | head -n 1)"
		if [[ -n "$pid" ]]; then
			log "Stopping process $pid on port $port"
			kill "$pid" >/dev/null 2>&1 || true
			sleep 0.2
			if kill -0 "$pid" >/dev/null 2>&1; then
				log "Force-stopping process $pid on port $port"
				kill -9 "$pid" >/dev/null 2>&1 || true
			fi
		fi
	fi
}

handle_signal() {
	local signal="$1"
	log "Received ${signal}. Cleaning up..."
	cleanup
	trap - EXIT
	trap - INT TERM
	exit 130
}

trap cleanup EXIT
trap 'handle_signal SIGINT' INT
trap 'handle_signal SIGTERM' TERM

require_cmd() {
	if ! command -v "$1" >/dev/null 2>&1; then
		log "Missing required command: $1"
		exit 1
	fi
}

detect_package_manager() {
	if command -v pacman >/dev/null 2>&1; then
		echo "pacman"
	elif command -v brew >/dev/null 2>&1; then
		echo "brew"
	elif command -v apt-get >/dev/null 2>&1; then
		echo "apt-get"
	elif command -v dnf >/dev/null 2>&1; then
		echo "dnf"
	else
		echo ""
	fi
}

postgres_ready() {
	if command -v pg_isready >/dev/null 2>&1; then
		pg_isready -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" >/dev/null 2>&1
		return $?
	fi

	if command -v psql >/dev/null 2>&1; then
		PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT 1" >/dev/null 2>&1
		return $?
	fi

	return 1
}

start_postgres_service() {
	if postgres_ready; then
		return 0
	fi

	if command -v systemctl >/dev/null 2>&1; then
		log "Starting Postgres with systemctl..."
		sudo systemctl start postgresql || true
	fi

	if postgres_ready; then
		return 0
	fi

	log "Postgres is not running at ${POSTGRES_HOST}:${POSTGRES_PORT}."
	log "Start it manually: sudo systemctl start postgresql"
	return 1
}

ensure_postgres_user_db() {
	if ! command -v psql >/dev/null 2>&1; then
		return 1
	fi

	if PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT 1" >/dev/null 2>&1; then
		return 0
	fi

	if command -v sudo >/dev/null 2>&1; then
		log "Creating Postgres user ${POSTGRES_USER} and database ${POSTGRES_DB}..."
		sudo -u postgres psql -d postgres -c "CREATE ROLE ${POSTGRES_USER} LOGIN PASSWORD '${POSTGRES_PASSWORD}'" 2>/dev/null || true
		sudo -u postgres psql -d postgres -c "CREATE DATABASE ${POSTGRES_DB} OWNER ${POSTGRES_USER}" 2>/dev/null || true
		sudo -u postgres psql -d "$POSTGRES_DB" -c "GRANT ALL ON SCHEMA public TO ${POSTGRES_USER}" 2>/dev/null || true
		return 0
	fi

	log "Cannot create Postgres user/db automatically."
	return 1
}

run_migrate() {
	require_cmd cargo
	log "Running database migrations with DATABASE_URL=$DATABASE_URL"
	pushd "$ROOT_DIR" >/dev/null
	cargo run --bin cli -- migrate
	popd >/dev/null
	log "Migrations complete."
}

run_import() {
	local source="$1"
	require_cmd cargo
	log "Importing data from source: ${source}"
	pushd "$ROOT_DIR" >/dev/null
	cargo run --bin cli -- import --source "$source"
	popd >/dev/null
	log "Import from ${source} complete."
}

run_seed() {
	require_cmd cargo
	log "Seeding sample data"
	pushd "$ROOT_DIR" >/dev/null
	cargo run --bin cli -- seed
	popd >/dev/null
	log "Seed complete."
}

run_backend() {
	require_cmd cargo
	free_port "$BACKEND_PORT"

	pushd "$ROOT_DIR" >/dev/null

	log "Building Rust backend..."
	cargo build --bin api

	log "Starting Rust API on port $BACKEND_PORT"
	cargo run --bin api &
	local backend_pid=$!
	PIDS+=("$backend_pid")
	record_pid "$backend_pid" "backend"

	popd >/dev/null
}

run_frontend() {
	require_cmd pnpm
	free_port "$FRONTEND_PORT"

	pushd "$FRONTEND_DIR" >/dev/null

	if [[ ! -d node_modules ]]; then
		log "Installing frontend dependencies..."
		pnpm install
	fi

	log "Starting Next.js dev server on port $FRONTEND_PORT"
	pnpm dev --port "$FRONTEND_PORT" &
	local frontend_pid=$!
	PIDS+=("$frontend_pid")
	record_pid "$frontend_pid" "frontend"

	popd >/dev/null
}

setup_services() {
	log "Setting up local Postgres..."
	start_postgres_service || return 1
	ensure_postgres_user_db || true
	run_migrate
	run_seed
	log "Setup complete."
	log "Use './runlocal.sh import-usaspending' (or lda, fec, sec, rulemaking) to import data."
}

start_services() {
	start_postgres_service || return 1
}

main() {
	local target="${1:-all}"

	case "$target" in
		setup)
			setup_services
			;;
		services)
			start_services
			;;
		backend)
			start_services
			run_backend
			;;
		frontend)
			start_services
			run_frontend
			;;
		migrate)
			start_services
			run_migrate
			;;
		seed)
			start_services
			run_seed
			;;
		import-usaspending)
			start_services
			run_import usaspending
			;;
		import-lda)
			start_services
			run_import lda
			;;
		import-fec)
			start_services
			run_import fec
			;;
		import-sec)
			start_services
			run_import sec
			;;
		import-rulemaking)
			start_services
			run_import rulemaking
			;;
		all)
			start_services
			run_backend
			run_frontend
			;;
		killall)
			killall_services
			exit 0
			;;
		help|--help|-h)
			usage
			exit 0
			;;
		*)
			log "Unknown target: $target"
			usage
			exit 1
			;;
	esac

	if [[ ${#PIDS[@]} -eq 0 ]]; then
		exit 0
	fi

	log "Services are running. Press Ctrl+C to stop."
	for pid in "${PIDS[@]}"; do
		wait "$pid" || true
	done
}

main "$@"
