# Examples

`library/` is a sample PromptShelf library with made-up prompts. Try it:

```sh
shelf libs add examples/library --name examples
shelf list --lib examples
shelf fill examples/strict-pr-review --var focus="error handling" --var diff="(paste a diff)"
```
