#!/usr/bin/env python3
"""Test script to verify function caching works correctly."""

import os
import sys
import time
import importlib
import shutil

# Add the src directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

# Change to test directory so model files can be found
os.chdir(os.path.dirname(__file__))

import oneil

def get_call_count():
    """Get call count from the persistent file (survives module reloads)."""
    try:
        with open("/tmp/oneil_test_call_count.txt", 'r') as f:
            return int(f.read().strip())
    except (FileNotFoundError, ValueError):
        return 0

def reset_call_count():
    """Reset call count."""
    try:
        os.remove("/tmp/oneil_test_call_count.txt")
    except FileNotFoundError:
        pass

def clear_cache_dir():
    """Remove the __oncache__ directory for clean tests."""
    cache_dir = os.path.join(os.path.dirname(__file__), "__oncache__")
    if os.path.exists(cache_dir):
        shutil.rmtree(cache_dir)

def test_caching():
    """Test that Python function results are cached and invalidated correctly."""
    
    # Clean up before testing
    clear_cache_dir()
    reset_call_count()
    oneil._function_cache.clear_all()
    
    print("=" * 60)
    print("Test 1: Initial load should call the function")
    print("=" * 60)
    
    model = oneil.Model("cache_test.on")
    model.build(quiet=True)
    
    call_count_1 = get_call_count()
    print(f"  Call count after initial load: {call_count_1}")
    assert call_count_1 == 1, f"Expected 1 call, got {call_count_1}"
    
    print("  ✓ Function was called once on initial load")
    
    print("\n" + "=" * 60)
    print("Test 2: Rebuild with unchanged inputs should use cache")
    print("=" * 60)
    
    # Reset the model to trigger recalculation
    model._reset_recursively()
    model.build(quiet=True)
    
    call_count_2 = get_call_count()
    print(f"  Call count after rebuild: {call_count_2}")
    assert call_count_2 == 1, f"Expected 1 call (cached), got {call_count_2}"
    
    print("  ✓ Function was NOT called again (used cache)")
    
    print("\n" + "=" * 60)
    print("Test 3: Full reload should use cache (file unchanged)")
    print("=" * 60)
    
    # Simulate a full reload like the REPL would do
    model2 = oneil.Model("cache_test.on")
    model2.build(quiet=True)
    
    call_count_3 = get_call_count()
    print(f"  Call count after full reload: {call_count_3}")
    assert call_count_3 == 1, f"Expected 1 call (cached), got {call_count_3}"
    
    print("  ✓ Function was NOT called again (used cache)")
    
    print("\n" + "=" * 60)
    print("Test 4: NEW REPL session should use DISK cache")
    print("=" * 60)
    
    # Clear in-memory state to simulate new REPL session
    oneil._function_cache._memory_cache.clear()
    oneil._function_cache._import_hashes.clear()
    oneil._function_cache._function_modules.clear()
    
    # Load model again - should use disk cache
    model3 = oneil.Model("cache_test.on")
    model3.build(quiet=True)
    
    call_count_4 = get_call_count()
    print(f"  Call count after 'new session': {call_count_4}")
    assert call_count_4 == 1, f"Expected 1 call (disk cache hit), got {call_count_4}"
    
    print("  ✓ Function was NOT called again (used DISK cache)")
    
    print("\n" + "=" * 60)
    print("Test 5: Modifying Python file should invalidate cache")
    print("=" * 60)
    
    # Modify the Python file to trigger cache invalidation
    func_file = "cache_test_funcs.py"
    with open(func_file, 'r') as f:
        original_content = f.read()
    
    # Add a comment to change the file hash
    modified_content = original_content + "\n# Modified\n"
    with open(func_file, 'w') as f:
        f.write(modified_content)
    
    try:
        # Give the filesystem time to update
        time.sleep(0.1)
        
        # Clear in-memory state again
        oneil._function_cache._memory_cache.clear()
        oneil._function_cache._import_hashes.clear()
        
        model4 = oneil.Model("cache_test.on")
        model4.build(quiet=True)
        
        call_count_5 = get_call_count()
        print(f"  Call count after file modification: {call_count_5}")
        assert call_count_5 == 2, f"Expected 2 calls (cache invalidated), got {call_count_5}"
        
        print("  ✓ Function WAS called again (cache invalidated)")
    finally:
        # Restore the original file
        with open(func_file, 'w') as f:
            f.write(original_content)
    
    print("\n" + "=" * 60)
    print("Test 6: Changing input values should re-run the function")
    print("=" * 60)
    
    # Reset for this test
    reset_call_count()
    clear_cache_dir()
    oneil._function_cache.clear_all()
    
    # Create model with original input
    model5 = oneil.Model("cache_test.on")
    model5.build(quiet=True)
    
    call_count_6a = get_call_count()
    print(f"  Call count with x=5: {call_count_6a}")
    assert call_count_6a == 1, f"Expected 1 call, got {call_count_6a}"
    
    # Now create a design file that changes the input value
    with open("cache_test_design.on", 'w') as f:
        f.write("x = 10\n")
    
    try:
        model6 = oneil.Model("cache_test.on")
        model6.overwrite(["cache_test_design.on"], quiet=True)
        
        call_count_6b = get_call_count()
        print(f"  Call count with x=10: {call_count_6b}")
        assert call_count_6b == 2, f"Expected 2 calls (different input), got {call_count_6b}"
        
        print("  ✓ Function WAS called again (different input values)")
    finally:
        os.remove("cache_test_design.on")
    
    print("\n" + "=" * 60)
    print("Test 7: Cache stats and index file")
    print("=" * 60)
    
    stats = oneil._function_cache.stats()
    print(f"  Cached results (disk): {stats['disk_entries']}")
    print(f"  Cached results (memory): {stats['memory_entries']}")
    print(f"  Tracked imports: {stats['tracked_imports']}")
    print(f"  Cache directory: {stats['cache_dir']}")
    print(f"  Cache size: {stats['cache_size_bytes']} bytes")
    
    assert stats['disk_entries'] >= 1, "Expected at least 1 disk cache entry"
    assert stats['tracked_imports'] >= 1, "Expected at least 1 tracked import"
    
    # Verify index.json exists and is readable
    index_path = os.path.join(stats['cache_dir'], "index.json")
    assert os.path.exists(index_path), "index.json should exist"
    
    import json
    with open(index_path, 'r') as f:
        index = json.load(f)
    
    print(f"  Index version: {index.get('version')}")
    print(f"  Index entries: {len(index.get('entries', {}))}")
    
    # Show a sample entry
    if index.get('entries'):
        sample_key = list(index['entries'].keys())[0]
        sample = index['entries'][sample_key]
        print(f"  Sample entry:")
        print(f"    Function: {sample.get('function')}")
        print(f"    Inputs: {sample.get('inputs_repr')}")
        print(f"    Cached at: {sample.get('cached_at')}")
    
    print("  ✓ Cache stats and index are valid")
    
    print("\n" + "=" * 60)
    print("All tests passed! ✓")
    print("=" * 60)
    
    # Cleanup
    reset_call_count()

if __name__ == "__main__":
    test_caching()
