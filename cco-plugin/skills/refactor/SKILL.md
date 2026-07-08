# Refactor: Safe Refactoring

## Trigger
`/refactor <file or component>`

Orchestrates safe refactoring with test coverage verification and incremental validation.

## Purpose
The Refactor skill ensures:
- Existing test coverage is verified before starting
- Refactoring happens incrementally with test validation after each step
- No regression of functionality
- Code quality improvements without behavior changes
- Clear before/after comparisons

## Workflow

### Phase 1: Analyze Current State

1. **Examine code to refactor**
   - Complexity metrics
   - Code smells
   - Duplication
   - Structure issues

2. **Check test coverage**
   ```bash
   # Verify tests exist
   cco knowledge search "tests for [component]"

   # Check coverage
   pytest [component] --cov=[component] --cov-report=term
   ```

3. **Identify refactoring needs**
   - Extract functions
   - Remove duplication
   - Simplify logic
   - Improve naming
   - Better structure

### Phase 2: Query Knowledge Base

1. **Retrieve architecture decisions**
   ```bash
   cco knowledge search "architecture for [component]"
   ```

2. **Retrieve performance constraints**
   ```bash
   cco knowledge search "performance targets"
   ```

3. **Retrieve related refactorings**
   ```bash
   cco knowledge search "previous refactorings [component]"
   ```

### Phase 3: Verify Test Coverage

**Spawn Test Engineer** (Sonnet)

1. **Analyze existing tests**
   - Coverage report
   - Test quality
   - Gap analysis

2. **Ensure minimum 80% coverage**
   ```
   If coverage < 80%:
   → Spawn TDD Agent
   → Write additional tests
   → Reach target coverage
   → Only then proceed
   ```

3. **Document coverage baseline**
   ```bash
   cco knowledge store "Refactor Baseline: [component]
   - Current coverage: [%]
   - Test count: [N]
   - Key scenarios: [list]" \
     --type status \
     --agent test-engineer
   ```

4. **Run full test suite**
   - All tests must pass
   - No failures before starting

### Phase 4: Design Refactoring Plan

**Spawn Architect Review** (Sonnet)

1. **Plan refactoring steps**
   - Order of changes
   - Dependency analysis
   - Risk assessment

2. **Identify refactoring patterns**
   - Extract method
   - Extract class
   - Rename variables
   - Remove duplication
   - Simplify conditions

3. **Document plan**
   ```bash
   cco knowledge store "Refactor Plan: [component]
   Steps:
   1. [Refactoring step 1]
   2. [Refactoring step 2]
   3. [Refactoring step 3]

   Risks: [identified risks]" \
     --type decision \
     --agent architect-review
   ```

### Phase 5: Execute Refactoring (Incremental)

**Spawn Language Specialist** (Haiku)

**Key Principle**: Small changes, validate after each step

1. **Step 1: Extract first function**
   ```python
   # BEFORE
   def process_user_data(user):
       if user.age < 18:
           return "Minor"
       if user.status == "active":
           send_email(user.email)
       return user.name.upper()

   # AFTER - Extract validation
   def is_minor(user):
       return user.age < 18

   def process_user_data(user):
       if is_minor(user):
           return "Minor"
       if user.status == "active":
           send_email(user.email)
       return user.name.upper()
   ```

2. **Run tests** (after each small change)
   ```bash
   pytest tests/ -v
   # All tests pass? Continue.
   # Test fails? Revert change, try different approach.
   ```

3. **Step 2: Extract next function**
   ```python
   def notify_active_user(user):
       if user.status == "active":
           send_email(user.email)

   def process_user_data(user):
       if is_minor(user):
           return "Minor"
       notify_active_user(user)
       return user.name.upper()
   ```

4. **Run tests again**

5. **Step 3: Continue with remaining refactorings**

6. **After each change**:
   ```bash
   # 1. Run tests
   pytest tests/ -v

   # 2. Check coverage hasn't dropped
   pytest --cov=[component] --cov-report=term

   # 3. Verify behavior is unchanged
   # (test results should be identical)

   # 4. Store progress
   cco knowledge store "Refactor Progress: [step]" \
     --type status \
     --agent language-specialist
   ```

### Phase 6: Validate After Each Phase

**Spawn Test Engineer** (Sonnet) - After every 2-3 changes

1. **Run full test suite**
   ```bash
   pytest tests/ -v --tb=short
   ```

2. **Check code quality**
   ```bash
   pylint [file]  # or language-specific linter
   ```

3. **Measure complexity reduction**
   ```bash
   # Before refactoring: Cyclomatic complexity 15
   # After step 1: Cyclomatic complexity 12
   # After step 2: Cyclomatic complexity 10
   # Goal: < 10
   ```

4. **Verify coverage maintained**
   ```bash
   pytest --cov=[component]
   # Coverage before: 85%
   # Coverage after: 85% (or better)
   ```

### Phase 7: Code Review

**Spawn Code Reviewer** (Sonnet) + **Architect Review** (Sonnet)

Run in parallel when refactoring is complete:

1. **Code Reviewer checks**:
   - Code clarity improved?
   - Naming is better?
   - Duplication removed?
   - Logic simplified?

2. **Architect Review checks**:
   - Architecture maintained?
   - No unexpected coupling?
   - Design patterns respected?
   - Scalability preserved?

3. **Both approve or request changes**

### Phase 8: Performance Validation

**Spawn Performance Engineer** (Sonnet)

1. **Compare performance metrics before/after**
   ```
   Metric: Response time for /api/users
   - Before refactoring: 250ms
   - After refactoring: 245ms
   - Improvement: 2% (acceptable)

   Metric: Memory usage
   - Before: 120MB
   - After: 120MB
   - Regression: None (good)
   ```

2. **Run performance tests**
   ```bash
   pytest tests/performance/ -v
   ```

3. **Benchmark critical paths**
   ```bash
   python -m cProfile -s cumulative main.py
   ```

### Phase 9: Final Validation

1. **All tests pass** ✓
2. **Coverage maintained** ✓
3. **Code quality improved** ✓
4. **Performance maintained or improved** ✓
5. **Architecture respected** ✓
6. **Code reviewed and approved** ✓

**Store completion**:
```bash
cco knowledge store "Refactor Complete: [component]
- Coverage: [before %] → [after %]
- Cyclomatic Complexity: [before] → [after]
- Lines of Code: [before] → [after]
- Test results: All [N] tests passing
- Reviewers: Approved by [names]" \
  --type completion \
  --agent language-specialist
```

## Agent Spawning Pattern

```
Refactor Skill
    │
    ├─→ Test Engineer (Sonnet)
    │      Verify baseline coverage
    │      Monitor coverage during refactoring
    │      Final coverage validation
    │
    ├─→ Architect Review (Sonnet)
    │      Design refactoring plan
    │      Validate architecture compliance
    │
    ├─→ Language Specialist (Haiku)
    │      Execute incremental changes
    │      Run tests after each change
    │      Implement refactorings
    │
    ├─→ Code Reviewer (Sonnet)
    │      Review refactored code
    │      Check for improvements
    │
    └─→ Performance Engineer (Sonnet)
           Validate performance
           Compare before/after metrics
```

## Refactoring Patterns

### Extract Method
```python
# BEFORE
def calculate_total(items):
    subtotal = sum(item.price for item in items)
    tax = subtotal * 0.08
    shipping = 10 if subtotal < 50 else 0
    return subtotal + tax + shipping

# AFTER
def calculate_tax(subtotal):
    return subtotal * 0.08

def calculate_shipping(subtotal):
    return 10 if subtotal < 50 else 0

def calculate_total(items):
    subtotal = sum(item.price for item in items)
    return subtotal + calculate_tax(subtotal) + calculate_shipping(subtotal)
```

### Remove Duplication
```python
# BEFORE
def get_active_users():
    users = db.query(User).filter(User.status == "active").all()
    return [u.to_dict() for u in users]

def get_premium_users():
    users = db.query(User).filter(User.status == "premium").all()
    return [u.to_dict() for u in users]

# AFTER
def get_users_by_status(status):
    users = db.query(User).filter(User.status == status).all()
    return [u.to_dict() for u in users]

def get_active_users():
    return get_users_by_status("active")

def get_premium_users():
    return get_users_by_status("premium")
```

### Simplify Conditional Logic
```python
# BEFORE
def is_eligible_for_discount(user):
    if user.is_premium:
        if user.spent_amount > 1000:
            if user.days_member > 365:
                return True
            else:
                return False
        else:
            return False
    else:
        return False

# AFTER
def is_eligible_for_discount(user):
    return (user.is_premium and
            user.spent_amount > 1000 and
            user.days_member > 365)
```

### Extract Class
```python
# BEFORE
class User:
    def __init__(self, name, email, street, city, zip):
        self.name = name
        self.email = email
        self.street = street
        self.city = city
        self.zip = zip

    def format_address(self):
        return f"{self.street}, {self.city} {self.zip}"

# AFTER
class Address:
    def __init__(self, street, city, zip):
        self.street = street
        self.city = city
        self.zip = zip

    def format(self):
        return f"{self.street}, {self.city} {self.zip}"

class User:
    def __init__(self, name, email, address):
        self.name = name
        self.email = email
        self.address = address
```

## Metrics to Track

| Metric | Goal | Tool |
|--------|------|------|
| **Cyclomatic Complexity** | < 10 per function | radon, pylint |
| **Lines per Function** | < 30 lines | wc, radon |
| **Duplication** | < 3% | pylint, radon |
| **Test Coverage** | Maintain 85%+ | pytest-cov |
| **Performance** | No regression | cProfile, benchmarks |

## Risk Mitigation

### Before Starting
- [ ] All tests passing
- [ ] Coverage baseline documented
- [ ] Refactoring plan documented
- [ ] Architecture decision reviewed
- [ ] Performance baseline measured

### During Refactoring
- [ ] Tests run after each small change
- [ ] No large unvalidated changes
- [ ] Progress tracked in knowledge base
- [ ] Incremental commits (not one large change)

### After Completion
- [ ] All tests pass
- [ ] Coverage maintained or improved
- [ ] Code review approved
- [ ] Performance validated
- [ ] Architect review completed

## Best Practices

1. **Small incremental changes** - 1-2 refactorings per commit
2. **Test after each change** - Validate before proceeding
3. **Measure complexity** - Know what you're improving
4. **Document decisions** - Why this refactoring?
5. **Preserve behavior** - Code should work identically
6. **Review thoroughly** - Different eyes catch issues
7. **Performance validate** - Refactoring shouldn't slow things down
8. **Architecture check** - Don't violate design patterns
9. **Git branching** - Isolated feature branch for refactoring
10. **Clear commits** - Logical, reviewable commit messages

## Knowledge Base Integration

### Store Baseline
```bash
cco knowledge store "Refactor Baseline: [component]
- Lines: [N]
- Functions: [N]
- Complexity: [avg]
- Coverage: [%]
- Performance: [metrics]" \
  --type status \
  --agent test-engineer
```

### Track Progress
```bash
cco knowledge store "Refactor Step [N]: [description]
- Changes: [files changed]
- Tests: All passing
- Coverage: [%]" \
  --type status \
  --agent language-specialist
```

### Document Completion
```bash
cco knowledge store "Refactor Complete: [component]
- Improved: [metrics improved]
- Maintained: [metrics maintained]
- Tests: All [N] passing
- Coverage: [%]
- Approved: [reviewers]" \
  --type completion \
  --agent language-specialist
```

## Output Format

Return comprehensive refactoring report with:
- Baseline metrics (before)
- Final metrics (after)
- Changes made (by category: extract, remove duplication, simplify, etc.)
- Test results (all passing)
- Coverage analysis (maintained or improved)
- Code review findings (if any)
- Performance impact
- Architecture compliance
- Lessons learned
- Knowledge base references
