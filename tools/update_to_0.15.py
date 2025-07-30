#!/usr/bin/env python3

import sys
from pathlib import Path

import convert_arrow
import convert_discrete
import convert_notes

def convert_file_content(content: str) -> str:
    """Apply all conversions in sequence: arrow, discrete, then notes."""
    # Step 1: Convert arrows
    content = convert_arrow.convert_file_content(content)
    
    # Step 2: Convert discrete patterns
    content = convert_discrete.convert_file_content(content)
    
    # Step 3: Convert notes
    content = convert_notes.convert_file_content(content)
    
    return content

def process_file(file_path: Path) -> None:
    """Process a single file, backing up the original and writing the converted content."""
    try:
        # Read original content
        content = file_path.read_text()
        
        # Create backup
        backup_path = Path(str(file_path) + '.bak')
        backup_path.write_text(content)
        
        # Convert and write new content
        new_content = convert_file_content(content)
        file_path.write_text(new_content)
        
        print(f"Successfully processed {file_path}")
    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)

def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: update_to_0.15.py FILE1 [FILE2 ...]", file=sys.stderr)
        print("This tool applies all conversions needed for Oneil 0.15:", file=sys.stderr)
        print("  1. Converts '=>' to '='", file=sys.stderr)
        print("  2. Converts discrete patterns [a, b, c] to ['a', 'b', 'c']", file=sys.stderr)
        print("  3. Converts whitespace-style notes to tilde-style notes", file=sys.stderr)
        sys.exit(1)
    
    for file_path in map(Path, sys.argv[1:]):
        if not file_path.exists():
            print(f"File not found: {file_path}", file=sys.stderr)
            continue
        process_file(file_path)

if __name__ == "__main__":
    main() 