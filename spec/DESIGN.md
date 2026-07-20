# GitEHR GUI - Design Document

> **For agentic coding sessions:** paste this document at the start of every session
> working on the GUI. Follow it strictly. Do not introduce new component libraries,
> styling approaches, or UI patterns that are not described here without explicit
> instruction.

---

## What we are building

A Tauri desktop application that lets a clinician or a patient managing their own
condition create, view, and navigate a GitEHR record. The backend is Rust (via issuing `gitehr` CLI
commands). The frontend is React + Mantine + TypeScript.

The application must feel calm, trustworthy, and clinical - not like a consumer app.
Density should be moderate: enough information visible at once to be useful, not so
much that it is overwhelming. Think of a well-designed GP system rather than a
dashboard product.

---

## Technology constraints - read before writing any code

### Framework versions

**Check `gui/package.json` first and use whatever versions are already installed.**
Do not upgrade or downgrade any dependency without explicit instruction.

Key version-specific notes:
- Mantine v7 uses CSS Modules and PostCSS. The `sx` prop does not exist in v7+.
  Use the `style` prop for one-off inline styles, or CSS Modules for anything
  reusable.
- Mantine v8 (released May 2025) has additional breaking changes from v7. If the
  project is on v8, consult the v8 migration guide before assuming v7 patterns work.
- Do not mix version patterns. If unsure which version is installed, run
  `grep mantine gui/package.json` before writing any component code.

### Icon library

Use **Tabler Icons** (`@tabler/icons-react`) exclusively. Mantine is designed around
Tabler Icons and the agent knows them well. Do not import from any other icon package.
Prefer outline variants unless a filled icon communicates state (e.g. a filled
bookmark meaning "saved").

Useful icons for this project:
- `IconTimeline` - journal / timeline views
- `IconNotes` - encounter/consultation entries  
- `IconPill` - medications
- `IconAlertTriangle` - allergies and alerts
- `IconHeartbeat` - vitals / observations
- `IconUser` - patient demographics
- `IconSearch` - SNOMED lookup
- `IconGitCommit` - git-specific actions (commit, sync)
- `IconFileExport` - export / FHIR export
- `IconLock` - record security / encryption status

### Data layer - Tauri IPC

**All data comes from Tauri `invoke()` calls.** There is no REST API, no fetch to
localhost, no mock JSON files in production code.

The Tauri Rust backend works by shelling out to the `gitehr` CLI binary. When a record is open, the binary at `<recordPath>/.gitehr/gitehr` is used (falling back to `gitehr` on `$PATH`). For reads, the backend may also parse the repository's files directly (journal YAML front matter, state files, git status) rather than calling CLI commands - this is acceptable for read-only access.

Invoke signatures map to CLI operations:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Open a record: validates the directory contains .gitehr/, reads status and all state
const record = await invoke<PatientRecord>('open_record', { path: recordPath });

// Add a journal entry: shells out to `gitehr journal add`
await invoke('journal_add', { recordPath, content: markdownContent });

// Read a state file: shells out to `gitehr state get <filename>` (or reads file directly)
const raw = await invoke<string>('state_get', { recordPath, filename: 'medications.md' });

// Write a state file: shells out to `gitehr state set <filename> <content>`
await invoke('state_set', { recordPath, filename: 'medications.md', content });

// Get git/encryption status: shells out to `gitehr status`
const status = await invoke<RecordStatus>('record_status', { recordPath });
```

For the file-open dialog, use Tauri's built-in dialog plugin directly — do **not** create a custom invoke for it:

```typescript
import { open } from '@tauri-apps/plugin-dialog';

const selected = await open({ directory: true, title: 'Open GitEHR record' });
```

For development and Storybook, mock data lives in `gui/src/mocks/`. Components
accept their data as props and do not call `invoke()` directly - that happens in
page-level components or hooks. This keeps components testable and Storybook-friendly.

### SNOMED lookup - sct MCP integration

SNOMED term lookup is powered by the `sct` MCP server (local SQLite + FTS5 backend).
The Tauri backend proxies queries to `sct` and returns results.

The invoke signature to expect:

```typescript
const results = await invoke<SnomedResult[]>('search_snomed', { query: searchTerm });

interface SnomedResult {
  id: string;           // SNOMED concept ID
  preferred_term: string;
  hierarchy: string;    // e.g. "Clinical finding"
}
```

The SNOMED search UI uses `@mantine/spotlight`. See the Spotlight section below.

---

## Colour theme and visual design

Define the theme once in `gui/src/theme.ts` and import it everywhere. Do not
hardcode colour values anywhere else.

```typescript
import { createTheme, MantineColorsTuple } from '@mantine/core';

// A calm teal-blue that reads as clinical without being NHS-brand derivative.
// High contrast against white. Accessible at AA level.
const clinical: MantineColorsTuple = [
  '#e8f4f8', '#d0e8f0', '#a0d0e0', '#6eb8d0', '#45a3c0',
  '#2b93b2', '#1d7f9e', '#156a85', '#0e566c', '#084255',
];

// Severity / status colours - used on allergy, alert, and entry type indicators.
// Do not use these for decorative purposes.
const severity = {
  critical: '#c0392b',   // life-threatening allergy, critical alert
  high:     '#e67e22',   // significant but not immediately life-threatening
  moderate: '#f1c40f',   // warrants attention
  low:      '#27ae60',   // informational / normal finding
  neutral:  '#7f8c8d',   // inactive, historical, or unknown severity
};

export const theme = createTheme({
  primaryColor: 'clinical',
  colors: { clinical },
  fontFamily: 'Inter, system-ui, sans-serif',
  fontFamilyMonospace: 'JetBrains Mono, monospace',  // used for SNOMED codes, git hashes
  defaultRadius: 'sm',
  headings: {
    fontFamily: 'Inter, system-ui, sans-serif',
    fontWeight: '600',
  },
  spacing: {
    xs: '0.5rem',
    sm: '0.75rem',
    md: '1rem',
    lg: '1.5rem',
    xl: '2rem',
  },
});

// Export severity colours separately for use in component files
export { severity };
```

**Light mode only for MVP.** Dark mode can come later. Do not implement
`colorScheme` switching in the MVP.

---

## Application shell

The main window has a two-column layout:

```
+------------------+----------------------------------------+
|                  |                                        |
|   Left sidebar   |   Main content area                    |
|   (240px fixed)  |   (fills remaining width)              |
|                  |                                        |
|   Patient name   |   [active screen renders here]         |
|   DOB / NHS no.  |                                        |
|   Nav links      |                                        |
|                  |                                        |
|   Record status  |                                        |
|   (git / lock)   |                                        |
+------------------+----------------------------------------+
```

Use Mantine `AppShell` with `navbar` for this. The navbar width is 240px and does
not collapse in the MVP - no hamburger menu required yet.

The record status indicator at the bottom of the sidebar shows:
- whether the record has uncommitted changes (git dirty state)
- whether the record is encrypted at rest
- last commit hash (monospace, truncated to 8 chars)

---

## Screens - MVP scope only

Build these five screens and nothing else for the demo. Everything else is explicitly
out of scope (see below).

### 1. Record selector (welcome screen)

**Purpose:** Choose a gitehr record directory to open, or create a new one.

**Layout:** Centred card on a neutral background. No sidebar (the sidebar only
appears once a record is open).

**Components:**
- `Button` ("Open existing record") - triggers a Tauri directory picker via `@tauri-apps/plugin-dialog`. After the user picks a directory, validate that it contains a `.gitehr/` subdirectory before proceeding; show an inline error if it does not.
- `Button` variant="outline" ("Create new record") - opens the new record form (this will invoke `gitehr store init` in a chosen directory)
- A short list of recently opened records (stored in Tauri app local data) rendered
  as `NavLink` items

Note: GitEHR repositories are UUID-named directories created by `gitehr store init` under a store root directory (which contains `gitehr-mpi.json`). Users who want to navigate by the MPI can open the store root; the MVP record selector simply lets them open any directory and validates the `.gitehr/` marker.

**Do not** build a patient search or a multi-patient record manager here. One record
at a time.

---

### 2. Patient overview (sidebar content + default landing)

**Purpose:** Show who this record belongs to and provide navigation.

This is the sidebar content that persists across all screens once a record is open,
plus the default "landing" view in the main content area when no specific screen
is selected.

**Sidebar content:**
- Patient preferred name (large, `fz="lg"` weight 600) — from `record.demographics.preferred_name` falling back to `record.demographics.patient_name`
- Date of birth + calculated age — from `record.demographics.date_of_birth`
- NHS number (formatted `XXX XXX XXXX`, monospace) — from `record.demographics.nhs_number` (omit row if absent)
- Divider
- Navigation links to the other screens (Timeline, Current State, New Entry)
- Divider
- Record status indicator (see shell section above)

**Main content area (landing state):**
- A brief summary card: number of journal entries, date of first and last entry,
  active medication count, active allergy/alert count
- A "quick actions" row: "New encounter", "Record observation", "Add medication"

---

### 3. Journal timeline

**Purpose:** The primary clinical view. Chronological list of all journal entries,
newest first.

**Layout:** Full-width scrollable list in the main content area.

**Use Mantine `Timeline` component** as the base. Each entry is a `Timeline.Item`.

Each timeline item shows:
- Entry type indicator: a coloured left border or bullet using severity/entry-type
  colour (see colour map below); show neutral colour for entries with no `entry_type` (CLI-authored)
- Timestamp: date + time, human-readable ("3 May 2025 at 14:22") from the `timestamp` front matter field
- Author: who recorded this entry (from `author` front matter field; display "Unknown" if absent)
- Entry type label: "Encounter", "Observation", "Medication change", etc. — omit for CLI-authored entries that have no `entry_type`
- Summary text: first ~150 chars of the Markdown `body`, truncated with "Show more"
- SNOMED codes if present: rendered as small `Badge` components with the concept ID
  in monospace and the preferred term as the label (GUI-extended entries only)
- Entry filename: bottom-right, show the timestamped filename in monospace and muted colour. It is the entry's stable on-disk identifier; Git history provides the tamper-evident audit trail.

**Entry type colour map** (use `severity` palette + clinical primary):
- Encounter / Consultation: `clinical[5]` (teal-blue)
- Observation / Vital sign: `clinical[3]` (lighter teal)
- Medication change: `#8e44ad` (purple - distinct from clinical palette)
- Allergy / Alert: `severity.critical` or `severity.high` depending on severity
- Patient-recorded entry: `severity.low` (green - distinguishes patient from clinician)
- Administrative: `severity.neutral` (grey)

**No pagination in MVP** - render all entries. Performance optimisation (virtual
scroll) is post-MVP.

---

### 4. New entry form

**Purpose:** Record a new clinical entry with optional SNOMED coding.

**Layout:** Full-width form in the main content area, not a modal.

**Fields:**

```
Entry type         [Select - dropdown, options from entry type list above]
Date/time          [DateTimePicker - defaults to now, editable]
Author             [TextInput - pre-filled from active user in .gitehr/contributors.json, editable]
Clinical notes     [Textarea - tall, resizable, free text SOAP or narrative]
SNOMED codes       [Multi-value SNOMED lookup - see below]
Severity/priority  [SegmentedControl - Low / Moderate / High / Critical]
                   (only shown for Allergy and Alert entry types)
```

**Journal entry file format note:**

The base gitehr journal format stores `timestamp`, optional `author`, and optional `documents` as YAML front matter, with a free-text Markdown body. The GUI extends this by writing `entry_type`, `snomed_codes`, and (where applicable) `severity` as additional YAML front matter fields. CLI tools and other gitehr consumers ignore unknown front matter fields, so this is safe.

Example of a GUI-authored journal file:

```yaml
---
timestamp: '2026-05-30T14:22:00Z'
author: 'dr-smith'
entry_type: 'encounter'
snomed_codes:
  - id: '84114007'
    preferred_term: 'Heart failure (disorder)'
    hierarchy: 'Clinical finding'
---

Patient presented with worsening breathlessness on exertion...
```

**SNOMED lookup field:**

Use `@mantine/spotlight` as the search mechanism. A "Add SNOMED code" button opens
the Spotlight overlay. The user types a term, results come back from `sct` via
`invoke('search_snomed', ...)`, and selecting a result adds a `Badge` to the
"SNOMED codes" section of the form.

The Spotlight trigger button should be labelled "Add SNOMED code (Cmd+K)" on macOS
or "Add SNOMED code (Ctrl+K)" on Linux/Windows. Detect platform via
`invoke('get_platform')` or Tauri's `platform()` from `@tauri-apps/plugin-os`.

Selected codes appear as dismissible `Badge` components below the trigger button.
Each badge shows: `[concept ID] preferred term` with the concept ID in monospace.

**Submit:** A single "Save entry" button. On success, navigate to the Journal
timeline and scroll to the new entry. On failure, show a Mantine `notifications`
error toast.

Do not implement auto-save or draft saving in the MVP.

---

### 5. Current state panel

**Purpose:** Show the patient's current active clinical state - the things that
are true right now, not the historical record.

This maps to the gitehr `/state` directory in the record structure.

**State file conventions:** The GUI reads and writes structured state via three files using YAML front matter + Markdown body (the same format as journal entries). The backend reads/writes these with `gitehr state get`/`gitehr state set`:
- `state/patient.md` — patient demographics (see `PatientDemographics` type)
- `state/medications.md` — YAML front matter with `medications: [...]` array
- `state/allergies.md` — YAML front matter with `allergies: [...]` array
- `state/conditions.md` — YAML front matter with `conditions: [...]` array

**Layout:** Three stacked sections in the main content area.

**Section 1 - Active medications:**
- A `Table` with columns: Medication name | Dose | Route | Frequency | Started | Prescribed by
- Each row has an "Inactive" action (moves entry to state history, not deletion)
- An "Add medication" button at the bottom opens a dedicated medication entry modal
  (the only modal in the MVP - kept simple)

**Section 2 - Allergies and alerts:**
- A `Table` with columns: Agent | Reaction | Severity | Recorded by | Date
- Severity shown as a coloured `Badge` using the severity palette
- An "Add allergy/alert" button

**Section 3 - Active conditions/diagnoses:**
- A simpler list using `List` component
- Each item: SNOMED-coded condition name + onset date + recording clinician
- An "Add condition" button

All three sections should show an empty state illustration (simple, text-based -
no external image assets) if no entries exist, with a clear call to action.

---

## Out of scope for MVP - do not build

Do not implement these even if asked in a session, unless the main ROADMAP.md has
been updated to include them:

- FHIR export
- openEHR mapping
- Record sharing or sync between devices
- Multi-patient / patient list view
- Dark mode / colour scheme toggle
- Imaging viewer
- User accounts or authentication within the app
- Print / PDF export
- The patient-facing view (distinct from the clinician view)
- Settings screen
- Notification or reminder system
- Any network calls other than to local Tauri commands

---

## Component naming conventions

Name components after their clinical purpose, not their visual shape.

| Component file | Purpose |
|---|---|
| `JournalTimeline` | The full timeline view |
| `JournalEntry` | A single entry in the timeline |
| `EncounterForm` | The new entry form |
| `SnomedLookup` | The Spotlight-based SNOMED search |
| `SnomedBadge` | A single coded term badge (concept ID + preferred term) |
| `PatientHeader` | Sidebar patient demographics block |
| `RecordStatus` | Git/encryption status indicator |
| `MedicationTable` | Active medications table |
| `AllergyTable` | Allergies and alerts table |
| `ConditionList` | Active conditions/diagnoses list |
| `EntryTypeBadge` | Coloured badge showing entry type |
| `RecordSelector` | Welcome screen record picker |

Components live in `gui/src/components/`. Pages live in `gui/src/pages/`.
Hooks live in `gui/src/hooks/`. Types live in `gui/src/types/`.

---

## TypeScript types - core record types

Define these in `gui/src/types/record.ts`. Use these consistently - do not
redefine inline.

```typescript
export type EntryType =
  | 'encounter'
  | 'observation'
  | 'medication_change'
  | 'allergy'
  | 'alert'
  | 'patient_recorded'
  | 'administrative';

export type Severity = 'low' | 'moderate' | 'high' | 'critical';

export interface SnomedCode {
  id: string;             // SNOMED concept ID, e.g. "73211009"
  preferred_term: string; // e.g. "Diabetes mellitus"
  hierarchy: string;      // e.g. "Clinical finding"
}

// Reflects the actual gitehr journal file format: YAML front matter + Markdown body.
// entry_type, snomed_codes, and severity are GUI-extended front matter fields;
// they are written by the GUI and absent from entries created by the CLI alone.
export interface JournalEntry {
  filename: string;            // e.g. "20260205T032720.630Z-uuid.md" - serves as the unique ID
  timestamp: string;           // ISO 8601 UTC
  author?: string;             // user ID from .gitehr/contributors.json (optional)
  body: string;                // Markdown narrative body
  // GUI-extended front matter fields (absent for CLI-authored entries):
  entry_type?: EntryType;
  snomed_codes?: SnomedCode[];
  severity?: Severity;         // only for allergy/alert entry types
}

// Patient demographics are stored by convention in state/patient.md as YAML front matter.
// The Tauri backend reads and writes this file; the fields below are the front matter keys.
export interface PatientDemographics {
  patient_name: string;
  preferred_name?: string;
  date_of_birth: string;  // ISO 8601 date
  nhs_number?: string;
}

// Medications are stored in state/medications.md.
// The file contains a YAML front matter block with a `medications` array.
export interface Medication {
  id: string;
  name: string;
  snomed_code?: SnomedCode;
  dose: string;
  route: string;
  frequency: string;
  started: string;        // ISO 8601 date
  prescribed_by: string;
  active: boolean;
}

// Allergies are stored in state/allergies.md (YAML front matter, `allergies` array).
export interface Allergy {
  id: string;
  agent: string;
  snomed_code?: SnomedCode;
  reaction: string;
  severity: Severity;
  recorded_by: string;
  recorded_at: string;    // ISO 8601
}

// Conditions are stored in state/conditions.md (YAML front matter, `conditions` array).
export interface Condition {
  id: string;
  name: string;
  snomed_code?: SnomedCode;
  onset: string;          // ISO 8601 date (can be partial - year only)
  recorded_by: string;
}

// The full record as assembled by the Tauri backend from the repository's files.
// The `open_record` invoke reads all of these from the filesystem; it does not
// come from a single gitehr CLI command.
export interface PatientRecord {
  path: string;                  // filesystem path to the record root
  demographics: PatientDemographics;
  journal: JournalEntry[];       // from journal/*.md
  medications: Medication[];     // from state/medications.md
  allergies: Allergy[];          // from state/allergies.md
  conditions: Condition[];       // from state/conditions.md
  git_dirty: boolean;            // true if `git status` shows uncommitted changes
  encrypted: boolean;            // true if .gitehr/ENCRYPTED exists
  last_commit_hash?: string;     // latest git commit SHA (40 chars), truncated in UI to 8
}
```

---

## What "done" looks like for the demo

The demo is complete when the following end-to-end flow works without errors:

1. Launch the app. The record selector screen appears.
2. Open an existing gitehr record directory. The sidebar populates with patient
   demographics. The journal timeline renders with existing entries if any exist.
3. Click "New encounter". The encounter form appears.
4. Type "heart failure" into the SNOMED lookup (Spotlight overlay opens, results
   appear from the local sct database). Select "Heart failure (disorder) [84114007]".
   The code appears as a badge on the form.
5. Type clinical notes into the free text area. Set entry type to "Encounter".
6. Click "Save entry". The form submits, the timeline updates, and the new entry
   appears at the top of the journal with the correct timestamp, author, and
   SNOMED badge.
7. Click "Current state". The three sections render (may be empty). The "Add
   medication" button works and a new medication appears in the table.

That flow, recorded as a short screen capture, is the demo.
