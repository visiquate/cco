# Claude Orchestra → Claude Orchestra Rename Plan

**Status**: Planning Phase
**Date**: 2025-11-10
**Estimated Time**: 4-6 hours
**Risk Level**: Medium (with proper backups: Low)

## 📊 Scope Summary

| Category | Count | Location |
|----------|-------|----------|
| **Main Repo References** | ~893 | `/Users/brent/git/cc-army` |
| **Global Config References** | 40 (10 paths) | `/Users/brent/.claude/CLAUDE.md` |
| **Cross-Repo References** | 3 files | `~/git/docs/`, `VisiQuateID-*` |
| **Files to Rename** | 18 files | Root, config, src, docs |
| **History Files** | ~90 files | `~/.claude/projects/` (optional) |
| **Total Impact** | ~950+ refs | Across 4 repos + global config |

---

## 🎯 Overview

Complete rebranding from military "Army" metaphor to musical "Orchestra" metaphor:
- **Army** → **Orchestra**
- **Agents** → **Musicians** (or keep "Agents" as musical ensemble members)
- **Chief Architect** → **Conductor**
- **Agent roles** → **Instrument sections** (optional)
- **Coordination** → **Orchestration** (already used!)
- **Deploy/Deployment** → **Performance/Composition** (context-dependent)

---

## 📋 Phase 1: Repository & Infrastructure

### 1.1 GitHub Repository
- [ ] Rename GitHub repo: `cc-army` → `cc-orchestra` or `claude-orchestra`
- [ ] Update all clone URLs in documentation
- [ ] Update git remote URLs for existing clones
- [ ] Archive or redirect old repository name (if possible)

### 1.2 Directory Structure
- [ ] Rename local directory: `/Users/brent/git/cc-army` → `/Users/brent/git/cc-orchestra`
- [ ] Update all absolute path references to new location
- [ ] Update global CLAUDE.md path references (10 explicit path references)

### 1.3 Package Configuration
**File**: `package.json`
- [ ] Change `name`: `"claude-army"` → `"claude-orchestra"`
- [ ] Update `description`: "Multi-agent development army..." → "Multi-agent development orchestra..."
- [ ] Update `repository.url`: Update GitHub URL
- [ ] Update `main`: `"src/orchestra-conductor.js"` → `"src/orchestra-conductor.js"`
- [ ] Update npm script: `"army": "node src/orchestra-conductor.js"` → `"orchestra": "node src/orchestra-conductor.js"`
- [ ] Update help script to reference new guide: `ORCHESTRA_USAGE_GUIDE.md`

---

## 📁 Phase 2: File Renames

### 2.1 Root Level Files
```bash
ORCHESTRA_ROSTER.md                  → ORCHESTRA_ROSTER.md
ORCHESTRA_ROSTER_V2.md               → ORCHESTRA_ROSTER_V2.md
ORCHESTRA_ROSTER_V1_DEPRECATED.md    → ORCHESTRA_ROSTER_V1_DEPRECATED.md
ARMY_INTEGRATION_PLAN.md        → ORCHESTRA_INTEGRATION_PLAN.md
```

### 2.2 Configuration Files
```bash
config/orchestra-config.json         → config/orchestra-config.json
config/orchestra-config-tdd.json     → config/orchestra-config-tdd.json
config/orchestra-config.json.backup  → config/orchestra-config.json.backup
```

### 2.3 Source Files
```bash
src/orchestra-conductor.js               → src/orchestra-conductor.js
src/orchestra-conductor.js.backup-*      → src/orchestra-conductor.js.backup-*
```

### 2.4 Documentation Files (18 files)
```bash
docs/ARMY_USAGE_GUIDE.md               → docs/ORCHESTRA_USAGE_GUIDE.md
docs/ORCHESTRA_ROSTER_TDD.md                → docs/ORCHESTRA_ROSTER_TDD.md
docs/COMPREHENSIVE_ORCHESTRA_ROSTER.md      → docs/COMPREHENSIVE_ORCHESTRA_ROSTER.md
docs/QUICK_AGENT_REFERENCE.md          → docs/QUICK_MUSICIAN_REFERENCE.md (or keep AGENT)
docs/ccproxy/ARMY_MODEL_ASSIGNMENTS.md → docs/ccproxy/ORCHESTRA_MODEL_ASSIGNMENTS.md
```

### 2.5 Scripts
```bash
scripts/build-comprehensive-config.js  → Update references inside
```

---

## 🎵 Phase 3: Terminology Mapping

### 3.1 Primary Terms
| Old Term | New Term | Context |
|----------|----------|---------|
| Army | Orchestra | All contexts |
| army | orchestra | All contexts |
| ARMY | ORCHESTRA | All contexts |
| Chief Architect | Conductor | Agent role |
| agent roster | musician roster OR agent roster | Keep "agent" or use "musician" |
| deploy the army | assemble the orchestra | Action |
| army configuration | orchestra configuration | Config |
| orchestra-conductor | orchestra-conductor | Main file |

### 3.2 Keep As-Is (Already Good)
- ✅ **Orchestration** - Already perfect!
- ✅ **Coordination** - Works well
- ✅ **Agent** - Can mean orchestra member
- ✅ **Swarm** - Different metaphor, leave as-is for now
- ✅ **Pipeline** - Technical term, keep

### 3.3 Context-Dependent
| Term | Keep/Change | Notes |
|------|-------------|-------|
| "Deploy" | Context | "Deploy agents" → "Assemble musicians" BUT "Deploy to production" stays |
| "Agent" | Keep | Works for orchestra members (musicians ARE agents) |
| "Specialist" | Keep | Musicians are specialists on instruments |
| "Team" | → "Ensemble" | Optional, but "team" works too |

---

## 📝 Phase 4: Content Updates (by priority)

### 4.1 Critical Files (Must Update)
1. **README.md** (80+ references)
   - Title: "Claude Orchestra" → "Claude Orchestra"
   - All "army" → "orchestra"
   - "Chief Architect" → "Conductor"

2. **CLAUDE.md** (project-level, 100+ references)
   - Complete rewrite of branding
   - Update all agent roster references
   - Fix file paths and references

3. **package.json** (see Phase 1.3)

4. **src/orchestra-conductor.js** (main entry point)
   - All function names with "army"
   - Comments and documentation
   - File header

5. **config/orchestra-config.json**
   - Schema references
   - Agent role descriptions

### 4.2 Documentation Files (47 files with references)
- Each file needs search/replace for "army" → "orchestra"
- Manual review for context (deploy, military terms)
- Update cross-references between docs

### 4.3 Global Configuration
**File**: `/Users/brent/.claude/CLAUDE.md` (40 references total, 10 explicit paths)

**Explicit Path References to Update:**
```bash
Line 11:  /Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md
Line 45:  /Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md
Line 50:  /Users/brent/git/cc-orchestra/src/knowledge-manager.js
Line 51:  /Users/brent/git/cc-orchestra/src/knowledge-manager.js
Line 548: /Users/brent/git/cc-orchestra/config/orchestra-config.json
Line 549: /Users/brent/git/cc-orchestra/docs/
Line 771: ~/git/cc-orchestra/src/knowledge-manager.js (3 instances)
```

**Terminology Updates:**
- [ ] "Claude Orchestra" → "Claude Orchestra" (section headers)
- [ ] "the army" → "the orchestra"
- [ ] "Army Configuration Location" → "Orchestra Configuration Location"
- [ ] "army configuration" → "orchestra configuration"
- [ ] "Deploy the army" → "Assemble the orchestra"
- [ ] "Army Composition" → "Orchestra Ensemble"
- [ ] "Army Invocation" → "Orchestra Performance"
- [ ] Keep "orchestration" and "coordination" (already correct!)

### 4.4 Cross-Repository References
**File**: `/Users/brent/git/docs/1password-secrets-migration-guide.md`
- [ ] Line 6: Update project list: "cc-army" → "cc-orchestra"

**File**: `/Users/brent/git/VisiQuateID-app/docs/AGENT_COORDINATION_MATRIX.md`
- [ ] Review for generic agent references (likely no changes needed)

**File**: `/Users/brent/git/VisiQuateID-push/docs/TEST_FIX_REPORT_FINAL.md`
- [ ] Review for generic agent references (likely no changes needed)

### 4.5 Claude History Files (Optional - Low Priority)
**Location**: `/Users/brent/.claude/projects/-Users-brent-git-cc-army/*.jsonl`
- [ ] These are historical logs - can be left as-is
- [ ] OR rename directory: `-Users-brent-git-cc-army` → `-Users-brent-git-cc-orchestra`
- [ ] File contents don't need updating (historical record)

---

## 🔧 Phase 5: Technical Implementation

### 5.1 Automated Search & Replace

**Safe patterns (regex):**
```bash
# Case-sensitive replacements
orchestra-config → orchestra-config
orchestra-conductor → orchestra-conductor
armyConfig → orchestraConfig
ORCHESTRA_ROSTER → ORCHESTRA_ROSTER

# Case-insensitive (review context first)
"army" → "orchestra" (in prose)
"the army" → "the orchestra"
"Claude Orchestra" → "Claude Orchestra"
```

**Exclude patterns:**
```bash
# Don't replace in:
- node_modules/
- .git/
- Binary files
- package-lock.json (update via npm)
```

### 5.2 Manual Review Required
- Comments with military metaphors
- Documentation with extended metaphors
- Error messages
- User-facing strings
- Git commit messages (historical)

### 5.3 Testing Checklist
- [ ] All file imports still resolve
- [ ] No broken links in documentation
- [ ] Package.json scripts still work
- [ ] npm commands execute correctly
- [ ] Knowledge manager paths updated
- [ ] Credential manager references updated

---

## 🎼 Phase 6: Enhanced Metaphor (Optional)

### 6.1 Consider These Additions
- **Sections**: Group agents by instrument family
  - Strings: Python, Swift, Go, Rust experts
  - Brass: DevOps, Security
  - Woodwinds: QA, Documentation
  - Percussion: Credential Manager, Utilities

- **Musical Terms**:
  - "Rehearsal" → Testing phase
  - "Performance" → Production deployment
  - "Score" → Project plan/specification
  - "Tempo" → Execution speed
  - "Harmony" → Agent coordination

### 6.2 Agent Roles Rename (Optional)
| Current | Musical Alternative |
|---------|-------------------|
| Chief Architect | Conductor |
| TDD Coding Agent | First Chair Coder |
| Python Expert | Principal Python |
| QA Engineer | Quality Maestro |
| Security Auditor | Security Virtuoso |
| DevOps Engineer | Stage Manager |

---

## 📊 Phase 7: Execution Order

### Recommended Approach: Big Bang Migration

1. **Pre-Migration Backup** ✅
   ```bash
   # Backup entire cc-army directory
   cp -r /Users/brent/git/cc-army /Users/brent/git/cc-army-backup-$(date +%Y%m%d)

   # Backup global CLAUDE.md
   cp /Users/brent/.claude/CLAUDE.md /Users/brent/.claude/CLAUDE.md.backup-$(date +%Y%m%d)

   # Backup cross-repo files
   cp /Users/brent/git/docs/1password-secrets-migration-guide.md \
      /Users/brent/git/docs/1password-secrets-migration-guide.md.backup
   ```

2. **Phase 2: File Renames** (in cc-army directory)
   ```bash
   cd /Users/brent/git/cc-army

   # Root level
   mv ORCHESTRA_ROSTER.md ORCHESTRA_ROSTER.md
   mv ORCHESTRA_ROSTER_V2.md ORCHESTRA_ROSTER_V2.md
   mv ORCHESTRA_ROSTER_V1_DEPRECATED.md ORCHESTRA_ROSTER_V1_DEPRECATED.md
   mv ARMY_INTEGRATION_PLAN.md ORCHESTRA_INTEGRATION_PLAN.md

   # Config files
   mv config/orchestra-config.json config/orchestra-config.json
   mv config/orchestra-config-tdd.json config/orchestra-config-tdd.json
   mv config/orchestra-config.json.backup config/orchestra-config.json.backup

   # Source files
   mv src/orchestra-conductor.js src/orchestra-conductor.js
   mv src/orchestra-conductor.js.backup-pre-removal src/orchestra-conductor.js.backup-pre-removal

   # Documentation files
   cd docs
   mv ARMY_USAGE_GUIDE.md ORCHESTRA_USAGE_GUIDE.md
   mv ORCHESTRA_ROSTER_TDD.md ORCHESTRA_ROSTER_TDD.md
   mv COMPREHENSIVE_ORCHESTRA_ROSTER.md COMPREHENSIVE_ORCHESTRA_ROSTER.md
   mv ccproxy/ARMY_MODEL_ASSIGNMENTS.md ccproxy/ORCHESTRA_MODEL_ASSIGNMENTS.md
   cd ..
   ```

3. **Phase 3: Automated Search & Replace** (within cc-army)
   ```bash
   # Use this carefully - review changes before committing!
   find . -type f \( -name "*.md" -o -name "*.js" -o -name "*.json" \) \
     -not -path "*/node_modules/*" \
     -exec sed -i '' 's/orchestra-config/orchestra-config/g' {} +

   find . -type f \( -name "*.md" -o -name "*.js" -o -name "*.json" \) \
     -not -path "*/node_modules/*" \
     -exec sed -i '' 's/orchestra-conductor/orchestra-conductor/g' {} +

   find . -type f \( -name "*.md" -o -name "*.js" -o -name "*.json" \) \
     -not -path "*/node_modules/*" \
     -exec sed -i '' 's/ORCHESTRA_ROSTER/ORCHESTRA_ROSTER/g' {} +

   # Case-insensitive replacements (review manually first!)
   find . -type f -name "*.md" -not -path "*/node_modules/*" \
     -exec sed -i '' 's/Claude Orchestra/Claude Orchestra/g' {} +
   ```

4. **Phase 4: Update Global CLAUDE.md**
   ```bash
   # Edit /Users/brent/.claude/CLAUDE.md
   sed -i '' 's|/Users/brent/git/cc-army|/Users/brent/git/cc-orchestra|g' ~/.claude/CLAUDE.md
   sed -i '' 's|~/git/cc-army|~/git/cc-orchestra|g' ~/.claude/CLAUDE.md
   sed -i '' 's/orchestra-config/orchestra-config/g' ~/.claude/CLAUDE.md
   sed -i '' 's/Claude Orchestra/Claude Orchestra/g' ~/.claude/CLAUDE.md
   ```

5. **Phase 5: Update Cross-Repository References**
   ```bash
   # Update 1password migration guide
   sed -i '' 's/cc-army/cc-orchestra/g' /Users/brent/git/docs/1password-secrets-migration-guide.md
   ```

6. **Phase 6: Rename Directory**
   ```bash
   cd /Users/brent/git
   mv cc-army cc-orchestra
   ```

7. **Phase 7: Update Package & Test**
   ```bash
   cd /Users/brent/git/cc-orchestra

   # Verify package.json was updated
   cat package.json | grep orchestra

   # Test npm commands
   npm run orchestra --help

   # Test knowledge manager
   node src/knowledge-manager.js stats

   # Test credential manager
   npm run credentials list
   ```

8. **Phase 8: Update GitHub**
   ```bash
   # Option A: Rename existing repo on GitHub (preserves stars/history)
   # - Go to Settings → Rename repository → cc-orchestra

   # Option B: Create new repo and push
   cd /Users/brent/git/cc-orchestra
   git remote set-url origin https://github.com/yourusername/cc-orchestra.git
   git push -u origin main
   ```

9. **Phase 9: Final Verification**
   ```bash
   # Check all links in documentation
   cd /Users/brent/git/cc-orchestra
   find docs -name "*.md" -exec grep -l "cc-army" {} +  # Should be empty!

   # Verify global config
   grep "cc-army" ~/.claude/CLAUDE.md  # Should be empty!

   # Test everything works
   npm run orchestra
   node src/knowledge-manager.js stats
   npm run credentials list
   ```

10. **Phase 10: Cleanup (Optional)**
    ```bash
    # After 2 weeks of successful operation
    rm -rf /Users/brent/git/cc-army-backup-*
    rm /Users/brent/.claude/CLAUDE.md.backup-*

    # Update Claude history directory (optional)
    mv ~/.claude/projects/-Users-brent-git-cc-army \
       ~/.claude/projects/-Users-brent-git-cc-orchestra
    ```

---

## ⚠️ Breaking Changes

### For Users
- [ ] Directory path changed: `~/git/cc-army` → `~/git/cc-orchestra`
- [ ] Command changed: `npm run army` → `npm run orchestra`
- [ ] Config files renamed: `orchestra-config.json` → `orchestra-config.json`
- [ ] Import paths changed in custom scripts
- [ ] Global CLAUDE.md needs manual update

### For Documentation
- [ ] All guides reference new names
- [ ] Links between docs updated
- [ ] Examples use new terminology
- [ ] Code snippets updated

---

## 📈 Success Metrics

- [ ] Zero broken file imports
- [ ] Zero broken documentation links
- [ ] All npm scripts execute successfully
- [ ] Knowledge manager operational
- [ ] All tests pass (when implemented)
- [ ] Global config references correct paths
- [ ] GitHub repo accessible at new URL

---

## 🚀 Rollout Strategy

### Option A: Big Bang (Recommended)
- Complete all phases in one session
- Single commit with full rename
- Tag as v2.0.0 (breaking change)
- Update all documentation simultaneously

### Option B: Gradual
- Phase 1: Repo & file renames
- Phase 2: Content updates
- Phase 3: Global config updates
- (Not recommended - confusing intermediate state)

---

## 📚 Post-Rename Tasks

1. **Update External References**
   - [ ] Any blog posts or articles
   - [ ] Social media mentions
   - [ ] Documentation hosted elsewhere
   - [ ] Wiki pages

2. **SEO & Discovery**
   - [ ] Update GitHub description
   - [ ] Update package.json keywords
   - [ ] Add redirect from old repo (if possible)

3. **User Communication**
   - [ ] Migration guide in README
   - [ ] CHANGELOG.md entry
   - [ ] Breaking changes notice

---

## 🎭 The New Brand

**Claude Orchestra** - A symphony of specialized AI agents working in harmony:
- The **Conductor** leads the ensemble
- **Musicians** each play their part perfectly
- **Orchestration** brings it all together
- **Performances** deliver production value
- **Rehearsals** ensure quality

Musical metaphor benefits:
- ✅ More collaborative (less militaristic)
- ✅ Emphasizes harmony over hierarchy
- ✅ "Orchestration" is already in use!
- ✅ Better brand perception
- ✅ More creative/artistic tone

---

## 💡 Notes

- Total estimated time: 4-6 hours for complete rename
- Critical path: Package.json → Source files → Docs → Global config
- Testing: 1-2 hours
- Low risk if backed up properly
- High impact on branding and perception

---

## 🤖 Automated Execution Script

For those who want to run the entire migration automatically:

```bash
#!/bin/bash
# army-to-orchestra.sh - Automated rename script

set -e  # Exit on error

echo "🎭 Claude Orchestra → Claude Orchestra Rename Script"
echo "=============================================="
echo ""

# Step 1: Backup
echo "📦 Step 1/10: Creating backups..."
cp -r /Users/brent/git/cc-army "/Users/brent/git/cc-army-backup-$(date +%Y%m%d)"
cp /Users/brent/.claude/CLAUDE.md "/Users/brent/.claude/CLAUDE.md.backup-$(date +%Y%m%d)"
cp /Users/brent/git/docs/1password-secrets-migration-guide.md \
   /Users/brent/git/docs/1password-secrets-migration-guide.md.backup
echo "✅ Backups complete"

# Step 2: File renames
echo "📝 Step 2/10: Renaming files..."
cd /Users/brent/git/cc-army
mv ORCHESTRA_ROSTER.md ORCHESTRA_ROSTER.md
mv ORCHESTRA_ROSTER_V2.md ORCHESTRA_ROSTER_V2.md
mv ORCHESTRA_ROSTER_V1_DEPRECATED.md ORCHESTRA_ROSTER_V1_DEPRECATED.md
mv ARMY_INTEGRATION_PLAN.md ORCHESTRA_INTEGRATION_PLAN.md
mv config/orchestra-config.json config/orchestra-config.json
mv config/orchestra-config-tdd.json config/orchestra-config-tdd.json
mv config/orchestra-config.json.backup config/orchestra-config.json.backup
mv src/orchestra-conductor.js src/orchestra-conductor.js
mv src/orchestra-conductor.js.backup-pre-removal src/orchestra-conductor.js.backup-pre-removal
cd docs
mv ARMY_USAGE_GUIDE.md ORCHESTRA_USAGE_GUIDE.md
mv ORCHESTRA_ROSTER_TDD.md ORCHESTRA_ROSTER_TDD.md
mv COMPREHENSIVE_ORCHESTRA_ROSTER.md COMPREHENSIVE_ORCHESTRA_ROSTER.md
mv ccproxy/ARMY_MODEL_ASSIGNMENTS.md ccproxy/ORCHESTRA_MODEL_ASSIGNMENTS.md
cd ..
echo "✅ Files renamed"

# Step 3: Content replacements
echo "🔄 Step 3/10: Replacing content in files..."
find . -type f \( -name "*.md" -o -name "*.js" -o -name "*.json" \) \
  -not -path "*/node_modules/*" \
  -exec sed -i '' 's/orchestra-config/orchestra-config/g' {} +
find . -type f \( -name "*.md" -o -name "*.js" -o -name "*.json" \) \
  -not -path "*/node_modules/*" \
  -exec sed -i '' 's/orchestra-conductor/orchestra-conductor/g' {} +
find . -type f \( -name "*.md" -o -name "*.js" -o -name "*.json" \) \
  -not -path "*/node_modules/*" \
  -exec sed -i '' 's/ORCHESTRA_ROSTER/ORCHESTRA_ROSTER/g' {} +
find . -type f -name "*.md" -not -path "*/node_modules/*" \
  -exec sed -i '' 's/Claude Orchestra/Claude Orchestra/g' {} +
echo "✅ Content updated"

# Step 4: Update global CLAUDE.md
echo "🌍 Step 4/10: Updating global configuration..."
sed -i '' 's|/Users/brent/git/cc-army|/Users/brent/git/cc-orchestra|g' ~/.claude/CLAUDE.md
sed -i '' 's|~/git/cc-army|~/git/cc-orchestra|g' ~/.claude/CLAUDE.md
sed -i '' 's/orchestra-config/orchestra-config/g' ~/.claude/CLAUDE.md
sed -i '' 's/Claude Orchestra/Claude Orchestra/g' ~/.claude/CLAUDE.md
echo "✅ Global config updated"

# Step 5: Update cross-repo references
echo "🔗 Step 5/10: Updating cross-repository references..."
sed -i '' 's/cc-army/cc-orchestra/g' /Users/brent/git/docs/1password-secrets-migration-guide.md
echo "✅ Cross-repo references updated"

# Step 6: Rename directory
echo "📂 Step 6/10: Renaming main directory..."
cd /Users/brent/git
mv cc-army cc-orchestra
echo "✅ Directory renamed"

# Step 7: Verify changes
echo "🧪 Step 7/10: Verifying changes..."
cd /Users/brent/git/cc-orchestra
if grep -q "orchestra" package.json; then
  echo "✅ package.json updated correctly"
else
  echo "❌ Warning: package.json might need manual review"
fi

# Step 8: Test functionality
echo "🧪 Step 8/10: Testing functionality..."
node src/knowledge-manager.js stats >/dev/null 2>&1 && echo "✅ Knowledge manager works" || echo "⚠️  Knowledge manager needs attention"

# Step 9: Check for remaining references
echo "🔍 Step 9/10: Checking for remaining 'army' references..."
ARMY_COUNT=$(find . -type f -name "*.md" -o -name "*.js" -o -name "*.json" | \
  grep -v node_modules | xargs grep -i "army" 2>/dev/null | wc -l | tr -d ' ')
if [ "$ARMY_COUNT" -gt 50 ]; then
  echo "⚠️  Found $ARMY_COUNT references to 'army' - may need manual review"
else
  echo "✅ Minimal army references remaining ($ARMY_COUNT - likely in prose)"
fi

# Step 10: Final summary
echo ""
echo "🎉 Step 10/10: Migration Complete!"
echo "=============================================="
echo "✅ All automatic steps completed successfully"
echo ""
echo "📋 Next steps:"
echo "1. Review changes: cd /Users/brent/git/cc-orchestra"
echo "2. Test commands: npm run orchestra"
echo "3. Update GitHub repo name in Settings"
echo "4. Commit and push changes"
echo ""
echo "📦 Backups located at:"
echo "  - cc-army-backup-$(date +%Y%m%d)"
echo "  - ~/.claude/CLAUDE.md.backup-$(date +%Y%m%d)"
echo ""
echo "🎭 Welcome to the Claude Orchestra! 🎼"
```

**To execute:**
```bash
# Save the script
cat > /tmp/army-to-orchestra.sh << 'EOF'
[paste script above]
EOF

# Make executable
chmod +x /tmp/army-to-orchestra.sh

# Run it
/tmp/army-to-orchestra.sh
```

---

**Next Step**:
1. Review this plan thoroughly
2. Ensure all backups are in place
3. Choose execution method (manual or automated script)
4. Execute the migration
5. Test everything
6. Update GitHub repository
7. Celebrate! 🎭🎼
