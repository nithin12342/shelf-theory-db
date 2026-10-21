# AGENTS.md - project instructions for the d2 repository

These instructions apply to every response in this project.

The base writing standard is **ASD-STE100 Simplified Technical English**. ASD-STE100 is the controlled language that aerospace companies use for maintenance manuals. ASD Europe published the standard. The current issue is Issue 9, from January 15, 2025. The official text is free at <https://www.asd-ste100.org/>.

The global rules at `~/.config/opencode/instructions.md` hold the full rule set. This file holds the rules for this project.

---

## 1. Writing standard (ASD-STE100)

### 1.1 Sentences

1. Use the active voice. Name the actor.
2. Use simple tenses. Use present, past, or future tense. Do not use present perfect.
3. Use can, will, and must for the modals. Do not use should, would, may, might, or could.
4. Keep the articles. Keep the word "that".
5. Do not use contractions.
6. Do not use semicolons. Write two sentences.
7. Do not use em-dashes. Name the relation with because, but, or for example.
8. Keep most sentences near 20 words or fewer.

### 1.2 Words

1. Define a concept term at its first use. Use fewer than ten words for the definition.
2. Use one word for one meaning in the whole answer.
3. Write "make sure that" for check, verify, confirm, validate, and ensure.
4. Write "configuration" for config, settings, and options.
5. Spell out every abbreviation at first use, in the same sentence.
6. Do not use short codes on their own, such as B1, B2, B3, N7, P3, or S4.
7. If a code is a required identifier, write the plain-English name next to it every time.
8. Delete words that carry no fact. Common examples are simply, robust, powerful, comprehensive, leverage, crucial, and seamlessly.
9. Do not write "not just X, it is Y". Do not use "studies show". Name the source or drop the claim.

### 1.3 Safety markers

1. Use WARNING for a risk of injury. Use CAUTION for a risk of damage. Use WARNING if both risks apply.
2. Put the command or the condition first. Give the risk after the command.
3. Keep the marker under 20 words.

### 1.4 Check before you send

1. Search for the em-dash character. Remove each one.
2. Search for a semicolon. Write two sentences instead.
3. Search for "should", "may", "might", "could", and "would". Replace each one.
4. Count the words in your three longest sentences. Split any sentence over 20 words.
5. Show the evidence. Do not claim that you did the work.

---

## 2. Response format

### 2.1 Shape of the answer

1. Give the answer or the result first, in one or two sentences.
2. Give the detail as numbered steps.
3. Give the evidence.
4. Give the warnings, cautions, and notes.

### 2.2 Tables

1. Give every table a title that says what it answers.
2. State what each row is and what each column is, before or just under the table.
3. Put the unit in the header of every numeric column, for example "Query time (ms)".
4. Put pass or fail, and any remarks, in their own column. Do not mix them into a number.
5. Order the rows so the reference system is first or last, and label it clearly.
6. If a comparison spans several tables, restate the row and column meaning in each table.
7. Write each column header in short words. Do not use an abbreviation in a header.

### 2.3 Diagrams and lists

1. Use simple plain-text diagrams for flows, hierarchies, and pipelines. Label every element.
2. Use short bullets for lists. Put one fact in each bullet.
3. Do not write a paragraph longer than four lines.
4. Do not use a paragraph where a list or a table is clearer.
5. Do not use bold as decoration. Do not use emoji as structure.

---

## 3. Project rules

1. Push to the remote only with `py push_personal.py`. Read `PUSH_RULES.md` before the first push.
2. Do not run `git push`, `git fetch`, `git pull`, or any other command that touches a remote.
3. Keep `SKELETON.md` up to date. It is the source of truth for this repository.
4. Follow the engineering method in Part 5 of the global rules file. Load the `intention-engineering` skill first.
