# Migrate DB: Database Migration

## Trigger
`/migrate-db <schema change or migration>`

Orchestrates safe database migrations with validation, rollback, and data preservation.

## Purpose
The Migrate DB skill provides:
- Schema migration planning
- Data migration strategy
- Backup and rollback capability
- Zero-downtime migration support
- Validation and testing
- Documentation

## Workflow

### Phase 1: Analyze Migration Scope

1. **Identify schema changes**
   - New tables
   - New columns
   - Column modifications
   - Index changes
   - Constraint additions
   - Data type changes

2. **Query related decisions**
   ```bash
   cco knowledge search "database schema"
   cco knowledge search "data model"
   cco knowledge search "migration"
   ```

3. **Assess impact**
   - Data volume
   - Downtime implications
   - Rollback complexity
   - Application compatibility

### Phase 2: Spawn Database Team

**Parallel team:**
- **Database Architect** (Sonnet) - Schema design & planning
- **Database Admin** (Sonnet) - Migration execution
- **Database Optimizer** (Sonnet) - Performance validation
- **Test Engineer** (Sonnet) - Migration testing

### Phase 3: Design Migration Plan

**Database Architect creates:**

1. **Migration Steps**
   ```
   Step 1: Create new column with default value
   - Column: user_sessions.active
   - Type: BOOLEAN
   - Default: TRUE
   - Nullable: No
   - Timeline: 5 seconds (minimal)

   Step 2: Populate new column from existing data
   - Query: UPDATE with calculation
   - Estimated time: 2 minutes (100k rows)
   - Rollback: Easy (reverse update)

   Step 3: Create index on active column
   - Type: B-tree index
   - Timeline: 3 minutes
   - Rollback: DROP INDEX

   Step 4: Add foreign key constraint
   - From: user_sessions.user_id
   - To: users.id
   - Timeline: 1 minute (validation only)
   - Rollback: Easy (drop constraint)
   ```

2. **Rollback Plan**
   ```
   If any step fails:
   1. Stop migration immediately
   2. Drop new column/index (if created)
   3. Restore previous state from backup
   4. Communicate failure to team
   5. Analyze root cause
   6. Redesign approach if needed
   7. Retry with new plan
   ```

3. **Downtime Strategy**
   ```
   Option 1: Full Downtime (Simple, fast)
   - Stop application (30 seconds)
   - Run migration (5 minutes)
   - Restart application (30 seconds)
   - Total: ~6 minutes downtime
   - Recommended for: Small tables, low-impact changes

   Option 2: Zero-Downtime (Complex, longer)
   - Create new table in parallel
   - Migrate data with triggers
   - Switch traffic with no downtime
   - Recommended for: Large tables, high-traffic apps

   Selected Strategy: Zero-Downtime (user_sessions is large)
   ```

### Phase 4: Create Migration Script

**Database Admin creates:**

```sql
-- Migration: 2024-01-15_001_add_active_column.sql
-- Author: Database Architect
-- Rollback: See rollback section

BEGIN TRANSACTION;

-- Step 1: Create new column
ALTER TABLE user_sessions
ADD COLUMN active BOOLEAN DEFAULT TRUE NOT NULL;

-- Step 2: Populate from existing data
UPDATE user_sessions
SET active = (status != 'inactive');

-- Step 3: Create index
CREATE INDEX idx_user_sessions_active
ON user_sessions(active);

-- Step 4: Add foreign key
ALTER TABLE user_sessions
ADD CONSTRAINT fk_user_sessions_user_id
FOREIGN KEY (user_id) REFERENCES users(id)
ON DELETE CASCADE;

COMMIT;

-- Rollback Script (if needed)
-- BEGIN TRANSACTION;
-- ALTER TABLE user_sessions
-- DROP CONSTRAINT fk_user_sessions_user_id;
-- DROP INDEX idx_user_sessions_active;
-- ALTER TABLE user_sessions
-- DROP COLUMN active;
-- COMMIT;
```

### Phase 5: Test Migration

**Test Engineer executes on staging:**

1. **Pre-migration validation**
   ```bash
   # Check database state
   SELECT COUNT(*) FROM user_sessions;
   SELECT COUNT(DISTINCT user_id) FROM user_sessions;

   # Expected:
   # 500,000 rows total
   # 50,000 distinct users
   ```

2. **Run migration on staging**
   ```bash
   # Execute migration script
   psql -d staging_db -f migration.sql

   # Validate success
   SELECT * FROM user_sessions LIMIT 5;
   # Check active column populated
   ```

3. **Post-migration validation**
   ```bash
   # Verify data integrity
   SELECT COUNT(CASE WHEN active = true THEN 1 END) FROM user_sessions;
   # Should show ~ expected count

   # Verify index created
   SELECT * FROM pg_indexes WHERE tablename = 'user_sessions';

   # Verify foreign key
   SELECT * FROM information_schema.table_constraints
   WHERE table_name='user_sessions' AND constraint_type='FOREIGN KEY';
   ```

4. **Performance test**
   ```bash
   # Test index effectiveness
   SELECT * FROM user_sessions WHERE active = TRUE LIMIT 10;
   # Should use new index

   EXPLAIN ANALYZE SELECT * FROM user_sessions WHERE active = TRUE;
   # Should show index scan
   ```

5. **Application compatibility test**
   ```bash
   # Deploy code to staging
   # Run application tests
   pytest tests/ -v

   # Verify application works with new schema
   # Test new feature using active column
   ```

### Phase 6: Create Backup

**Database Admin ensures backup:**

```bash
# Full database backup before migration
pg_dump prod_db > prod_db_backup_2024-01-15.sql

# Verify backup
pg_dump -l prod_db_backup_2024-01-15.sql

# Store backup in multiple locations
aws s3 cp prod_db_backup_2024-01-15.sql s3://backups/
```

### Phase 7: Deploy Migration

**Execution options:**

**Option A: Full Downtime**
```
1. 14:00 - Notify users of maintenance window
2. 14:30 - Stop application
3. 14:35 - Run migration (5 min)
4. 14:40 - Run post-migration validation
5. 14:45 - Start application
6. 14:50 - Monitor for issues
7. 15:00 - Resume normal operations
```

**Option B: Zero-Downtime**
```
1. 14:00 - Create new user_sessions_v2 table
2. 14:30 - Start replicating data (trigger-based)
3. 14:45 - Verify all data replicated
4. 14:50 - Switch application to new table (code deployment)
5. 14:55 - Monitor for issues
6. 15:00 - Drop old table
7. 15:05 - Resume normal operations
```

### Phase 8: Validate Migration

**Post-deployment checks:**

```bash
# 1. Data integrity
SELECT COUNT(*) FROM user_sessions;
# Should match pre-migration count

# 2. Schema verification
\d user_sessions
# Should show new column/index/constraint

# 3. Application functionality
# Manual testing of affected features
# Users can log in, refresh tokens work

# 4. Performance
SELECT AVG(duration) FROM activity_log WHERE timestamp > NOW() - INTERVAL '5 min';
# Should show no degradation

# 5. Monitor metrics
# Application error rate: < 0.1%
# Response time: Within baseline
# Database CPU: < 70%

# 6. User impact
# Check support tickets
# No complaints about auth/session issues
```

### Phase 9: Document and Monitor

**Store migration results:**

```bash
cco knowledge store "Migration: user_sessions schema
Date: 2024-01-15 14:30
Duration: 15 minutes
Rows affected: 500,000
Downtime: Zero (Option B)
Status: SUCCESS ✓

Changes:
- Added active column (BOOLEAN)
- Created index idx_user_sessions_active
- Added foreign key to users table

Validation:
- Data integrity: ✓
- Schema verified: ✓
- Performance: ✓
- Application: ✓
- Zero data loss: ✓

Rollback: Available but not needed" \
  --type completion \
  --agent database-admin
```

**Ongoing monitoring:**

```bash
# Monitor for issues
# Alert on error rate increase
# Alert on query performance degradation
# Alert on constraint violations

# Continue monitoring for 24 hours
# Then move to normal monitoring
```

## Migration Patterns

### Add Column
```sql
-- Simple, low risk
ALTER TABLE users ADD COLUMN created_at TIMESTAMP DEFAULT NOW();

-- Considerations:
-- - Default value allows existing rows
-- - Non-nullable column needs default or existing data
-- - Index needed if used in queries
```

### Remove Column
```sql
-- Moderate risk (data loss)
ALTER TABLE users DROP COLUMN deprecated_field;

-- Considerations:
-- - Irreversible (backup first!)
-- - Dependent code must be updated first
-- - Check for foreign keys referencing it
```

### Rename Column
```sql
-- Moderate risk (dependent code)
ALTER TABLE users RENAME COLUMN old_name TO new_name;

-- Considerations:
-- - Requires code changes first (or compatibility layer)
-- - Triggers/views referencing old name break
-- - Application must handle both names during transition
```

### Change Data Type
```sql
-- High risk (data loss possible)
ALTER TABLE users ALTER COLUMN age TYPE VARCHAR;

-- Considerations:
-- - May fail if data incompatible
-- - Requires temporary column approach:
--   1. Create temp column with new type
--   2. Copy and convert data
--   3. Drop old column
--   4. Rename temp column
```

### Add Constraint
```sql
-- Low-to-moderate risk (could block)
ALTER TABLE users
ADD CONSTRAINT email_unique UNIQUE(email);

-- Considerations:
-- - Could fail if constraint violated
-- - Verify data first
-- - May require data cleanup
```

## Best Practices

1. **Test thoroughly** - Staging environment first
2. **Backup always** - Before any migration
3. **Plan rollback** - Know how to reverse
4. **Minimize downtime** - Use zero-downtime when possible
5. **Validate data** - Verify integrity after migration
6. **Monitor closely** - Watch for 24 hours
7. **Document changes** - For team reference
8. **Update code** - Ensure compatibility
9. **Communicate** - Notify users of maintenance
10. **Iterate cautiously** - Large migrations in steps

## Timeline Considerations

| Change Type | Estimated Time | Risk | Downtime |
|------------|-----------------|------|----------|
| Add nullable column | 30 sec | Low | None |
| Add column with default | 2-5 min | Low | None |
| Add index | 5-15 min | Low | None |
| Remove column | 5-10 min | High | Low |
| Change data type | 10-30 min | High | Medium |
| Large table migration | 30 min-hours | High | Variable |

## Output Format

Return comprehensive migration plan with:
- Migration scope and changes
- Step-by-step migration script
- Rollback procedure
- Downtime strategy
- Testing and validation plan
- Backup and recovery procedures
- Post-deployment validation checklist
- Monitoring plan for 24 hours
- Knowledge base references
