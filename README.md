# octx

GitHub Data Extractor [![Test](https://github.com/udzura/octx/actions/workflows/ci.yml/badge.svg)](https://github.com/udzura/octx/actions/workflows/ci.yml)

## usage

```bash
$ export GITHUB_API_TOKEN=...
$ export GITHUB_API_URL=https://... # Optional
$ octx --issues --owner <owner> --name <name>
# CSV will be put out to stdout
```
