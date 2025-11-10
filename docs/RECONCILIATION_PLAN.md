# Orchestra Configuration Reconciliation Plan

**Generated**: 2025-11-10
**Current Config**: orchestra-config.json v2.0.0
**Agent Files**: 106 agents in ~/.claude/agents/ (107 .md files, 1 is documentation)
**Config Agents**: 129 entries (with duplicates)

---

## 1. Executive Summary

### Current State
The orchestra configuration has evolved organically but now requires reconciliation to align with:
- Available agent files (106 usable agents in filesystem, excluding agent-overview.md documentation)
- Best practices for model selection (haiku vs sonnet-4.5)
- Elimination of duplicate entries (15+ duplicates found)
- More granular agent type assignments (moving away from generic "coder")

### Key Issues Identified
1. **Missing Agents**: 7 agent files exist but aren't in config (agent-overview.md is documentation, not an agent)
2. **Duplicate Entries**: 15 agents appear twice in supportAgents section
3. **Suboptimal Model Usage**: 30+ agents using sonnet-4.5 that should use haiku
4. **Generic Types**: 50+ agents using "coder" when more specific types available
5. **Organization**: Poor categorization leading to config bloat

### Impact
- **Performance**: Using expensive sonnet-4.5 for lightweight tasks (estimated 40% cost reduction with haiku)
- **Maintainability**: Duplicates cause confusion and sync issues
- **Accuracy**: Generic types prevent proper orchestration and agent selection
- **Scalability**: Poor organization makes adding new agents difficult

### Recommended Approach
1. **Phase 1**: Remove duplicates (immediate, no breaking changes)
2. **Phase 2**: Add missing agents (low risk, extends capabilities)
3. **Phase 3**: Switch appropriate agents to haiku (cost optimization)
4. **Phase 4**: Refine agent types (improves orchestration)
5. **Phase 5**: Update documentation (keeps docs in sync)

---

## 2. Missing Agents Analysis

### 7 Agents in Filesystem But Not in Config

**Note**: agent-overview.md is documentation, not an agent definition (excluded from count)

| Agent Name | File | Suggested Category | Type | Model | Priority |
|------------|------|-------------------|------|-------|----------|
| architect-review | architect-review.md | developmentAgents | reviewer | sonnet-4.5 | Medium |
| flutter-go-reviewer | flutter-go-reviewer.md | developmentAgents | reviewer | sonnet-4.5 | Medium |
| supabase-schema-architect | supabase-schema-architect.md | dataAgents | system-architect | sonnet-4.5 | Low |
| supabase-realtime-optimizer | supabase-realtime-optimizer.md | dataAgents | coder | sonnet-4.5 | Low |
| graphql-performance-optimizer | graphql-performance-optimizer.md | developmentAgents | coder | sonnet-4.5 | Medium |
| graphql-security-specialist | graphql-security-specialist.md | securityAgents | security-auditor | sonnet-4.5 | High |
| unused-code-cleaner | unused-code-cleaner.md | supportAgents | coder | haiku | High |
| web-vitals-optimizer | web-vitals-optimizer.md | developmentAgents | coder | sonnet-4.5 | Medium |

**Note**: Some of these (graphql-performance-optimizer, graphql-security-specialist) appear to already be in config but investigation shows they're missing from actual configuration sections.

### Rationale for Each Addition

**architect-review**: Code architecture review specialist, complements code-reviewer with system-level focus.

**flutter-go-reviewer**: Specialized reviewer for Flutter+Go stacks, addresses common integration patterns.

**supabase-schema-architect**: Supabase-specific database design, increasingly common in modern stacks.

**supabase-realtime-optimizer**: Supabase Realtime API optimization, performance-critical for real-time apps.

**unused-code-cleaner**: Automated dead code detection and removal, high ROI with haiku pricing.

**web-vitals-optimizer**: Core Web Vitals optimization, critical for SEO and UX.

---

## 3. Duplicate Analysis

### 15 Duplicate Entries in supportAgents Section

All duplicates appear in **supportAgents** section (lines 1815-2327). Each entry is duplicated sequentially:

| Agent Name | Lines | Type | Model | Action |
|------------|-------|------|-------|--------|
| Test Engineer | 1817-1831, 2073-2087 | test-automator | sonnet-4.5 | Remove second |
| Test Automator | 1834-1849, 2090-2105 | test-automator | sonnet-4.5 | Remove second |
| UI/UX Designer | 1852-1865, 2108-2121 | ux-designer | sonnet-4.5 | Remove second |
| CLI UI Designer | 1868-1883, 2124-2138 | coder | sonnet-4.5 | Remove second |
| Performance Engineer | 1885-1900, 2141-2156 | coder | sonnet-4.5 | Remove second |
| Performance Profiler | 1903-1917, 2159-2173 | coder | sonnet-4.5 | Remove second |
| Context Manager | 1920-1934, 2176-2190 | system-architect | sonnet-4.5 | Remove second |
| Task Decomposition Expert | 1937-1953, 2193-2209 | planner | sonnet-4.5 | Remove second |
| Command Expert | 1956-1969, 2212-2225 | coder | sonnet-4.5 | Remove second |
| Connection Agent | 1972-1986, 2228-2242 | coder | sonnet-4.5 | Remove second |
| Metadata Agent | 1989-2003, 2245-2259 | coder | sonnet-4.5 | Remove second |
| Tag Agent | 2006-2020, 2262-2276 | coder | sonnet-4.5 | Remove second |
| Document Structure Analyzer | 2023-2037, 2279-2293 | coder | sonnet-4.5 | Remove second |
| URL Link Extractor | 2040-2053, 2296-2309 | coder | sonnet-4.5 | Remove second |
| Project Supervisor Orchestrator | 2056-2070, 2312-2326 | coder | sonnet-4.5 | Remove second |

**Root Cause**: Likely copy-paste error or merge conflict during previous config update.

**Impact**:
- Config file bloat (512 extra lines)
- Potential confusion about which entry is authoritative
- Wasted parsing/validation time

**Remediation**: Remove lines 2073-2327 (all duplicate entries in second block)

---

## 4. Agent Type Recommendations

### Current Type Distribution
- **coder (generic)**: 68 agents (53%)
- **test-automator**: 4 agents
- **system-architect**: 6 agents
- **researcher**: 5 agents
- **security-auditor**: 3 agents
- **deployment-engineer**: 4 agents
- **Others**: 11 agents

### Available Claude Code Types (from context)
- `test-automator` - Testing specialists
- `technical-writer` - Documentation
- `deployment-engineer` - DevOps/infrastructure
- `security-auditor` - Security review
- `researcher` - Analysis/research
- `ios-developer` - iOS/mobile
- `backend-dev` - Backend specialists
- `mobile-developer` - Cross-platform mobile
- `python-expert` - Python specialists
- `reviewer` - Code review
- `planner` - Planning/coordination
- `ux-designer` - UI/UX design
- `debugger` - Debugging specialist
- `coder` - General purpose (use sparingly)

### Recommended Type Changes (High Priority)

| Agent Name | Current Type | Recommended Type | Rationale |
|------------|--------------|------------------|-----------|
| Typescript Pro | coder | backend-dev | Backend language specialist |
| Javascript Pro | coder | backend-dev | Backend/Node.js specialist |
| Frontend Developer | coder | reviewer | Reviews frontend patterns |
| Debugger | debugger | debugger | CORRECT (no change) |
| Code Analyzer | coder | reviewer | Analyzes code quality |
| Review Agent | coder | reviewer | Code review specialist |
| Architect Review | coder | reviewer | Architecture review |
| Flutter Go Reviewer | coder | reviewer | Cross-stack review |
| Database Admin | coder | backend-dev | Database operations |
| Database Optimization | coder | backend-dev | Performance specialist |
| Database Optimizer | coder | backend-dev | Query optimization |
| Data Engineer | coder | backend-dev | Data infrastructure |
| Nosql Specialist | coder | backend-dev | Database specialist |
| Sql Pro | coder | backend-dev | Database specialist |
| Shell Scripting Pro | coder | deployment-engineer | Infrastructure scripts |
| Legacy Modernizer | coder | backend-dev | Code refactoring |
| Architecture Modernizer | coder | system-architect | System design |
| Dx Optimizer | coder | deployment-engineer | Developer tooling |
| Git Flow Manager | coder | deployment-engineer | Git automation |
| Dependency Manager | coder | deployment-engineer | Package management |
| Error Detective | coder | debugger | Log analysis |
| Cloud Migration Specialist | coder | deployment-engineer | Cloud migrations |
| Terraform Specialist | coder | deployment-engineer | IaC specialist |
| Network Engineer | coder | deployment-engineer | Network config |
| Monitoring Specialist | coder | deployment-engineer | Observability |
| Devops Troubleshooter | coder | debugger | Production issues |
| Load Testing Specialist | coder | test-automator | Performance testing |
| Api Security Audit | coder | security-auditor | Security review |
| Penetration Tester | coder | security-auditor | Security testing |
| Compliance Specialist | coder | security-auditor | Compliance review |
| Mcp Security Auditor | coder | security-auditor | MCP security |
| Web Accessibility Checker | coder | security-auditor | Accessibility audit |
| Risk Manager | coder | researcher | Risk analysis |

### Recommended Type Changes (Medium Priority)

| Agent Name | Current Type | Recommended Type | Rationale |
|------------|--------------|------------------|-----------|
| Nextjs Architecture Expert | coder | reviewer | Architecture patterns |
| React Performance Optimization | coder | reviewer | Performance review |
| React Performance Optimizer | coder | reviewer | Performance review |
| Graphql Architect | coder | system-architect | API architecture |
| Graphql Performance Optimizer | coder | reviewer | Performance review |
| Graphql Security Specialist | coder | security-auditor | Security review |
| Ai Engineer | coder | backend-dev | ML systems |
| Ml Engineer | coder | backend-dev | ML infrastructure |
| Model Evaluator | coder | researcher | Model analysis |
| Prompt Engineer | coder | researcher | Prompt optimization |
| Llms Maintainer | coder | deployment-engineer | LLM operations |
| All MCP Agents (6) | coder | backend-dev | Protocol integration |
| Changelog Generator | coder | technical-writer | Release notes |
| Markdown Syntax Formatter | coder | technical-writer | Documentation formatting |
| Report Generator | coder | technical-writer | Report creation |
| All Research Agents (10) | coder | researcher | Research specialists |

---

## 5. Haiku Candidates (Cost Optimization)

### Criteria for Haiku Assignment
1. **Lightweight tasks**: Simple transformations, formatting, extraction
2. **Well-defined scope**: Narrow, predictable operations
3. **No complex reasoning**: Pattern matching, not decision-making
4. **Fast response priority**: User-facing operations
5. **Cost optimization**: High-frequency, low-complexity tasks

### High Priority Haiku Conversions (20 agents)

| Agent Name | Current Model | Category | Est. Monthly Savings | Rationale |
|------------|--------------|----------|---------------------|-----------|
| Changelog Generator | sonnet-4.5 | Documentation | $15-25 | Template-based formatting |
| Markdown Syntax Formatter | sonnet-4.5 | Documentation | $20-30 | Pure formatting, no logic |
| Unused Code Cleaner | sonnet-4.5 | Support | $10-20 | Pattern detection |
| URL Link Extractor | sonnet-4.5 | Support | $5-10 | Text parsing |
| Tag Agent | sonnet-4.5 | Support | $5-10 | Taxonomy tagging |
| Metadata Agent | sonnet-4.5 | Support | $5-10 | Metadata extraction |
| Document Structure Analyzer | sonnet-4.5 | Support | $10-15 | Structure parsing |
| CLI UI Designer | sonnet-4.5 | Support | $15-25 | Template generation |
| Git Flow Manager | sonnet-4.5 | Development | $10-20 | Git command patterns |
| Dependency Manager | sonnet-4.5 | Development | $10-20 | Version management |
| Connection Agent | sonnet-4.5 | Support | $5-10 | Obsidian integration |
| Command Expert | sonnet-4.5 | Support | $10-15 | CLI generation |
| Api Documenter | sonnet-4.5 | Documentation | $20-30 | OpenAPI generation |
| Technical Writer | sonnet-4.5 | Documentation | $25-35 | Documentation writing |
| Documentation Expert | sonnet-4.5 | Documentation | $25-35 | Documentation writing |
| Fact Checker | sonnet-4.5 | Research | $15-25 | Verification checks |
| Query Clarifier | sonnet-4.5 | Research | $10-15 | Query parsing |
| Search Specialist | sonnet-4.5 | Research | $20-30 | Search execution |
| Report Generator | sonnet-4.5 | Documentation | $20-30 | Report formatting |
| Web Accessibility Checker | sonnet-4.5 | Security | $15-25 | WCAG validation |

**Estimated Total Monthly Savings**: $300-450 (assuming 20-30 invocations per agent per month)

### Medium Priority Haiku Conversions (10 agents)

| Agent Name | Current Model | Category | Rationale |
|------------|--------------|----------|-----------|
| Business Analyst | sonnet-4.5 | Business | Metrics reporting |
| Content Marketer | sonnet-4.5 | Business | Content templates |
| Llms Maintainer | sonnet-4.5 | AI/ML | SEO maintenance |
| Project Supervisor Orchestrator | sonnet-4.5 | Support | Task coordination |
| Research Brief Generator | sonnet-4.5 | Research | Brief formatting |
| Error Detective | sonnet-4.5 | Development | Log pattern matching |
| Dx Optimizer | sonnet-4.5 | Development | Config generation |
| Monitoring Specialist | sonnet-4.5 | Infrastructure | Alert templates |
| Risk Manager | sonnet-4.5 | Business | Risk templates |
| Compliance Specialist | sonnet-4.5 | Security | Compliance checklists |

**Total Haiku Conversions Recommended**: 30 agents (23% of total)

---

## 6. Granular Categorization Improvements

### Current Category Distribution
1. **developmentAgents**: 26 agents (too broad)
2. **dataAgents**: 9 agents
3. **infrastructureAgents**: 10 agents
4. **securityAgents**: 8 agents
5. **aiMlAgents**: 6 agents
6. **mcpAgents**: 6 agents
7. **documentationAgents**: 7 agents
8. **researchAgents**: 10 agents
9. **supportAgents**: 31 agents (too broad, includes duplicates)
10. **businessAgents**: 4 agents

### Proposed New Categories

#### Split developmentAgents into:
1. **frontendAgents** (8 agents)
   - Frontend Developer
   - Nextjs Architecture Expert
   - React Performance Optimization
   - React Performance Optimizer
   - Ui Ux Designer
   - Web Accessibility Checker
   - Web Vitals Optimizer
   - CLI UI Designer

2. **backendAgents** (10 agents)
   - Backend Architect
   - Golang Pro
   - Python Pro
   - Rust Pro
   - Typescript Pro
   - Javascript Pro
   - Shell Scripting Pro
   - Graphql Architect
   - Graphql Performance Optimizer
   - Graphql Security Specialist

3. **mobileAgents** (5 agents)
   - Mobile Developer
   - Ios Developer
   - Flutter Specialist (move from codingAgents)
   - Swift Specialist (move from codingAgents)
   - Flutter Go Reviewer

4. **codeQualityAgents** (8 agents)
   - Code Reviewer
   - Review Agent
   - Architect Review
   - Debugger
   - Error Detective
   - Legacy Modernizer
   - Architecture Modernizer
   - Unused Code Cleaner

#### Consolidate supportAgents into:
1. **testingAgents** (4 agents)
   - Test Engineer
   - Test Automator
   - Load Testing Specialist
   - TDD Coding Agent (move from codingAgents)

2. **utilityAgents** (11 agents, after removing duplicates)
   - Context Manager
   - Task Decomposition Expert
   - Command Expert
   - Connection Agent
   - Metadata Agent
   - Tag Agent
   - Document Structure Analyzer
   - URL Link Extractor
   - Project Supervisor Orchestrator
   - Performance Engineer
   - Performance Profiler

3. **workflowAgents** (5 agents)
   - Git Flow Manager
   - Dependency Manager
   - Dx Optimizer
   - Product Strategist
   - Business Analyst

### Result After Reorganization
- **More focused categories**: 16 categories vs 10
- **Better agent discovery**: Easier to find right agent
- **Clearer responsibilities**: Less overlap
- **Future scalability**: Room for new agents

---

## 7. Documentation Impact

### Files Requiring Updates

#### Critical Updates (Must Update)
1. **/Users/brent/git/cc-orchestra/CLAUDE.md**
   - Update agent count (15 → 122 after dedup)
   - Update category structure
   - Add missing agents to descriptions
   - Update haiku agent list

2. **/Users/brent/git/cc-orchestra/docs/ARMY_ROSTER.md**
   - Complete rewrite of agent roster
   - Organize by new categories
   - Document model assignments (haiku vs sonnet-4.5)
   - Add specialties and use cases

3. **/Users/brent/git/cc-orchestra/README.md**
   - Update quick start examples
   - Update agent count
   - Refresh feature highlights

4. **/Users/brent/git/cc-orchestra/config/orchestra-config.json**
   - Remove duplicates
   - Add missing agents
   - Switch models to haiku
   - Update agent types

#### Important Updates
5. **/Users/brent/git/cc-orchestra/docs/TECHNICAL_OVERVIEW.md**
   - Update architecture diagrams
   - Document new categorization
   - Update model routing section

6. **/Users/brent/git/cc-orchestra/docs/EXECUTIVE_SUMMARY.md**
   - Update agent count
   - Update cost estimates
   - Refresh capabilities section

7. **/Users/brent/.claude/CLAUDE.md** (Global instructions)
   - Update orchestra roster reference
   - Update agent count in examples
   - Add new agent categories to examples

#### Optional Updates
8. **/Users/brent/git/cc-orchestra/docs/DEEP_DIVE.md**
   - Add reconciliation notes
   - Document optimization decisions

9. **/Users/brent/git/cc-orchestra/docs/DEPLOYMENT_STATUS.md**
   - Update ccproxy model usage stats
   - Add haiku routing information

---

## 8. Implementation Steps

### Phase 1: Duplicate Removal (Day 1, 30 minutes)
**Risk**: Low | **Impact**: High | **Priority**: Critical

1. ✅ **Backup current config**
   ```bash
   cp config/orchestra-config.json config/orchestra-config.json.backup-$(date +%Y%m%d)
   ```

2. ✅ **Remove duplicate entries**
   - Delete lines 2073-2327 in supportAgents section
   - Verify JSON validity: `node -e "JSON.parse(require('fs').readFileSync('config/orchestra-config.json'))"`

3. ✅ **Test config loading**
   ```bash
   node src/orchestra-conductor.js validate
   ```

4. ✅ **Commit changes**
   ```bash
   git add config/orchestra-config.json
   git commit -m "fix: remove 15 duplicate agent entries in supportAgents section"
   ```

### Phase 2: Add Missing Agents (Day 1-2, 2 hours)
**Risk**: Low | **Impact**: Medium | **Priority**: High

1. ✅ **Add 7 missing agents** to appropriate sections (agent-overview.md excluded as documentation):
   - architect-review → developmentAgents
   - flutter-go-reviewer → developmentAgents
   - supabase-schema-architect → dataAgents
   - supabase-realtime-optimizer → dataAgents
   - unused-code-cleaner → supportAgents (with haiku model)
   - web-vitals-optimizer → developmentAgents

2. ✅ **Configure each agent** with:
   - Appropriate type (reviewer, coder, etc.)
   - Model selection (sonnet-4.5 or haiku)
   - Role description from agent file
   - Specialties (parse from agent file)
   - autonomousAuthority settings

3. ✅ **Test agent loading**
   ```bash
   node src/orchestra-conductor.js list-agents
   ```

4. ✅ **Commit changes**
   ```bash
   git add config/orchestra-config.json
   git commit -m "feat: add 8 missing agents from agent files"
   ```

### Phase 3: Model Optimization (Day 2-3, 3 hours)
**Risk**: Medium | **Impact**: High (cost savings) | **Priority**: High

1. ✅ **Switch 30 agents to haiku model**
   - Update model field: "sonnet-4.5" → "haiku"
   - Document rationale in commit message
   - Priority: Documentation agents first (highest volume)

2. ✅ **Add ccproxyMapping for haiku agents**
   ```json
   "ccproxyMapping": {
     "apiAlias": "claude-3-haiku",
     "ollama": "qwen-fast:latest",
     "phase": "Phase 1 - Lightweight"
   }
   ```

3. ✅ **Test haiku routing**
   ```bash
   # Verify ccproxy handles haiku alias
   curl -s https://coder.visiquate.com/health
   ```

4. ✅ **Monitor first week**
   - Track response quality
   - Monitor cost reduction
   - Rollback if quality issues

5. ✅ **Commit changes**
   ```bash
   git add config/orchestra-config.json
   git commit -m "perf: optimize 30 agents to use haiku model (estimated $300-450/mo savings)"
   ```

### Phase 4: Type Refinement (Day 3-4, 4 hours)
**Risk**: Medium | **Impact**: Medium | **Priority**: Medium

1. ✅ **Update agent types** (high priority first):
   - Security agents → security-auditor
   - Testing agents → test-automator
   - Infrastructure agents → deployment-engineer
   - Review agents → reviewer
   - Research agents → researcher
   - Database agents → backend-dev

2. ✅ **Validate type mappings**
   ```bash
   # Ensure types match Claude Code's available types
   node src/orchestra-conductor.js validate-types
   ```

3. ✅ **Test agent selection logic**
   - Verify orchestrator selects correct agents
   - Test sample workflows

4. ✅ **Commit changes**
   ```bash
   git add config/orchestra-config.json
   git commit -m "refactor: use more specific agent types for better orchestration"
   ```

### Phase 5: Reorganization (Day 4-5, 5 hours)
**Risk**: High | **Impact**: High | **Priority**: Medium

1. ✅ **Create new categories** in config:
   - frontendAgents
   - backendAgents
   - mobileAgents
   - codeQualityAgents
   - testingAgents
   - utilityAgents
   - workflowAgents

2. ✅ **Migrate agents** to new categories:
   - Move agents from developmentAgents
   - Move agents from supportAgents
   - Update cross-references

3. ✅ **Update category metadata**:
   - Add description for each category
   - Document when to use category
   - Add example use cases

4. ✅ **Test orchestration**:
   ```bash
   node src/orchestra-conductor.js "Build a React app with Go backend"
   # Verify correct agents selected from new categories
   ```

5. ✅ **Commit changes**
   ```bash
   git add config/orchestra-config.json
   git commit -m "refactor: reorganize agents into 16 focused categories"
   ```

### Phase 6: Documentation (Day 5-6, 6 hours)
**Risk**: Low | **Impact**: High | **Priority**: High

1. ✅ **Update ARMY_ROSTER.md**
   - Complete agent listing
   - Organized by new categories
   - Include model assignments
   - Add use case examples

2. ✅ **Update CLAUDE.md**
   - New agent count
   - Category structure
   - Haiku vs Sonnet guidance
   - Updated examples

3. ✅ **Update README.md**
   - Quick start examples
   - Feature highlights
   - Performance metrics

4. ✅ **Update technical docs**
   - TECHNICAL_OVERVIEW.md
   - EXECUTIVE_SUMMARY.md
   - DEPLOYMENT_STATUS.md

5. ✅ **Generate migration guide**
   - Create RECONCILIATION_CHANGELOG.md
   - Document breaking changes (if any)
   - Provide migration examples

6. ✅ **Commit documentation**
   ```bash
   git add docs/ CLAUDE.md README.md
   git commit -m "docs: comprehensive update for orchestra v2.1.0 reconciliation"
   ```

### Phase 7: Validation & Rollout (Day 6-7, 3 hours)
**Risk**: Low | **Impact**: Critical | **Priority**: Critical

1. ✅ **Comprehensive testing**
   ```bash
   # Test all major workflows
   npm run test:integration

   # Test agent selection
   node src/orchestra-conductor.js "Test various requirements"

   # Validate config integrity
   npm run validate:config
   ```

2. ✅ **Performance testing**
   - Benchmark haiku agents
   - Compare response times
   - Verify cost savings

3. ✅ **Create release**
   ```bash
   git tag v2.1.0
   git push origin v2.1.0
   ```

4. ✅ **Monitor rollout**
   - Watch for issues in first week
   - Collect feedback
   - Track cost metrics

5. ✅ **Document results**
   - Create post-mortem report
   - Document lessons learned
   - Update best practices

---

## 9. Risk Assessment

### High Risk Items
1. **Category reorganization** (Phase 5)
   - **Risk**: Breaking agent selection logic
   - **Mitigation**: Comprehensive testing before commit
   - **Rollback**: Revert to backup config

2. **Type changes** (Phase 4)
   - **Risk**: Claude Code might not recognize new types
   - **Mitigation**: Validate against Claude Code's type list
   - **Rollback**: Revert specific agents to "coder"

### Medium Risk Items
3. **Haiku model switches** (Phase 3)
   - **Risk**: Quality degradation for some agents
   - **Mitigation**: Monitor first week, switch back if needed
   - **Rollback**: Switch back to sonnet-4.5

4. **Adding missing agents** (Phase 2)
   - **Risk**: Agents might have undocumented dependencies
   - **Mitigation**: Test each agent individually
   - **Rollback**: Remove problematic agents

### Low Risk Items
5. **Duplicate removal** (Phase 1)
   - **Risk**: Minimal, duplicates are identical
   - **Mitigation**: Keep second occurrence as fallback
   - **Rollback**: Restore from backup

6. **Documentation updates** (Phase 6)
   - **Risk**: Documentation out of sync
   - **Mitigation**: Update docs alongside code changes
   - **Rollback**: Git revert

---

## 10. Success Metrics

### Phase 1 (Duplicate Removal)
- ✅ Config file size reduced by ~512 lines
- ✅ No duplicate agent names in config
- ✅ JSON validates successfully
- ✅ All tests pass

### Phase 2 (Missing Agents)
- ✅ 7 new agents added to config
- ✅ All agent files matched in config (106 agents)
- ✅ Agents load successfully
- ✅ Specialties parsed correctly

### Phase 3 (Model Optimization)
- ✅ 30 agents switched to haiku
- ✅ Response quality maintained (spot check 10 agents)
- ✅ Cost reduction: $300-450/month (track via ccproxy logs)
- ✅ Response time improvement: 20-30% faster for haiku agents

### Phase 4 (Type Refinement)
- ✅ Generic "coder" usage reduced from 53% to <20%
- ✅ All security agents use security-auditor type
- ✅ All testing agents use test-automator type
- ✅ Agent selection accuracy improved (measure via user feedback)

### Phase 5 (Reorganization)
- ✅ 16 focused categories created
- ✅ No agents in wrong category
- ✅ Orchestration selects correct agents 95%+ of time
- ✅ User agent discovery time reduced (subjective feedback)

### Phase 6 (Documentation)
- ✅ All critical docs updated
- ✅ Examples reflect new structure
- ✅ No broken references
- ✅ Migration guide complete

### Phase 7 (Validation)
- ✅ All tests pass
- ✅ No production issues in first week
- ✅ Cost savings confirmed
- ✅ User satisfaction maintained/improved

---

## 11. Timeline Summary

| Phase | Duration | Days | Priority | Risk |
|-------|----------|------|----------|------|
| 1. Duplicate Removal | 30 min | Day 1 | Critical | Low |
| 2. Add Missing Agents | 2 hours | Day 1-2 | High | Low |
| 3. Model Optimization | 3 hours | Day 2-3 | High | Medium |
| 4. Type Refinement | 4 hours | Day 3-4 | Medium | Medium |
| 5. Reorganization | 5 hours | Day 4-5 | Medium | High |
| 6. Documentation | 6 hours | Day 5-6 | High | Low |
| 7. Validation & Rollout | 3 hours | Day 6-7 | Critical | Low |
| **Total** | **23.5 hours** | **7 days** | - | - |

**Recommended Approach**: Execute phases 1-3 immediately (high ROI, low risk), then phases 4-7 over next 2 weeks.

---

## 12. Cost-Benefit Analysis

### Current State (Monthly)
- **Total Agents**: 129 (with duplicates) → 122 (after dedup)
- **Sonnet-4.5 Agents**: 116 agents
- **Haiku Agents**: 6 agents (mostly Phase 1 lightweight)
- **Estimated Monthly Cost**: $850-1,200 (assuming 25 invocations/agent)
  - Sonnet-4.5: $0.60/1K tokens avg per agent
  - Haiku: $0.08/1K tokens avg per agent

### Proposed State (Monthly)
- **Total Agents**: 122 (after dedup + additions)
- **Sonnet-4.5 Agents**: 86 agents
- **Haiku Agents**: 36 agents (30 converted + 6 existing)
- **Estimated Monthly Cost**: $550-800
  - Sonnet-4.5: 86 agents × $0.60 = $516
  - Haiku: 36 agents × $0.08 = $28.80

### Savings
- **Monthly**: $300-450 (35-40% reduction)
- **Annual**: $3,600-5,400
- **Implementation Cost**: 23.5 hours × $75/hr = $1,762
- **ROI**: 4-6 months payback period

### Non-Financial Benefits
- **Faster response times**: Haiku agents 30-50% faster
- **Better organization**: 16 focused categories vs 10 broad
- **Improved accuracy**: Specific types enable better agent selection
- **Easier maintenance**: Clear categorization, no duplicates
- **Future scalability**: Room for 50+ more agents

---

## 13. Rollback Plan

### Quick Rollback (Phases 1-3)
```bash
# Restore backup config
cp config/orchestra-config.json.backup-20251110 config/orchestra-config.json

# Verify
node src/orchestra-conductor.js validate

# Commit rollback
git add config/orchestra-config.json
git commit -m "revert: rollback to pre-reconciliation config"
```

### Selective Rollback (Phases 4-5)
```bash
# Revert specific commits
git revert HEAD~N  # N = number of commits to revert

# Or cherry-pick good changes
git checkout backup-branch -- config/orchestra-config.json
git cherry-pick <good-commit-hash>
```

### Full Rollback (All Phases)
```bash
# Reset to tag
git reset --hard v2.0.0

# Force push (if needed)
git push origin main --force-with-lease
```

---

## 14. Appendix: Agent Type Mapping Reference

### Claude Code Agent Types (Official)
Based on context from CLAUDE.md:

| Type | Purpose | Example Agents | Model Preference |
|------|---------|----------------|------------------|
| `coder` | General-purpose coding | Generic tasks | sonnet-4.5 |
| `test-automator` | Testing specialists | Test Engineer, Test Automator | sonnet-4.5 |
| `technical-writer` | Documentation | Documentation Expert, API Documenter | haiku |
| `deployment-engineer` | DevOps/Infrastructure | DevOps Engineer, Deployment Engineer | sonnet-4.5 |
| `security-auditor` | Security review | Security Auditor, Security Engineer | sonnet-4.5 |
| `researcher` | Analysis/research | Technical Researcher, Academic Researcher | sonnet-4.5 |
| `ios-developer` | iOS/mobile | iOS Developer, Swift Specialist | sonnet-4.5 |
| `backend-dev` | Backend specialists | Go Specialist, Rust Specialist | sonnet-4.5 |
| `mobile-developer` | Cross-platform mobile | Flutter Specialist, Mobile Developer | sonnet-4.5 |
| `python-expert` | Python specialists | Python Specialist, Python Pro | sonnet-4.5 |
| `reviewer` | Code review | Code Reviewer, Architect Review | sonnet-4.5 |
| `planner` | Planning/coordination | Task Decomposition Expert | sonnet-4.5 |
| `ux-designer` | UI/UX design | UI/UX Designer | sonnet-4.5 |
| `debugger` | Debugging | Debugger, Error Detective | sonnet-4.5 |
| `system-architect` | Architecture | Chief Architect, Database Architect | opus/sonnet-4.5 |

---

## 15. Next Steps

### Immediate Actions (Week 1)
1. **Review this plan** with team/stakeholders
2. **Approve phases** to execute (recommend 1-3 immediately)
3. **Backup current config** before any changes
4. **Execute Phase 1** (duplicate removal) - 30 minutes
5. **Execute Phase 2** (add 7 missing agents) - 2 hours
6. **Execute Phase 3** (haiku optimization) - 3 hours
7. **Monitor results** for first week

### Short-term (Weeks 2-3)
1. **Execute Phase 4** (type refinement) if Phase 3 successful
2. **Execute Phase 5** (reorganization) - requires more careful testing
3. **Execute Phase 6** (documentation updates)
4. **Execute Phase 7** (validation and rollout)
5. **Collect metrics** and validate cost savings

### Long-term (Months 1-3)
1. **Monitor cost reduction** - track actual vs. estimated savings
2. **Gather user feedback** - agent selection accuracy
3. **Optimize further** - identify additional haiku candidates
4. **Add more agents** - leverage new category structure
5. **Document lessons learned** - update best practices

---

## 16. Questions & Answers

### Q: Why not switch all agents to haiku?
**A**: Haiku excels at lightweight, well-defined tasks but struggles with complex reasoning. Sonnet-4.5 (and qwen2.5-coder via ccproxy) is essential for agents requiring:
- Multi-step decision making
- Complex code generation
- Security analysis requiring threat modeling
- Architecture design
- Performance optimization requiring deep analysis

### Q: What if haiku agents don't perform well?
**A**: Each phase includes testing and monitoring. If quality degrades:
1. Identify problematic agents (check logs/user feedback)
2. Switch back to sonnet-4.5 immediately
3. Document why agent needs more powerful model
4. Update categorization for similar agents

### Q: Will this break existing workflows?
**A**: Phases 1-3 are transparent to users (config changes only). Phases 4-5 might affect agent selection logic, which is why comprehensive testing is required before rollout.

### Q: How do we handle new agents in the future?
**A**: Follow the framework established here:
1. Assess complexity: lightweight → haiku, complex → sonnet-4.5
2. Choose specific type: avoid "coder" unless truly general-purpose
3. Place in appropriate category: use new 16-category structure
4. Document specialties: ensure discoverability

### Q: What if ccproxy routing fails for haiku?
**A**: ccproxy already handles haiku routing (`claude-3-haiku` → `qwen-fast:latest`). If issues arise:
1. Check ccproxy health endpoint
2. Verify qwen-fast is loaded in Ollama
3. Fallback to Claude API haiku as temporary measure
4. Monitor and resolve routing issues

---

**Document Status**: READY FOR REVIEW
**Approval Required**: Chief Architect or Project Lead
**Estimated Implementation**: 7 days (23.5 hours)
**Estimated ROI**: $3,600-5,400/year (payback in 4-6 months)
