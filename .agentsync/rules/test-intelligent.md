---
targets:
- '*'
description: This rule helps with writing test files and test utilities
globs: '**/*'
cursor:
  alwaysApply: false
  globs: ''
windsurf:
  trigger: model_decision
  globs: ''
copilot:
  applyTo: '**'
---

# Test Rule - Apply Intelligently

This rule is applied intelligently based on context and description.

It should appear in:
- Cursor: Auto Attached mode with description (alwaysApply: false, description present, no globs)
- Windsurf: model_decision mode (description present, no globs)
- Copilot: Applied based on description

## Guidelines

When writing tests:
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)
- Keep tests isolated and independent
- Mock external dependencies

