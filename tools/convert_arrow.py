#!/usr/bin/env python3

import sys
from pathlib import Path

def convert_file_content(content: str) -> str:
    """Replace all instances of '=>' with '=' in the given content."""
    return content.replace('=>', '=')

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
        print("Usage: convert_arrow.py FILE1 [FILE2 ...]", file=sys.stderr)
        sys.exit(1)
    
    for file_path in map(Path, sys.argv[1:]):
        if not file_path.exists():
            print(f"File not found: {file_path}", file=sys.stderr)
            continue
        process_file(file_path)

if __name__ == "__main__":
    main() 