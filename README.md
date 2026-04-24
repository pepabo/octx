# octx

GitHub Data Extractor [![Test](https://github.com/udzura/octx/actions/workflows/ci.yml/badge.svg)](https://github.com/udzura/octx/actions/workflows/ci.yml)

## usage

```bash
$ export GITHUB_API_URL=https://... # Optional
$ export GITHUB_API_TOKEN=...       # Personal access token
$ octx --issues rust-lang rust --days-ago 30
# CSV will be put out to stdout
```

### GitHub App installation token

Instead of `GITHUB_API_TOKEN`, you can authenticate as a GitHub App
installation. octx will obtain an installation token from the given
App credentials and automatically refresh it before it expires.

```bash
$ export GITHUB_API_URL=https://...
$ export GITHUB_APP_ID=12345
$ export GITHUB_APP_PRIVATE_KEY="$(cat /path/to/private-key.pem)"
$ export GITHUB_APP_INSTALLATION_ID=67890
$ octx --issues rust-lang rust --days-ago 30
```

`GITHUB_API_TOKEN` and the three `GITHUB_APP_*` variables are mutually
exclusive; set either one PAT or the full App triple.

## note

* It is reccomended to specify `--days-ago` or `-since-date` for limiting issue/comment/event extracton when you run this tool against github.com.
* This tool is mainly intended to cooperate with GHES.
