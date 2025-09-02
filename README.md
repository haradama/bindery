# Bindery

Bindery is a CLI tool that **concatenates project source files** into a single output file, making it easier to submit code for **reviews, audits, or analysis in services such as ChatGPT**.

## Features

* **Ignore hidden files/directories by default** (e.g., `.git`, `.vscode`, `.DS_Store`)
* Use `-a, --all` to **include hidden files** in the scan
* **Keep comments** by default
* Use `-n, --no-comments` to **strip comments** before concatenation
* **Default output:** standard output (stdout)
* Use `-o, --output <FILE>` to write output to a file (the output file is excluded from scanning)
* Stable and human-readable output format:

~~~md
src/services/userService.ts

```ts
export async function createUser(...) { ... }
// ... omitted ...
````

src/routes/user.ts

```ts
router.post('/users', createUserHandler)
// ... omitted ...
```

~~~

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/haradama/bindery/main/install.sh | sh -
```

This will:

* Download the latest release of **bindery**
* Install it to `~/.local/bin`
* Make sure `~/.local/bin` is in your PATH

The compiled binary will be available at `target/release/bindery`.

## Usage

```bash
# Concatenate all project files (excluding hidden files, keeping comments)
bindery . > bundle.md

# Include hidden files
bindery -a . > bundle.md

# Remove comments before concatenation
bindery -n . > bundle.md

# Write to file (excluded from scan)
bindery -n -o out/bindery.md .
```

## Options

```bash
-a, --all           Include hidden files and directories
-n, --no-comments   Strip comments from source files
-o, --output FILE   Write output to file (instead of stdout)
```

## License

This project is licensed under the MIT License.
