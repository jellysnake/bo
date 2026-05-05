# Dogfood corpora

Each subdirectory is one dogfood tree seed.

```text
corpora/<tree-name>/urls.txt
```

Running:

```bash
./scripts/dogfood-collect <tree-name>
```

creates one isolated temp bo tree and collects every URL from that corpus into that tree.

Corpora are not combined during dogfooding. If we add more corpora later, each one should represent a distinct tree shape a user might plausibly maintain.
