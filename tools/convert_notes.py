#!/usr/bin/env python3

import sys
import re
from typing import List, Tuple
from pathlib import Path
from dataclasses import dataclass
from itertools import groupby

@dataclass
class Note:
    content: str
    is_multiline: bool

def is_whitespace_style_note_line(line: str) -> bool:
    """Check if a line is a whitespace-style note line (starts with spaces/tabs but not with '{')."""
    return bool(re.match(r'^[ \t]+[^\{ \t]', line))

def strip_leading_whitespace(text: str) -> str:
    """Remove leading whitespace from each line while preserving relative indentation."""
    lines = text.splitlines()
    if not lines:
        return ""
    
    # Find minimum indentation across non-empty lines
    indents = [len(line) - len(line.lstrip()) 
               for line in lines if line.strip()]
    min_indent = min(indents) if indents else 0
    
    # Remove that amount of whitespace from each line
    stripped_lines = [line[min_indent:] if line.strip() else line 
                     for line in lines]
    return '\n'.join(stripped_lines)

def group_note_lines(lines: List[str]) -> List[Tuple[bool, List[str]]]:
    """Group lines into note and non-note sections."""
    return [(key, list(group)) 
            for key, group in groupby(lines, is_whitespace_style_note_line)]

def is_blank_line(line: str) -> bool:
    return line.strip() == ''

def merge_groups(groups: List[Tuple[bool, List[str]]]) -> List[Tuple[bool, List[str]]]:
    """Merge note groups that are separated only by blank lines."""
    def recurse(groups: List[Tuple[bool, List[str]]], result: List[Tuple[bool, List[str]]]) -> List[Tuple[bool, List[str]]]:
        if len(groups) < 2:
            return result + groups

        prev_group = result[-1]
        curr_group = groups[0]
        next_group = groups[1]
        
        if prev_group[0] and next_group[0]:
            if all(is_blank_line(line) for line in curr_group[1]):
                return recurse(groups[2:], result[:-1] + [(True, prev_group[1] + curr_group[1] + next_group[1])])

        return recurse(groups[1:], result + [curr_group])

    if not groups:
        return []
        
    return recurse(groups[1:], [groups[0]])

def process_note_group(lines: List[str]) -> Note:
    """Process a group of note lines into a Note object."""
    content = strip_leading_whitespace('\n'.join(lines))
    is_multiline = len(lines) > 1 or '\n' in content
    return Note(content=content, is_multiline=is_multiline)

def format_note(note: Note) -> str:
    """Format a note in the tilde style."""
    if note.is_multiline:
        return f"~~~\n{note.content}\n~~~"
    return f"~ {note.content}"

def convert_file_content(content: str) -> str:
    """Convert whitespace-style notes to tilde-style in the given content."""
    lines = content.splitlines()
    result = []

    groups = group_note_lines(lines)
    groups = merge_groups(groups)

    for is_note_group, group in groups:
        print(is_note_group, group)
        if is_note_group:
            note = process_note_group(group)
            result.append(format_note(note))
        else:
            result.extend(group)
    
    return '\n'.join(result)

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
        print("Usage: convert_notes.py FILE1 [FILE2 ...]", file=sys.stderr)
        sys.exit(1)
    
    for file_path in map(Path, sys.argv[1:]):
        if not file_path.exists():
            print(f"File not found: {file_path}", file=sys.stderr)
            continue
        process_file(file_path)

if __name__ == "__main__":
    main() 