# Documentation checks

Run `python scripts/validate_docs.py` from the local Git root. The checker reads tracked and nonignored files. Its Python environment needs `markdown-it-py`, `jsonschema[format]` and `PyYAML`. These are documentation tools. The native application and its fixture have no Python runtime requirement.

The check covers relative Markdown destinations and heading anchors, strict JSON syntax, Draft 2020-12 schemas and examples, traceability IDs, archive hashes, and editable diagram/render pairing. For official CFF 1.2.0 validation pass `--cff-schema <local-schema.json>` using the [official tagged schema](https://github.com/citation-file-format/citation-file-format/blob/1.2.0/schema.json). Document findings and tool limits in the [audit](../docs/DOCUMENTATION_AUDIT.md).

Before changing a diagram, update the Mermaid source and regenerate its matching SVG. The [diagram catalog](../docs/diagrams/README.md) maintains the source-to-render links.
