## GitHub-Secrets

---

It's a tool to push .env file entries to the github secrets section of a repository.

Usage

```bash
Push a .env file to GitHub Actions secrets

Usage: gh-secrets [OPTIONS] --repo <REPO>

Options:
      --repo <REPO>                GitHub repository in "owner/repo" format
      --env-file <ENV_FILE>        Path to .env file (default: .env) [default: .env]
      --prefix <PREFIX>            Prefix for each secret name (optional)
      --environment <ENVIRONMENT>  GitHub environment name for environment-scoped secrets (optional)
      --token <TOKEN>              GitHub API token (falls back to GITHUB_TOKEN or GH_TOKEN env var)
  -h, --help                       Print help
  -V, --version                    Print version

```
