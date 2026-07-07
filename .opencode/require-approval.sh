#!/bin/sh
# Git hook: rejects commits/pushes unless OPENCODE_COMMIT_APPROVED=1 is set.
# Enable per-repo:  git config opencode.requireApproval true
# To bypass:        OPENCODE_COMMIT_APPROVED=1 git commit ...
set -e

if [ "$(git config --bool opencode.requireApproval 2>/dev/null)" = "true" ]; then
  if [ "${OPENCODE_COMMIT_APPROVED:-0}" != "1" ]; then
    echo ""
    echo "============================================="
    echo "  ⛔ Commit blocked: no approval marker"
    echo "============================================="
    echo ""
    echo "  The AI must ask for explicit permission"
    echo "  before running git commit or git push."
    echo ""
    echo "  Set OPENCODE_COMMIT_APPROVED=1 to proceed."
    echo ""
    echo "============================================="
    echo ""
    exit 1
  fi
fi
