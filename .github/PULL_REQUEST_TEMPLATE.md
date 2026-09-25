## Description
Provide a clear description of the changes proposed in this Pull Request. Include relevant motivation, context, and link to any related issues.

Fixes #(issue)

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Verification Checklist
- [ ] `cd watchmark-tauri/src-tauri && cargo check` passes with 0 errors.
- [ ] `cd watchmark-tauri && npx tsc --noEmit` passes with 0 errors.
- [ ] `cd watchmark-tauri && npm run build` completes successfully.
- [ ] Manual testing has been performed on desktop.
- [ ] `CHANGELOG.md` has been updated if applicable.
