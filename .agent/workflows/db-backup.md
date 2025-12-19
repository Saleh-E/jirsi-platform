# Database Backup & Recovery Workflow

## Prevent Future Data Loss

This workflow ensures you never lose database data again.

---

## 1. Create a Backup (BEFORE any changes)

```powershell
# Backup entire database to SQL file
docker exec saas-postgres pg_dump -U postgres -d saas > e:\s_programmer\Saas-System\backups\saas_backup_$(Get-Date -Format "yyyyMMdd_HHmmss").sql

# Or use compressed format
docker exec saas-postgres pg_dump -U postgres -d saas -Fc > e:\s_programmer\Saas-System\backups\saas_backup.dump
```

## 2. Restore from Backup

```powershell
# Restore from SQL file
docker exec -i saas-postgres psql -U postgres -d saas < e:\s_programmer\Saas-System\backups\saas_backup.sql

# Or restore from compressed dump
docker exec -i saas-postgres pg_restore -U postgres -d saas e:\s_programmer\Saas-System\backups\saas_backup.dump
```

---

## For DigitalOcean Production

### Use Managed PostgreSQL (RECOMMENDED)

1. Create a **Managed PostgreSQL Database** in DigitalOcean
2. Get the connection string from the dashboard
3. Set `DATABASE_URL` environment variable to this connection string
4. **Automatic daily backups** are included
5. **Point-in-time recovery** available

### Connection String Format:
```
postgresql://username:password@your-db-host:25060/defaultdb?sslmode=require
```

### Benefits:
- ✅ Automatic daily backups (7-day retention)
- ✅ Point-in-time recovery
- ✅ High availability
- ✅ No Docker networking issues
- ✅ Scales with your app

---

## Quick Commands

```powershell
# Start postgres
docker start saas-postgres

# Start backend
$env:DATABASE_URL="postgres://postgres@localhost:15432/saas"
cargo run --package backend-api

# Start frontend
cd crates/frontend-web
trunk serve --port 8097

# Create backup
docker exec saas-postgres pg_dump -U postgres -d saas > backup.sql

# Restore backup
docker exec -i saas-postgres psql -U postgres -d saas < backup.sql
```
