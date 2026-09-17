# Legacy Python codecs and CLI

The primary application is now the Rust Ratatui terminal workbench. From the repository root, run:

```bash
./run-tui.sh
```

See the [project README](../README.md) for current installation, terminal shortcuts, and CLI examples.

This directory retains the original Python DNA encoding modules, safety screener, tests, and standalone [Python CLI](biocypher_cli/README.md). The Flask application, templates, static assets, and web dependencies have been removed.

To exercise the legacy codecs:

```bash
python -m pip install -r requirements.txt
python -m unittest discover -p 'test_*.py'
```

The Rust application does not require Python.
