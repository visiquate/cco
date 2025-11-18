# Test Suite Index - Critical Issues Verification

**Quick index of all test documentation and resources.**

---

## Start Here

### For Running Tests (Impatient)
```bash
/tmp/comprehensive_test_suite.sh
```
**Time**: 8-10 minutes | **Output**: Pass/Fail result

### For Understanding Tests (Detailed)
Read: `TESTING_INSTRUCTIONS.md`

### For Quick Reference (Middle Ground)
Read: `QUICK_TEST_GUIDE.md`

---

## All Test Documents

### 1. TESTING_INSTRUCTIONS.md
**Purpose**: Complete guide for executing tests
**Contents**: 
- Prerequisites and setup
- Three testing options (automated, manual, quick)
- Understanding results
- Troubleshooting guide
- Performance recording

**When to read**: Before running any tests

---

### 2. COMPREHENSIVE_TEST_PLAN.md
**Purpose**: Detailed test specification (15+ tests)
**Contents**:
- 4 test suites with detailed procedures
- Acceptance criteria for each test
- Expected results
- Failure investigation guide
- Test report template

**When to read**: During test execution or if tests fail

---

### 3. QUICK_TEST_GUIDE.md
**Purpose**: Fast reference for quick testing
**Contents**:
- TL;DR quick start
- Individual test procedures
- Debugging tips
- Expected results
- Performance baseline table
- Testing checklist

**When to read**: For quick verification or troubleshooting

---

### 4. TEST_VERIFICATION_CHECKLIST.md
**Purpose**: Detailed checklist for comprehensive verification
**Contents**:
- Pre-test setup checklist (15 items)
- 15+ detailed tests with pass/fail boxes
- Evidence collection guidance
- Tester sign-off section
- Performance recording template

**When to read**: For detailed manual verification and documentation

---

### 5. TEST_DELIVERABLES_SUMMARY.md
**Purpose**: Overview of all test deliverables
**Contents**:
- Summary of each document
- What's being tested (3 issues)
- Test execution flow chart
- Expected results
- Troubleshooting quick reference
- Document reference map

**When to read**: To understand overall test approach

---

### 6. /tmp/comprehensive_test_suite.sh
**Purpose**: Automated executable test script
**Status**: Ready to run
**Tests**: 11 core tests covering all issues
**Duration**: 8-10 minutes
**Output**: Color-coded results + summary

**Usage**:
```bash
/tmp/comprehensive_test_suite.sh
```

---

## Three Critical Issues Being Tested

### Issue #1: Ctrl+C Shutdown Takes 4+ Seconds
**Target**: Shutdown within 1-2 seconds
**Tests**: 5 (response time, message, port release, zombies, stress)
**Success Criterion**: All tests pass, < 2000ms shutdown time
**Location in docs**: 
- COMPREHENSIVE_TEST_PLAN.md - TEST SUITE 1
- QUICK_TEST_GUIDE.md - Test 1
- TEST_VERIFICATION_CHECKLIST.md - TEST SUITE 1

---

### Issue #2: Terminal Prompt Not Displaying
**Target**: Terminal functionality operational
**Tests**: 4 (health endpoint, WebSocket, browser UI, no errors)
**Success Criterion**: All tests pass, terminal accessible
**Location in docs**:
- COMPREHENSIVE_TEST_PLAN.md - TEST SUITE 3
- QUICK_TEST_GUIDE.md - Test 3
- TEST_VERIFICATION_CHECKLIST.md - TEST SUITE 3

---

### Issue #3: Logging Spam Every 5 Seconds
**Target**: Reduce to startup only (≤ 6 messages in 15 seconds)
**Tests**: 4 (message count, pattern, log levels, stability)
**Success Criterion**: All tests pass, ≤ 6 total spam messages
**Location in docs**:
- COMPREHENSIVE_TEST_PLAN.md - TEST SUITE 2
- QUICK_TEST_GUIDE.md - Test 2
- TEST_VERIFICATION_CHECKLIST.md - TEST SUITE 2

---

## Quick Navigation

### "I want to..."

**...run all tests quickly**
→ `/tmp/comprehensive_test_suite.sh`

**...understand what's being tested**
→ `TEST_DELIVERABLES_SUMMARY.md`

**...get detailed test specifications**
→ `COMPREHENSIVE_TEST_PLAN.md`

**...run individual tests**
→ `QUICK_TEST_GUIDE.md`

**...document detailed results**
→ `TEST_VERIFICATION_CHECKLIST.md`

**...understand test process**
→ `TESTING_INSTRUCTIONS.md`

**...find specific test procedures**
→ `COMPREHENSIVE_TEST_PLAN.md` (search test number)

**...troubleshoot a failing test**
→ `COMPREHENSIVE_TEST_PLAN.md` (Failure Investigation Guide)

**...record performance baseline**
→ `TESTING_INSTRUCTIONS.md` (Performance Recording section)

---

## File Organization

```
/Users/brent/git/cc-orchestra/cco/
├── TEST_SUITE_INDEX.md (THIS FILE)
├── TESTING_INSTRUCTIONS.md ..................... How to run tests
├── COMPREHENSIVE_TEST_PLAN.md ................. Detailed specifications
├── QUICK_TEST_GUIDE.md ......................... Fast reference
├── TEST_VERIFICATION_CHECKLIST.md ............. Detailed checklist
├── TEST_DELIVERABLES_SUMMARY.md .............. Overview
└── [source files being tested] ................. src/main.rs, src/server.rs, etc.

/tmp/
└── comprehensive_test_suite.sh ................. Automated test runner
```

---

## Expected Test Duration

| Option | Duration | Effort | Documentation |
|--------|----------|--------|---------------|
| Automated script | 8-10 min | Low | Automatic |
| Quick manual | 15-20 min | Medium | Basic |
| Detailed manual | 30-45 min | High | Comprehensive |

---

## Success Criteria

**All three issues must be fixed**:

```
[  ] Issue #1: Shutdown < 2 seconds
     └─ Status: ______

[  ] Issue #2: Terminal prompt displays
     └─ Status: ______

[  ] Issue #3: Logging spam ≤ 6 messages in 15s
     └─ Status: ______

OVERALL: [  ] ALL PASS [  ] SOME FAIL [  ] REVIEW NEEDED
```

---

## Test Summary Table

| Suite | Issue | Tests | Pass Criteria |
|-------|-------|-------|---------------|
| 1 | Shutdown | 5 | Shutdown < 2000ms |
| 2 | Logging | 4 | Spam ≤ 6 in 15s |
| 3 | Terminal | 4 | Endpoints respond |
| 4 | Integration | 1 | Full lifecycle works |

---

## Document Cross-References

### Finding a Specific Test
1. Identify issue number (1, 2, 3, or 4)
2. Find test suite in COMPREHENSIVE_TEST_PLAN.md
3. Follow detailed procedure
4. Document results in TEST_VERIFICATION_CHECKLIST.md

### Finding a Specific Error
1. Check error message in test output
2. Search COMPREHENSIVE_TEST_PLAN.md for "Failure Investigation"
3. Follow debugging steps
4. Check specific component code

### Recording Results
1. Complete TEST_VERIFICATION_CHECKLIST.md
2. Record performance metrics in TESTING_INSTRUCTIONS.md
3. Archive test logs: `mkdir -p /tmp/test_logs_archive; mv /tmp/test_*.log /tmp/test_logs_archive/`

---

## Quick Commands Reference

### Build and Test
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release 2>&1 | tail -3
/tmp/comprehensive_test_suite.sh
```

### Manual Shutdown Test
```bash
export NO_BROWSER=1
timeout 5 cargo run --release -- run --debug --port 3000 &
sleep 3
time kill -INT $!
```

### Monitor Logging
```bash
timeout 16 cargo run --release -- run --debug --port 3000 2>&1 | tee /tmp/test.log &
sleep 15
pkill -f cargo
grep -c "CCO_PROJECT_PATH" /tmp/test.log
```

### Test Terminal
```bash
cargo run --release -- run --debug --port 3000 &
sleep 3
curl http://127.0.0.1:3000/health
pkill -f cargo
```

---

## Support Resources

### Documentation Map
- **Overview**: TEST_DELIVERABLES_SUMMARY.md
- **How-to**: TESTING_INSTRUCTIONS.md
- **Reference**: QUICK_TEST_GUIDE.md
- **Detailed**: COMPREHENSIVE_TEST_PLAN.md
- **Checklist**: TEST_VERIFICATION_CHECKLIST.md

### Getting Help
1. Check TESTING_INSTRUCTIONS.md (Troubleshooting section)
2. Review COMPREHENSIVE_TEST_PLAN.md (Failure Investigation)
3. Run individual test from QUICK_TEST_GUIDE.md
4. Document issue in TEST_VERIFICATION_CHECKLIST.md

### Debugging Failed Tests
1. Find relevant test log in /tmp/test_*.log
2. Review failure message and error details
3. Check COMPREHENSIVE_TEST_PLAN.md for that test
4. Follow debugging steps in "Failure Investigation Guide"

---

## Workflow Summary

```
1. Read TESTING_INSTRUCTIONS.md (5 min)
   ↓
2. Run /tmp/comprehensive_test_suite.sh (8-10 min)
   ↓
3. Review results:
   ├─ ALL PASS → Record baseline, done ✓
   └─ FAIL → Check COMPREHENSIVE_TEST_PLAN.md debugging guide
   ↓
4. Document in TEST_VERIFICATION_CHECKLIST.md (5 min)
   ↓
5. Archive logs and complete
```

**Total time**: 30-45 minutes

---

## Next Steps After Testing

### If Tests PASS (All Green)
1. Record performance metrics
2. Archive test logs
3. Create release notes documenting fixes
4. Prepare for production deployment

### If Tests FAIL (Red Alert)
1. Review specific failure details
2. Check component-specific logs
3. Use debugging guide to identify issue
4. Wait for fixes, then re-test

### If Tests Show WARNINGS (Yellow)
1. Evaluate if warning is acceptable
2. Document the warning
3. Proceed if acceptable, or investigate if not

---

## Performance Metrics Template

After testing, record these values:

```
PERFORMANCE BASELINE
====================

Build: [commit hash]
Date: [test date]

Shutdown Performance:
  Target: < 2000ms
  Actual: ___ ms
  Status: PASS / FAIL

Logging Quality:
  Target: ≤ 6 messages in 15 seconds
  Actual: ___ messages
  Status: PASS / FAIL

Terminal Functionality:
  Health endpoint: Response [X]ms
  Status: PASS / FAIL

Overall Status: ✓ PASS / ✗ FAIL
```

---

## Document Versions

| Document | Version | Updated | Notes |
|----------|---------|---------|-------|
| TEST_SUITE_INDEX.md | 1.0 | 2025-11-16 | This file |
| TESTING_INSTRUCTIONS.md | 1.0 | 2025-11-16 | Ready |
| COMPREHENSIVE_TEST_PLAN.md | 1.0 | 2025-11-16 | Complete |
| QUICK_TEST_GUIDE.md | 1.0 | 2025-11-16 | Ready |
| TEST_VERIFICATION_CHECKLIST.md | 1.0 | 2025-11-16 | Ready |
| TEST_DELIVERABLES_SUMMARY.md | 1.0 | 2025-11-16 | Ready |
| comprehensive_test_suite.sh | 1.0 | 2025-11-16 | Executable |

---

## Ready Status

✓ All test documentation prepared
✓ Automated test script ready
✓ Manual test guides available
✓ Troubleshooting guides included
✓ All supporting materials complete

**Status**: READY FOR TEST EXECUTION

**Start here**: `/tmp/comprehensive_test_suite.sh`

---

**Last Updated**: 2025-11-16
**Version**: 1.0
**Status**: Complete and ready for use
