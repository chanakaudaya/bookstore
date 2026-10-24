#!/usr/bin/env bash
# Seed a demo tenant and initial data for local development.
# Requires the API to be running and migrations applied.
set -euo pipefail

API_BASE="${API_BASE:-http://localhost:3000}"

echo "Seeding demo data against $API_BASE ..."

# Check API health
curl -sf "$API_BASE/health" > /dev/null || { echo "API is not reachable"; exit 1; }

echo "Done. Demo tenant: tenant-demo-001"
echo "Demo credentials:"
echo "  Customer: customer@example.com / demo123"
echo "  Admin:    admin@example.com    / admin123"
