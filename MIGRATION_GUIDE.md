# Field Type Migration Guide

## Overview
This guide explains how to migrate from legacy field types (`Select` and `Link`) to the new enhanced types (`Dropdown` and `Association`).

## Prerequisites
- ✅ Backup your database before running any migration
- ✅ Test on a staging environment first
- ✅ Review the migration script: `migrations/fieldtype_migration.sql`
- ✅ Ensure application code supports fallback handling

## Migration Strategy

### Two-Phase Approach

#### Phase 1: Database Migration (Optional)
Run the SQL migration script to convert existing field definitions:

```bash
# PostgreSQL
psql -U your_user -d your_database -f migrations/fieldtype_migration.sql

# MySQL/MariaDB (with modifications)
mysql -u your_user -p your_database < migrations/fieldtype_migration_mysql.sql
```

#### Phase 2: Application-Level Fallback (Recommended)
The application automatically handles legacy field types using the `migrate_if_needed()` method.

## What Gets Migrated

### Select → Dropdown
**Before:**
```json
{
  "field_type": "Select",
  "metadata": {
    "options": ["Option 1", "Option 2", "Option 3"]
  }
}
```

**After:**
```json
{
  "field_type": "Dropdown",
  "metadata": {
    "options": [
      {
        "value": "Option 1",
        "label": "Option 1",
        "color": null,
        "icon": null,
        "is_default": false,
        "sort_order": 0
      },
      {
        "value": "Option 2",
        "label": "Option 2",
        "color": null,
        "icon": null,
        "is_default": false,
        "sort_order": 1
      }
    ],
    "allow_create": false
  }
}
```

### Link → Association
**Before:**
```json
{
  "field_type": "Link",
  "metadata": {
    "target_entity": "Contact"
  }
}
```

**After:**
```json
{
  "field_type": "Association",
  "metadata": {
    "target_entity": "Contact",
    "display_field": "name",
    "allow_inline_create": false
  }
}
```

## Testing the Migration

### 1. Verify Backup Created
```sql
SELECT COUNT(*) FROM field_definitions_backup_20251224;
```

### 2. Check Migration Results
```sql
-- Should return 0 rows if migration is complete
SELECT * FROM field_definitions 
WHERE field_type IN ('Select', 'Link');

-- Check new field types
SELECT field_type, COUNT(*) 
FROM field_definitions 
WHERE field_type IN ('Dropdown', 'Association')
GROUP BY field_type;
```

### 3. Test in Application
1. Navigate to Component Playground: `/app/playground`
2. Test Dropdown fields with existing data
3. Test Association fields with entity lookups
4. Verify form submissions work correctly

## Fallback Mechanism

The application includes automatic fallback for unmigrated data:

```rust
// Automatically converts legacy types on load
let field = field_repo.get_field(id).await?;
// Field is auto-migrated if it's a legacy type
```

This means:
- ✅ Migration script is **optional** but recommended
- ✅ Application handles both old and new formats
- ✅ Zero downtime deployment possible
- ✅ Gradual migration supported

## Rollback Procedure

If issues occur, rollback using:

```sql
-- Restore from backup
TRUNCATE TABLE field_definitions;
INSERT INTO field_definitions 
SELECT * FROM field_definitions_backup_20251224;

-- Verify restoration
SELECT COUNT(*) FROM field_definitions;
```

## Post-Migration Cleanup

After confirming successful migration (7-14 days):

```sql
-- Drop backup tables
DROP TABLE IF EXISTS field_definitions_backup_20251224;
DROP TABLE IF EXISTS entity_metadata_backup_20251224;

-- Remove migration flags
UPDATE field_definitions
SET metadata = metadata - 'migrated'
WHERE metadata->>'migrated' = 'true';
```

## Troubleshooting

### Issue: Fields not displaying correctly
**Solution:** Clear browser cache and reload

### Issue: Validation errors on form submit
**Solution:** Verify field metadata structure matches new schema

### Issue: API errors for entity lookups
**Solution:** Ensure Association field has `target_entity` and `display_field`

### Issue: Migration script  fails
**Solution:** Check database user permissions (need ALTER TABLE, UPDATE, CREATE TABLE)

## Support

For migration issues:
1. Check logs: `tail -f /var/log/your-app/migration.log`
2. Verify database schema: `\d+ field_definitions`
3. Test with sample data first
4. Contact support if rollback is needed

## Best Practices

1. **Always backup** before migration
2. **Test on staging** environment first
3. **Monitor application logs** during migration
4. **Keep backups** for at least 30 days
5. **Gradual rollout**: Migrate entity types one at a time
6. **User communication**: Notify users of scheduled maintenance

## Timeline

- **Pre-Migration:** 30 minutes (backup + verification)
- **Migration:** 5-15 minutes (depends on data volume)
- **Verification:** 30 minutes (testing)
- **Cleanup:** After 7-14 days of successful operation

---

**Last Updated:** 2025-12-24  
**Version:** 1.0.0  
**Compatible With:** SmartField v2.0+
