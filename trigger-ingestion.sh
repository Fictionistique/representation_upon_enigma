#!/bin/bash

# Manually trigger bill ingestion in the cron container

echo "Triggering bill ingestion..."
docker exec civic_bill_cron /app/ingest-bills.sh

echo ""
echo "To view logs:"
echo "  docker logs -f civic_bill_cron"

