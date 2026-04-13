#!/usr/bin/env python3
"""Update README.md with current werk tension tree."""

import os
import re
import subprocess


def get_tension_tree():
    """Get werk tension tree if available."""
    for cmd in [["werk", "tree"], ["target/debug/werk", "tree"]]:
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
            if result.returncode == 0 and result.stdout.strip():
                return result.stdout.strip()
        except (FileNotFoundError, subprocess.TimeoutExpired):
            continue
    return None


def update_readme():
    readme_path = "README.md"
    if not os.path.isfile(readme_path):
        return

    readme = open(readme_path).read()
    changed = False

    # Update tension tree
    tension_tree = get_tension_tree()
    if tension_tree:
        pattern = r"(## Current tension tree\s*\n\s*```\n).*?(```)"
        new_section = rf"\g<1>{tension_tree}\n\g<2>"
        new_readme = re.sub(pattern, new_section, readme, flags=re.DOTALL)
        if new_readme != readme:
            readme = new_readme
            changed = True

    if changed:
        open(readme_path, "w").write(readme)


if __name__ == "__main__":
    update_readme()
