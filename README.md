# octx

GitHub Data Extractor [![Test](https://github.com/udzura/octx/actions/workflows/ci.yml/badge.svg)](https://github.com/udzura/octx/actions/workflows/ci.yml)

## usage

```bash
$ export GITHUB_API_TOKEN=...
$ export GITHUB_API_URL=https://... # Optional
$ octx --issues rust-lang rust --days-ago 30
# CSV will be put out to stdout
```

## note

* It is reccomended to specify `--days-ago` or `-since-date` for limiting issue/comment/event extracton when you run this tool against github.com.
* This tool is mainly intended to cooperate with GHES.
