#!/bin/sh
for f in /tmp/migrations/*.sql; do
    echo "Running: $f"
    psql -U postgres -d saas -f "$f"
done
