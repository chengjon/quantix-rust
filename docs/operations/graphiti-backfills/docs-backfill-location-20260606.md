Graphiti backfill required

Date: 2026-06-06
Group: quantix_rust_docs
Episode: 490a73e8-5626-49b1-bf4c-e0e0ef46b3b4
Status: processing after retry

Decision:
Local Graphiti backfill records that should be durable and reviewable belong under:

docs/operations/graphiti-backfills/

Rationale:
- /tmp is suitable for transient operational notes, but it can be cleaned and is not auditable.
- Repository docs keep failed-ingest summaries durable until Graphiti can be backfilled.
- The operations subdirectory keeps these records separate from product docs, feature specs, and design reviews.

Applied example:
The import-klines closure backfill was committed at:

docs/operations/graphiti-backfills/import-klines-closure-20260606.md

Related commit:
ade14e59151774a869c8525cf9132869861cad45 docs: record graphiti backfill for import-klines closure
