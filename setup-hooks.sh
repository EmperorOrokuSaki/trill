# setup-hooks.sh
#!/bin/sh
# This script installs the Git hooks from the .githooks directory.

# Copy the pre-commit hook
cp .githooks/pre-commit .git/hooks/pre-commit

# Make sure it is executable
chmod +x .git/hooks/pre-commit

echo "Git hooks installed."

