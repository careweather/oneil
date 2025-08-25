#!/usr/bin/env python3

import sys
import re
from pathlib import Path
from typing import List, Tuple

def find_discrete_patterns(content: str) -> List[Tuple[int, str, int, int]]:
    """
    Find all discrete patterns in the content.
    
    Returns a list of tuples: (line_number, line_content, start_pos, end_pos)
    where start_pos and end_pos are the positions within the line.
    """
    lines = content.splitlines()
    patterns = []
    
    for line_num, line in enumerate(lines, 1):
        # Find all matches of the pattern [\w+(, \w+)*]
        matches = list(re.finditer(r'\[(\w+(?:, \w+)*)\]', line))
        
        for match in matches:
            patterns.append((line_num, line, match.start(), match.end()))
    
    return patterns

def convert_discrete_pattern(match_text: str) -> str:
    """
    Convert a discrete pattern by replacing `\\w+` with `'\\w+'`.
    
    Example: [a, b, c] -> ['a', 'b', 'c']
    """
    # Remove the outer brackets
    inner_content = match_text[1:-1]
    
    # Split by comma and space, then quote each part
    parts = [part.strip() for part in inner_content.split(',')]
    quoted_parts = [f"'{part}'" for part in parts]
    
    # Rejoin with comma and space, and add brackets back
    return f"[{', '.join(quoted_parts)}]"

def find_trailing_word(line: str, pattern_end: int) -> Tuple[int, int]:
    """
    Find the trailing word at the end of the line.
    
    Returns (start_pos, end_pos) of the trailing word, or (-1, -1) if not found.
    """
    # Look for a word at the end of the line
    remaining_line = line[pattern_end:].strip()
    if remaining_line:
        # Find the last word in the remaining line
        words = remaining_line.split()
        if words:
            last_word = words[-1]
            # Find the position of this last word in the original line
            last_word_start = line.rfind(last_word)
            if last_word_start != -1:
                return (last_word_start, last_word_start + len(last_word))
    
    return (-1, -1)

def convert_file_content(content: str) -> str:
    """Convert discrete patterns in the given content."""
    lines = content.splitlines()
    converted_lines = lines.copy()
    
    # Find all discrete patterns
    patterns = find_discrete_patterns(content)
    
    # Process patterns in reverse order to maintain line indices
    for line_num, line, start_pos, end_pos in reversed(patterns):
        # Convert the discrete pattern
        original_pattern = line[start_pos:end_pos]
        converted_pattern = convert_discrete_pattern(original_pattern)
        
        # Find trailing word
        trailing_start, trailing_end = find_trailing_word(line, end_pos)
        
        # Build the new line
        new_line = line[:start_pos] + converted_pattern
        
        if trailing_start != -1:
            # Quote the trailing word
            trailing_word = line[trailing_start:trailing_end]
            new_line += line[end_pos:trailing_start] + f"'{trailing_word}'" + line[trailing_end:]
        else:
            new_line += line[end_pos:]
        
        converted_lines[line_num - 1] = new_line
    
    return '\n'.join(converted_lines) + '\n'

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
        print("Usage: convert_discrete.py FILE1 [FILE2 ...]", file=sys.stderr)
        sys.exit(1)
    
    for file_path in map(Path, sys.argv[1:]):
        if not file_path.exists():
            print(f"File not found: {file_path}", file=sys.stderr)
            continue
        process_file(file_path)

if __name__ == "__main__":
    main() 