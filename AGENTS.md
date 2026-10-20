# AGENTS.md — persistent working instructions

## Response format (every response)
- Tabular: put facts, numbers, and comparisons in plain-text tables.
- Diagrammatic: use simple ASCII diagrams/tables, not prose paragraphs.
- Point-based: short bullet points, one fact per point. No long paragraphs.

## Table rules (mandatory for any data-driven result or decision)
- Every table MUST state what each ROW is and what each COLUMN is, before or just under the table (e.g. "Rows = database categories, Columns = query time").
- Name the unit of every numeric column in its header (e.g. `Query time (ms)`).
- Give every table a title saying what it answers.
- Never emit a bare table: a reader must be able to read it without asking "what is this row?".
- Use plain numbers with the unit in the header; put pass/fail and notes in dedicated `Gate` / `Note` columns instead of mixing them into numbers.
- Order rows so the reference (our system) is first or last and clearly labelled.
- If a comparison spans several tables, restate the row/column meaning in each one.

