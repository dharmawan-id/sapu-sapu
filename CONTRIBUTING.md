# Contributing

Sapu Sapu gets better when its cache list is wider and its safety is tighter. Four kinds of contribution are welcome.

1. **Bug reports.** A scan that misses, a clean that fails, a number that looks wrong. Open an issue with your Windows version and what you saw.
2. **New safe cache targets.** A regenerating cache Sapu Sapu does not know yet (another package manager, an editor, a tool). Propose it with the exact path and why it is safe to clear. New targets must regenerate on their own.
3. **Tighter guards.** A path that should be protected but is not, or a project folder that should be flagged unsafe but is not. These are the most valuable reports.
4. **Translations.** The interface and the README live in English and Indonesian. Other languages are welcome.

## How to propose a change

- Small fixes: open a pull request directly.
- Larger changes (a new scan mode, a new tier, a new safety check): open an issue first so the design can be discussed before the work.
- Keep the protected-path guard in `src-tauri/src/cleaner.rs` as the single source of truth. A new cache target must never overlap a protected path.

## Writing standard for prose

If you contribute prose (README, interface text, docs), please follow the same standard the rest of the project uses:

- No em dashes. Use commas, colons, parentheses, or a new sentence.
- Avoid contrast-framing as a device ("not X but Y"). State the point directly.
- Plain, precise words. No filler, no inflated vocabulary.
- Do not invent numbers. The reclaim figures come from the real free-space delta.

## License of contributions

By contributing, you agree that your contribution is licensed under the MIT License, the same as the rest of the project.
