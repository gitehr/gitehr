<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Clinical Calculators in GitEHR

## Goal

Provide a comprehensive library of open-source clinical calculators integrated with GitEHR's journal and state management, enabling clinicians to perform evidence-based calculations directly within the EHR and automatically record both inputs and results.

## Scope

- **RCPCH Digital Growth Charts**: UK-specific paediatric growth assessment tools (centiles, z-scores, SDS)
- **MDCalc-style calculators**: Wide range of clinical decision support tools covering:
  - Cardiology (CHADS2, CHA2DS2-VASc, Wells, GRACE, TIMI)
  - Renal (eGFR, CrCl, FENa)
  - Respiratory (CURB-65, PSI/PORT, PFT interpretation)
  - Neurology (NIH Stroke Scale, ABCD2, ICH Score)
  - Emergency (GCS, Trauma scores, Sepsis)
  - Oncology (TNM staging, prognostic scores)
  - And many others from established clinical practice

## Architecture

### Workspace Structure

GitEHR will adopt a **Cargo workspace** structure to support the calculators as an independent crate:

```
gitehr/
├── Cargo.toml                    # Workspace root
├── cli/                          # Renamed from src/
│   ├── Cargo.toml
│   └── src/
├── gitehr-calculators/           # New crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                # Public calculator API
│   │   ├── growth/               # RCPCH growth charts
│   │   │   ├── mod.rs
│   │   │   ├── centiles.rs
│   │   │   └── lms.rs
│   │   ├── cardiology/
│   │   │   ├── mod.rs
│   │   │   ├── chads2.rs
│   │   │   └── wells.rs
│   │   ├── renal/
│   │   │   ├── mod.rs
│   │   │   └── egfr.rs
│   │   ├── respiratory/
│   │   ├── neurology/
│   │   ├── emergency/
│   │   ├── models.rs             # Common data types
│   │   └── validation.rs         # Input validation
│   └── tests/
│       └── integration.rs
├── mcp/                          # MCP server crate
└── gui/
```

### Calculator Crate Design

The `gitehr-calculators` crate will be:
- **Open source** (AGPL-3.0-or-later to match GitEHR)
- **Independently versioned** (semantic versioning)
- **Well-tested** (unit tests for all calculators with clinical validation)
- **Documented** (clinical references, citations, validation studies)
- **Type-safe** (strong typing for inputs/outputs, compile-time safety)

### Public API Design

```rust
// gitehr-calculators/src/lib.rs
pub mod growth;
pub mod cardiology;
pub mod renal;
pub mod respiratory;
pub mod neurology;
pub mod emergency;

pub use models::{CalculatorInput, CalculatorResult, CalculatorError};

// Example usage
pub fn calculate(
    calculator_name: &str,
    inputs: serde_json::Value,
) -> Result<CalculatorResult, CalculatorError>;
```

### Example Calculator Implementation

```rust
// gitehr-calculators/src/cardiology/chads2.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Chads2Input {
    pub age_over_75: bool,
    pub congestive_heart_failure: bool,
    pub hypertension: bool,
    pub diabetes: bool,
    pub prior_stroke_tia: bool,  // Worth 2 points
}

#[derive(Debug, Serialize)]
pub struct Chads2Result {
    pub score: u8,
    pub risk_category: String,
    pub annual_stroke_risk_percent: f64,
    pub recommendation: String,
    pub citation: String,
}

pub fn calculate_chads2(input: Chads2Input) -> Chads2Result {
    let mut score = 0;
    if input.age_over_75 { score += 1; }
    if input.congestive_heart_failure { score += 1; }
    if input.hypertension { score += 1; }
    if input.diabetes { score += 1; }
    if input.prior_stroke_tia { score += 2; }
    
    let (risk_category, annual_risk, recommendation) = match score {
        0 => ("Low", 1.9, "Aspirin or no therapy"),
        1 => ("Moderate", 2.8, "Aspirin or anticoagulation"),
        2..=6 => ("Moderate-High", 4.0 + (score - 2) as f64 * 2.0, "Anticoagulation recommended"),
        _ => ("High", 18.2, "Anticoagulation strongly recommended"),
    };
    
    Chads2Result {
        score,
        risk_category: risk_category.to_string(),
        annual_stroke_risk_percent: annual_risk,
        recommendation: recommendation.to_string(),
        citation: "Gage BF, et al. JAMA. 2001;285(22):2864-2870.".to_string(),
    }
}
```

## CLI Integration

### New Command: `gitehr calc`

```bash
gitehr calc <calculator> [OPTIONS]

# Examples:
gitehr calc chads2 --age-over-75 --hypertension --diabetes
gitehr calc egfr --creatinine 1.2 --age 65 --sex female --race white
gitehr calc growth --age-months 24 --weight-kg 12.5 --height-cm 85 --sex male
```

### Implementation Path

1. **New command module**: `cli/src/commands/calculator.rs`
2. **Command enum addition** in `main.rs`:
```rust
enum Commands {
    // ... existing commands
    #[command(subcommand)]
    Calc {
        command: CalcCommands,
    },
}

enum CalcCommands {
    Chads2 { /* fields */ },
    Egfr { /* fields */ },
    Growth { /* fields */ },
    List,  // List all available calculators
}
```

3. **Dependency** in `cli/Cargo.toml`:
```toml
[dependencies]
gitehr-calculators = { path = "../gitehr-calculators" }
```

### Journal Integration

Calculator results are automatically recorded as journal entries with structured metadata:

```yaml
---
parent_hash: "abc123..."
parent_entry: "20260306T120000.000Z-previous.md"
timestamp: "2026-03-06T12:30:00Z"
author: "dr-jones"
calculator:
  type: "CHADS2"
  version: "1.0.0"
  inputs:
    age_over_75: true
    congestive_heart_failure: false
    hypertension: true
    diabetes: true
    prior_stroke_tia: false
  result:
    score: 3
    risk_category: "Moderate-High"
    annual_stroke_risk_percent: 8.5
    recommendation: "Anticoagulation recommended"
  citation: "Gage BF, et al. JAMA. 2001;285(22):2864-2870."
---

# Clinical Calculation: CHADS2 Score

**Score**: 3 (Moderate-High risk)

**Annual stroke risk**: 8.5%

**Recommendation**: Anticoagulation recommended

**Criteria met**:
- Age > 75 years
- Hypertension
- Diabetes mellitus

**Reference**: Gage BF, et al. JAMA. 2001;285(22):2864-2870.
```

### State File Storage

Latest calculator results can be stored in `state/calculations/` for quick reference:

```
state/
└── calculations/
    ├── chads2-latest.json
    ├── egfr-latest.json
    └── growth-latest.json
```

Example `state/calculations/chads2-latest.json`:
```json
{
  "calculator": "CHADS2",
  "version": "1.0.0",
  "calculated_at": "2026-03-06T12:30:00Z",
  "calculated_by": "dr-jones",
  "inputs": {
    "age_over_75": true,
    "congestive_heart_failure": false,
    "hypertension": true,
    "diabetes": true,
    "prior_stroke_tia": false
  },
  "result": {
    "score": 3,
    "risk_category": "Moderate-High",
    "annual_stroke_risk_percent": 8.5,
    "recommendation": "Anticoagulation recommended"
  },
  "citation": "Gage BF, et al. JAMA. 2001;285(22):2864-2870.",
  "journal_entry": "journal/20260306T123000.000Z-uuid.md"
}
```

## GUI Integration

### Calculator Panel

Add a dedicated calculator section in the Mantine UI:

```typescript
// gui/src/api/gitehr.ts
export async function calculateClinical(
  repoPath: string,
  calculator: string,
  inputs: Record<string, any>
): Promise<CalculatorResult> {
  return await invoke("calculate_clinical", {
    repoPath,
    calculator,
    inputs,
  });
}
```

### Tauri Command

```rust
// gui/src-tauri/src/lib.rs
#[tauri::command]
fn calculate_clinical(
    repo_path: String,
    calculator: String,
    inputs: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use gitehr_calculators::calculate;
    
    // Run calculation
    let result = calculate(&calculator, inputs)
        .map_err(|e| e.to_string())?;
    
    // Create journal entry
    // ... (similar to add_journal_entry)
    
    Ok(serde_json::to_value(result).unwrap())
}
```

### UI Components

- **Calculator Selector**: Dropdown or searchable list of available calculators
- **Input Form**: Dynamic form based on calculator requirements
- **Result Display**: Card showing score, interpretation, recommendation, citation
- **History**: List of recent calculations from `state/calculations/`
- **Chart Integration**: For growth charts, display centile curves with plotted data points

## RCPCH Digital Growth Charts

### Special Requirements

The RCPCH digital growth charts require:

1. **Reference data**: LMS parameters for UK90 reference population
2. **Age calculation**: Precise gestational-age correction for premature infants
3. **Centile calculation**: Z-score to centile conversion
4. **Chart rendering**: Visual display of growth curves (GUI feature)

### Data Sources

- **UK-WHO growth charts** (0-4 years): WHO 2006 standard
- **UK90 growth charts** (4-20 years): British 1990 reference data
- **Licensed from RCPCH**: Verify licensing terms for commercial use

### Implementation Notes

```rust
// gitehr-calculators/src/growth/mod.rs
pub struct GrowthInput {
    pub age_months: f64,
    pub measurement_type: MeasurementType,  // Weight, Height, BMI, Head Circumference
    pub value: f64,
    pub sex: Sex,
    pub gestational_age_weeks: Option<f64>,  // For prematurity correction
}

pub enum MeasurementType {
    Weight,
    Height,
    BMI,
    HeadCircumference,
    MidUpperArmCircumference,
}

pub struct GrowthResult {
    pub centile: f64,
    pub z_score: f64,
    pub sds: f64,  // Standard deviation score
    pub interpretation: String,
    pub corrected_age_months: Option<f64>,  // If gestational correction applied
}
```

## Calculator Library

### Proposed Initial Set

**Cardiology** (10 calculators):
- CHADS2 / CHA2DS2-VASc
- Wells Score (DVT/PE)
- GRACE Score
- TIMI Risk Score
- Framingham Risk Score
- ASCVD Risk Calculator
- HAS-BLED
- HEART Score
- Revised Cardiac Risk Index
- PERC Rule

**Renal** (5 calculators):
- eGFR (CKD-EPI, MDRD)
- Creatinine Clearance (Cockcroft-Gault)
- Fractional Excretion of Sodium (FENa)
- Fractional Excretion of Urea (FEUrea)
- Kidney Failure Risk Equation

**Respiratory** (5 calculators):
- CURB-65
- PSI/PORT Score
- PFT Interpretation
- BODE Index
- A-a Gradient

**Neurology** (5 calculators):
- NIH Stroke Scale (NIHSS)
- ABCD2 Score
- ICH Score
- Hunt and Hess Scale
- Fisher Scale

**Emergency** (8 calculators):
- Glasgow Coma Scale (GCS)
- Revised Trauma Score
- ISS (Injury Severity Score)
- qSOFA
- SOFA Score
- SIRS Criteria
- Modified Early Warning Score (MEWS)
- Pediatric Trauma Score

**Obstetrics** (3 calculators):
- Bishop Score
- Edinburgh Postnatal Depression Scale
- WHO Partograph

**Paediatrics** (5 calculators):
- RCPCH Growth Charts (weight, height, BMI, head circumference)
- Pediatric Early Warning Score (PEWS)
- Apgar Score
- Pediatric GCS
- Centor Score (Paediatric modification)

**Total**: ~40 calculators in initial release

## Clinical Validation

Each calculator must include:

1. **Primary citation**: Peer-reviewed publication
2. **Validation studies**: Evidence of clinical utility
3. **Test cases**: Known inputs/outputs from literature
4. **Limitations**: Known edge cases and contraindications
5. **Updates**: Process for incorporating guideline changes

## Implementation Steps

1. **Create workspace structure** - Convert to Cargo workspace
2. **Create `gitehr-calculators` crate** - Initial scaffolding
3. **Implement core calculators** - Start with CHADS2, eGFR, CURB-65 (high-utility, well-validated)
4. **Add CLI command** - `gitehr calc` with subcommands
5. **Integrate with journal** - Automatic recording of calculations
6. **Add state storage** - Latest results in `state/calculations/`
7. **GUI integration** - Tauri command + React components
8. **RCPCH growth charts** - Special implementation with reference data
9. **Expand calculator library** - Add remaining calculators incrementally
10. **Documentation** - Clinical usage guide, API reference, validation evidence
11. **Testing** - Comprehensive test suite with clinical validation cases
12. **Versioning** - Establish update process for guideline changes

## Licensing Considerations

- **GitEHR calculators crate**: AGPL-3.0-or-later (consistent with main project)
- **Clinical algorithms**: Most are in public domain or published with permissive use
- **RCPCH growth charts**: Verify licensing terms with RCPCH for commercial distribution
- **MDCalc**: Ensure no IP violations; implement from primary literature sources
- **Citations**: All calculators must cite original publications and validation studies

## Open Questions

- Should calculators support units conversion (metric/imperial)?
- Should historical calculation results be queryable via `gitehr calc history`?
- Should GUI include printable reports for calculator results?
- Should calculators integrate with FHIR Observations for standardized data exchange?
- Should we support custom/user-defined calculators via plugin system?

## Future Enhancements

- **Calculator plugins**: Allow third-party calculator contributions
- **Real-time updates**: Fetch latest guideline changes from registry
- **Decision trees**: Multi-step clinical algorithms beyond simple scores
- **Risk prediction models**: More complex ML-based prognostic calculators
- **Integration with state**: Auto-populate calculator inputs from current patient state
- **Trending**: Show calculation results over time with charts
- **Alerts**: Trigger warnings for high-risk scores

---

This specification establishes GitEHR as a comprehensive clinical decision support tool with auditable, version-controlled calculation results integrated into the patient's permanent medical record.
