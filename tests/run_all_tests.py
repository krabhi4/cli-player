#!/usr/bin/env python3
"""
Comprehensive Test Runner for CLI Music Player

Runs all test suites and provides detailed reporting.
"""

import os
import sys
import time
import unittest

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

# Import all test modules
from test_bug_fixes import *
from test_config import *
from test_equalizer import *
from test_queue import *
from test_subsonic import *


class ColoredTextTestResult(unittest.TextTestResult):
    """Custom test result with colored output"""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.test_times = {}

    def startTest(self, test):
        super().startTest(test)
        self.test_times[test] = time.time()

    def addSuccess(self, test):
        super().addSuccess(test)
        elapsed = time.time() - self.test_times[test]
        if self.showAll:
            self.stream.write(f" ✓ ({elapsed:.3f}s)\n")

    def addError(self, test, err):
        super().addError(test, err)
        if self.showAll:
            self.stream.write(" ✗ ERROR\n")

    def addFailure(self, test, err):
        super().addFailure(test, err)
        if self.showAll:
            self.stream.write(" ✗ FAILED\n")

    def addSkip(self, test, reason):
        super().addSkip(test, reason)
        if self.showAll:
            self.stream.write(f" ⊘ SKIPPED: {reason}\n")


class ColoredTextTestRunner(unittest.TextTestRunner):
    """Custom test runner with colored output"""

    resultclass = ColoredTextTestResult


def get_test_suites():
    """Get all test suites organized by category"""
    loader = unittest.TestLoader()

    suites = {
        "Bug Fixes (v2.0.1)": unittest.TestSuite(
            [
                loader.loadTestsFromName("test_bug_fixes.TestBugFix1_DoubleScrobbling"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix2_QueueJumpTo"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix3_SessionCleanup"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix4_PasswordDecryption"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix5_QueueRemove"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix6_ThreadSafety"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix7_SearchResultsBounds"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix8_NegativeIndices"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix9_NavigationHistory"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix10_EqualizerConversion"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix11_RequestTimeout"),
                loader.loadTestsFromName("test_bug_fixes.TestBugFix12_EqualizerClickBounds"),
            ]
        ),
        "Subsonic API Client": unittest.TestSuite(
            [
                loader.loadTestsFromName("test_subsonic.TestSubsonicDataModels"),
                loader.loadTestsFromName("test_subsonic.TestSubsonicClient"),
                loader.loadTestsFromName("test_subsonic.TestSubsonicAPIMethods"),
            ]
        ),
        "Queue Manager": unittest.TestSuite(
            [
                loader.loadTestsFromName("test_queue.TestRepeatMode"),
                loader.loadTestsFromName("test_queue.TestQueueManager"),
            ]
        ),
        "Configuration": unittest.TestSuite(
            [
                loader.loadTestsFromName("test_config.TestPasswordEncryption"),
                loader.loadTestsFromName("test_config.TestServerConfig"),
                loader.loadTestsFromName("test_config.TestEQPreset"),
                loader.loadTestsFromName("test_config.TestAppConfig"),
            ]
        ),
        "Equalizer": unittest.TestSuite(
            [
                loader.loadTestsFromName("test_equalizer.TestEqualizerConstants"),
                loader.loadTestsFromName("test_equalizer.TestEqualizer"),
                loader.loadTestsFromName("test_equalizer.TestEqualizerPresets"),
            ]
        ),
    }

    return suites


def print_header():
    """Print test header"""
    print("=" * 80)
    print("CLI Music Player - Comprehensive Test Suite")
    print("Version 2.0.1")
    print("=" * 80)
    print()


def print_category_header(category_name):
    """Print category header"""
    print()
    print("─" * 80)
    print(f"  {category_name}")
    print("─" * 80)


def print_summary(results):
    """Print test summary"""
    total_tests = sum(r.testsRun for r in results)
    total_failures = sum(len(r.failures) for r in results)
    total_errors = sum(len(r.errors) for r in results)
    total_skipped = sum(len(r.skipped) for r in results)
    total_success = total_tests - total_failures - total_errors - total_skipped

    print()
    print("=" * 80)
    print("TEST SUMMARY")
    print("=" * 80)
    print(f"Total Tests Run:     {total_tests}")
    print(f"✓ Passed:            {total_success}")
    print(f"✗ Failed:            {total_failures}")
    print(f"✗ Errors:            {total_errors}")
    print(f"⊘ Skipped:           {total_skipped}")
    print()

    if total_failures == 0 and total_errors == 0:
        print("🎉 ALL TESTS PASSED! 🎉")
        print()
        print("Coverage by Category:")
        return True
    print("❌ SOME TESTS FAILED")
    print()
    print("Failed Tests:")
    for result in results:
        for test, traceback in result.failures + result.errors:
            print(f"  - {test}")
    print()
    return False


def print_coverage_breakdown(results_dict):
    """Print test coverage breakdown by category"""
    for category, result in results_dict.items():
        total = result.testsRun
        failed = len(result.failures) + len(result.errors)
        passed = total - failed
        percentage = (passed / total * 100) if total > 0 else 0
        status = "✓" if failed == 0 else "✗"
        print(f"  {status} {category:30s} {passed}/{total} ({percentage:.0f}%)")


def run_all_tests(verbosity=2):
    """Run all tests with detailed reporting"""
    start_time = time.time()

    print_header()

    suites = get_test_suites()
    results = []
    results_dict = {}

    for category_name, suite in suites.items():
        print_category_header(category_name)

        runner = ColoredTextTestRunner(verbosity=verbosity)
        result = runner.run(suite)
        results.append(result)
        results_dict[category_name] = result

    elapsed_time = time.time() - start_time

    success = print_summary(results)
    print_coverage_breakdown(results_dict)

    print()
    print(f"Total execution time: {elapsed_time:.2f}s")
    print("=" * 80)
    print()

    return 0 if success else 1


def run_specific_category(category_name, verbosity=2):
    """Run tests for a specific category"""
    suites = get_test_suites()

    if category_name not in suites:
        print(f"Error: Category '{category_name}' not found")
        print(f"Available categories: {', '.join(suites.keys())}")
        return 1

    print_header()
    print_category_header(category_name)

    runner = ColoredTextTestRunner(verbosity=verbosity)
    result = runner.run(suites[category_name])

    success = len(result.failures) == 0 and len(result.errors) == 0

    print()
    print("=" * 80)
    if success:
        print("✓ All tests in category passed!")
    else:
        print("✗ Some tests failed")
    print("=" * 80)

    return 0 if success else 1


def main():
    """Main test runner"""
    import argparse

    parser = argparse.ArgumentParser(description="Run CLI Music Player tests")
    parser.add_argument(
        "--category",
        help="Run tests for specific category",
        choices=[
            "Bug Fixes (v2.0.1)",
            "Subsonic API Client",
            "Queue Manager",
            "Configuration",
            "Equalizer",
        ],
    )
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")
    parser.add_argument("-q", "--quiet", action="store_true", help="Minimal output")

    args = parser.parse_args()

    verbosity = 2
    if args.verbose:
        verbosity = 2
    elif args.quiet:
        verbosity = 1

    if args.category:
        return run_specific_category(args.category, verbosity)
    return run_all_tests(verbosity)


if __name__ == "__main__":
    sys.exit(main())
