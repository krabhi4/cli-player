# CLI Music Player - Test Suite

Comprehensive automated tests for all components of the CLI Music Player.

## Overview

This test suite provides thorough coverage of:
- ✅ Bug fixes (v2.0.1 - 12 critical/high-priority bugs)
- ✅ Subsonic API client (data models, requests, error handling)
- ✅ Queue manager (playback queue, shuffle, repeat)
- ✅ Configuration (settings, encryption, persistence)
- ✅ Equalizer (18-band EQ, presets, dB conversion)

**Total Tests:** 120+ test cases
**Coverage:** Core functionality, edge cases, error handling

## Quick Start

### Run All Tests

```bash
cd /home/abhi/docker/cli-music-player
python3 tests/run_all_tests.py
```

### Run Specific Test File

```bash
# Bug fix tests only
python3 tests/test_bug_fixes.py

# Subsonic API tests
python3 tests/test_subsonic.py

# Queue manager tests
python3 tests/test_queue.py

# Configuration tests
python3 tests/test_config.py

# Equalizer tests
python3 tests/test_equalizer.py
```

### Run Tests by Category

```bash
# Run only bug fix tests
python3 tests/run_all_tests.py --category "Bug Fixes (v2.0.1)"

# Run only queue tests
python3 tests/run_all_tests.py --category "Queue Manager"

# Run only equalizer tests
python3 tests/run_all_tests.py --category "Equalizer"
```

### Verbose/Quiet Output

```bash
# Verbose output (more details)
python3 tests/run_all_tests.py -v

# Quiet output (less details)
python3 tests/run_all_tests.py -q
```

## Test Categories

### 1. Bug Fixes (v2.0.1)
**File:** `test_bug_fixes.py`
**Tests:** 24 tests covering all 12 bug fixes

- ✅ Double scrobbling prevention
- ✅ Thread-safe queue navigation
- ✅ HTTP session cleanup
- ✅ Password decryption error handling
- ✅ Queue index bounds
- ✅ Player observer thread safety
- ✅ Search result race conditions
- ✅ Negative index validation
- ✅ Navigation history limits
- ✅ Equalizer dB conversion
- ✅ HTTP timeout enforcement
- ✅ EQ click edge cases

### 2. Subsonic API Client
**File:** `test_subsonic.py`
**Tests:** 20+ tests

**Data Models:**
- Song, Album, Artist, Playlist, Genre creation
- API data parsing
- Missing field handling

**Client Operations:**
- Authentication (token + salt)
- Request handling
- Error responses
- Timeout handling
- Connection errors
- Stream URL generation

**API Methods:**
- Ping
- Get albums
- Get artists
- Search
- Get playlists
- Get genres

### 3. Queue Manager
**File:** `test_queue.py`
**Tests:** 40+ tests

**Core Functionality:**
- Queue initialization
- Add/remove songs
- Queue clearing
- Song reordering

**Playback Control:**
- Next/previous navigation
- Jump to index
- Current song tracking

**Shuffle & Repeat:**
- Shuffle toggle
- Shuffle history
- Order restoration
- Repeat modes (Off/All/One)

**Edge Cases:**
- Empty queue handling
- Boundary conditions
- Index validation

### 4. Configuration
**File:** `test_config.py`
**Tests:** 30+ tests

**Password Security:**
- Encryption/decryption
- Invalid data handling
- Salt randomization

**Server Management:**
- Add/remove servers
- Server switching
- Active server tracking
- Password retrieval

**Settings Persistence:**
- Save/load config
- Volume settings
- Shuffle/repeat state
- Audio device

**EQ Presets:**
- Built-in presets
- Custom presets
- Preset management

### 5. Equalizer
**File:** `test_equalizer.py`
**Tests:** 30+ tests

**Core Functionality:**
- 18-band gain control
- Gain clamping (-12 to +12 dB)
- Enable/disable toggle

**dB to Linear Conversion:**
- Correct formula (10^(dB/20))
- Zero dB = 1.0 linear
- Positive/negative dB
- Clamping to 0-20 range

**Presets:**
- Flat (all zeros)
- Bass Boost
- Treble Boost
- Vocal
- Rock
- Custom presets

**Filter Generation:**
- MPV superequalizer format
- Band value formatting
- Empty filter optimization

## Test Structure

```
tests/
├── __init__.py                 # Package init
├── README.md                   # This file
├── run_all_tests.py           # Comprehensive test runner
├── test_bug_fixes.py          # Bug fix verification tests
├── test_subsonic.py           # Subsonic API tests
├── test_queue.py              # Queue manager tests
├── test_config.py             # Configuration tests
└── test_equalizer.py          # Equalizer tests
```

## Writing New Tests

### Test File Template

```python
#!/usr/bin/env python3
"""
Description of what this test file covers
"""

import unittest
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from cli_music_player.module import ClassName


class TestClassName(unittest.TestCase):
    """Test ClassName functionality"""

    def setUp(self):
        """Run before each test"""
        pass

    def tearDown(self):
        """Run after each test"""
        pass

    def test_feature_name(self):
        """Test specific feature"""
        # Arrange
        expected = "value"

        # Act
        result = do_something()

        # Assert
        self.assertEqual(result, expected)


if __name__ == "__main__":
    unittest.main()
```

### Adding Tests to Runner

Edit `run_all_tests.py` and add your test class to the appropriate category in `get_test_suites()`:

```python
suites = {
    "Your Category": unittest.TestSuite([
        loader.loadTestsFromName('your_test_file.YourTestClass'),
    ]),
}
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Set up Python
        uses: actions/setup-python@v2
        with:
          python-version: '3.11'
      - name: Install dependencies
        run: |
          pip install -e .
      - name: Run tests
        run: python3 tests/run_all_tests.py
```

## Coverage Report

To generate a coverage report:

```bash
# Install coverage tool
pip install coverage

# Run tests with coverage
coverage run -m pytest tests/

# Generate report
coverage report

# Generate HTML report
coverage html
```

## Test Best Practices

1. **Isolation** - Each test should be independent
2. **Arrange-Act-Assert** - Clear test structure
3. **Descriptive Names** - Test names should describe what they test
4. **Mock External Dependencies** - Don't hit real APIs/databases
5. **Test Edge Cases** - Not just happy paths
6. **Fast Execution** - Tests should run quickly
7. **Deterministic** - Same input → same output every time

## Debugging Failed Tests

### Run Single Test

```bash
python3 -m unittest tests.test_queue.TestQueueManager.test_next_without_repeat
```

### Verbose Output

```bash
python3 -m unittest tests.test_queue -v
```

### Print Debug Info

Add print statements in your test:

```python
def test_something(self):
    result = function_under_test()
    print(f"Debug: result = {result}")  # Will show in verbose mode
    self.assertEqual(result, expected)
```

## Known Limitations

- **No UI Tests** - Textual widget testing requires special setup
- **No Integration Tests** - Tests focus on unit testing individual components
- **No Network Tests** - Real Navidrome connections not tested
- **No Audio Tests** - MPV audio output not verified

## Future Improvements

- [ ] Integration tests with test Navidrome instance
- [ ] Textual app widget tests
- [ ] Performance/load testing
- [ ] Coverage reporting
- [ ] CI/CD integration
- [ ] Mutation testing
- [ ] Property-based testing

## Troubleshooting

### Import Errors

If you get import errors:

```bash
# Make sure you're in the project root
cd /home/abhi/docker/cli-music-player

# Install package in development mode
pip install -e .

# Run tests
python3 tests/run_all_tests.py
```

### Mock Errors

If mocking isn't working:

```python
from unittest.mock import Mock, patch, MagicMock

# Use patch as decorator
@patch('module.ClassName')
def test_something(self, mock_class):
    pass

# Or as context manager
def test_something(self):
    with patch('module.ClassName') as mock_class:
        pass
```

## Contributing

When adding new features:

1. Write tests FIRST (TDD approach)
2. Ensure all existing tests pass
3. Add new tests for your feature
4. Aim for >80% code coverage
5. Document your tests

## License

Tests are part of the CLI Music Player project and follow the same MIT license.
