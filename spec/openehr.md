<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# openEHR Native Entity Support in GitEHR

## Goal

Enable GitEHR to store, serve, and validate native openEHR Reference Model (RM) entities, supporting full openEHR conformance—including versioning, audit, archetype/template validation, and REST API compatibility.

## Scope

- **Native openEHR RM entities**: EHR, VERSIONED_COMPOSITION, COMPOSITION, ENTRY (OBSERVATION, EVALUATION, INSTRUCTION, ACTION, ADMIN_ENTRY), SECTION, FOLDER, CONTRIBUTION, AUDIT_DETAILS, PARTY_PROXY, etc.
- **Archetype/template support**: All data validated against openEHR archetypes and templates (CKM mirror, operational templates).
- **REST API**: All mandatory openEHR REST endpoints, resource identification, and content negotiation.
- **Versioning and audit**: Full support for versioning, audit trail, and change control as per openEHR specs.
- **Interoperability**: Canonical XML/JSON formats, alternative formats (SDT, FLAT, STRUCTURED), and conformance manifest.

## Data Model

- Store all EHR data as native openEHR RM classes, including all required attributes and relationships.
- Support archetype_node_id, template_id, and archetyped ITEM_STRUCTURE for all relevant entities.
- Use UUIDs for EHR, VERSIONED_OBJECT, VERSION, and other identifiers.
- All change-controlled resources must be versioned and auditable.

## Data Serialization and Formats

- Support openEHR XML and JSON canonical formats (valid against XSDs/JSON Schemas).
- Use lower_snake_case for JSON, include \_type for polymorphism.
- Support alternative formats (SDT, FLAT, STRUCTURED) for interoperability.

## REST API Conformance

- Implement all mandatory openEHR REST endpoints: /ehr, /composition, /folder, /contribution, /definition, /query, etc.
- Use correct URIs and identifiers (versioned_object_uid, version_uid, ehr_id, template_id).
- Support GET, POST, PUT, DELETE, OPTIONS; implement all required HTTP headers (openEHR-VERSION, openEHR-AUDIT_DETAILS, openEHR-TEMPLATE_ID, ETag, Last-Modified, Location, Prefer, If-Match).
- Support authentication/authorization and correct status codes.
- Content negotiation for XML, JSON, and alternative formats.

## Archetype and Template Support

- Validate all data against archetypes/templates from CKM mirror.
- Expose /definition endpoint for archetype/template management.

## Query and AQL Support

- Implement /query endpoint for Archetype Query Language (AQL) queries over native RM data.

## Versioning, Audit, and Change Control

- All changes tracked via CONTRIBUTION and VERSION objects.
- Store and expose audit details (who, when, why, change_type, etc.).

## Interoperability and Conformance Manifest

- Expose OPTIONS endpoint for conformance profile (solution, version, vendor, restapi_specs_version, endpoints).
- Validate implementation against openEHR ITS (XML, JSON, REST) using published schemas and OpenAPI validation files.

## Additional Requirements

- All relevant RM entities must record time (ISO 8601), context, participations, healthcare facility, etc.
- Use PARTY_SELF and PARTY_IDENTIFIED for subject and provider references; support external demographic services.

## Testing and Certification

- Run openEHR conformance test suite (all endpoints, data formats, versioning, audit, archetype/template support, query).
- Document all implementation decisions and deviations.

## Implementation Strategy: Hybrid Approach with Archie

### Recommended Approach

Based on comprehensive research, a **hybrid architecture** is recommended for GitEHR's openEHR implementation:

1. **Native Rust RM structures** for core data model
2. **Archie service** for complex archetype/template operations
3. **Gradual migration** to full Rust implementation over time

### Why This Approach?

**Archie** is the official openEHR Foundation library (Java, Apache 2.0) that provides comprehensive archetype parsing, validation, and operational template generation. However, direct Java-Rust integration is impractical.

**Key Challenges:**
- No mature Rust openEHR libraries exist
- JNI integration is complex and fragile
- Building full openEHR RM + archetype engine from scratch = 12-18 months effort
- GitEHR only needs subset of openEHR functionality initially

**Hybrid Solution Benefits:**
- ✅ Quick time-to-market (weeks vs. months)
- ✅ Ensures openEHR compliance via Archie
- ✅ Maintains Rust-native architecture for core features
- ✅ Provides migration path to full Rust implementation

### Architecture

```
┌─────────────────────────────────────────┐
│ GitEHR Repository (Rust)                │
│                                         │
│  /openehr/                              │
│  ├── rm/  (Rust RM structures)         │
│  │   ├── composition.rs                │
│  │   ├── observation.rs                │
│  │   └── data_structures.rs            │
│  ├── instances/  (JSON compositions)    │
│  │   └── COMPOSITION/comp-001.json     │
│  └── templates/  (OPT files)           │
│      └── encounter-v1.opt              │
└─────────────────────────────────────────┘
         │
         │ HTTP/REST API
         ↓
┌─────────────────────────────────────────┐
│ Archie Service (Java/Spring Boot)      │
│                                         │
│  ┌─────────────────────────┐           │
│  │ Archetype Validation    │           │
│  │ Template Flattening     │           │
│  │ ADL Parsing             │           │
│  │ OPT Generation          │           │
│  └─────────────────────────┘           │
│            │                            │
│            v                            │
│    Archie Library v3.17.0              │
└─────────────────────────────────────────┘
```

### Implementation Phases

#### Phase 1: Basic RM Structures (Months 1-2)

Implement core openEHR RM structures in Rust:

```rust
// cli/src/openehr/rm/composition.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Composition {
    #[serde(rename = "_type")]
    pub type_name: String,  // "COMPOSITION"
    
    pub uid: Option<String>,
    pub archetype_node_id: String,
    pub name: DvText,
    pub archetype_details: Option<Archetyped>,
    pub content: Vec<ContentItem>,
    pub context: Option<EventContext>,
    // ... other fields per openEHR RM spec
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "_type")]
pub enum ContentItem {
    #[serde(rename = "OBSERVATION")]
    Observation(Observation),
    #[serde(rename = "EVALUATION")]
    Evaluation(Evaluation),
    #[serde(rename = "INSTRUCTION")]
    Instruction(Instruction),
    // ... other types
}
```

**Scope:**
- COMPOSITION, OBSERVATION, EVALUATION, INSTRUCTION, ACTION
- Data structures (DvText, DvQuantity, DvDateTime, etc.)
- JSON serialization (FLAT format for simplicity)
- Basic field validation (required fields, types)

**Dependencies:**
- `serde` + `serde_json` for JSON handling
- Custom validation logic

#### Phase 2: Archie REST Service (Months 2-3)

Deploy Archie as a microservice for complex operations:

```java
// archie-service/src/main/java/org/gitehr/archie/ArchetypeController.java
@RestController
@RequestMapping("/api/v1/archetype")
public class ArchetypeController {
    
    @PostMapping("/validate")
    public ResponseEntity<ValidationResponse> validate(
            @RequestBody ArchetypeRequest request) {
        ADLParser parser = new ADLParser();
        Archetype archetype = parser.parse(request.getAdl());
        
        ArchetypeValidator validator = new ArchetypeValidator(metaModels);
        ValidationResult result = validator.validate(archetype);
        
        return ResponseEntity.ok(
            ValidationResponse.fromResult(result)
        );
    }
    
    @PostMapping("/flatten")
    public ResponseEntity<FlattenResponse> flatten(
            @RequestBody TemplateRequest request) {
        Flattener flattener = new Flattener(repository, metaModels);
        OperationalTemplate opt = flattener.flatten(request.getTemplate());
        
        return ResponseEntity.ok(
            FlattenResponse.fromOpt(opt)
        );
    }
}
```

**Deployment:**
```yaml
# docker-compose.yml
services:
  archie:
    image: gitehr/archie-service:latest
    ports:
      - "8080:8080"
    environment:
      - SPRING_PROFILES_ACTIVE=production
```

**GitEHR Integration:**
```rust
// cli/src/openehr/archie_client.rs
pub struct ArchieClient {
    client: reqwest::Client,
    base_url: String,
}

impl ArchieClient {
    pub async fn validate_archetype(&self, adl: &str) 
        -> Result<ValidationResult> {
        let response = self.client
            .post(&format!("{}/api/v1/archetype/validate", self.base_url))
            .json(&ArchetypeRequest { adl: adl.to_string() })
            .send()
            .await?
            .json::<ValidationResponse>()
            .await?;
        
        Ok(response.into())
    }
}
```

#### Phase 3: REST API Implementation (Months 3-4)

Implement mandatory openEHR REST endpoints:

```rust
// cli/src/openehr/api.rs
pub async fn post_composition(
    ehr_id: &str,
    composition: &Composition,
    template_id: &str,
) -> Result<String> {
    // 1. Validate against template (via Archie service)
    let valid = archie_client.validate_composition(composition, template_id).await?;
    
    // 2. Store in /openehr/instances/
    let comp_uid = store_composition(ehr_id, composition)?;
    
    // 3. Create journal entry
    create_journal_entry_for_composition(ehr_id, &comp_uid)?;
    
    Ok(comp_uid)
}
```

#### Phase 4: Incremental Rust Features (Months 4+)

Gradually implement native Rust features to reduce dependency on Archie:

1. **Basic archetype validation** (subset of ADL features)
2. **AQL query parsing** (using `nom` or `pest`)
3. **Simple path queries** (`/content[at0001]/data`)
4. **Eventually:** Full archetype engine (if needed)

### Repository Layout

```
/openehr/
  ├── templates/              # Operational templates (.opt)
  │   └── encounter-v1.opt
  ├── instances/              # RM instances (JSON)
  │   ├── COMPOSITION/
  │   │   └── 550e8400-e29b-41d4-a716-446655440000.json
  │   └── EHR/
  │       └── ehr-123.json
  └── indexes/                # Optional search indexes
      └── compositions.db
```

### CLI Commands

```bash
# Import openEHR composition
gitehr openehr import <composition.json> --template encounter-v1

# Validate composition
gitehr openehr validate <composition.json>

# Query with AQL
gitehr openehr query "SELECT c FROM COMPOSITION c WHERE c/context/start_time > '2026-01-01'"

# List templates
gitehr openehr templates

# Export composition
gitehr openehr export <composition-uid>
```

### Archie Service API

**Endpoints:**
- `POST /api/v1/archetype/validate` - Validate archetype (ADL)
- `POST /api/v1/template/flatten` - Generate OPT from template
- `POST /api/v1/composition/validate` - Validate composition against template
- `GET /api/v1/definition/{id}` - Get archetype/template definition

### Alternative: Service-Free Approach

For minimal openEHR support without Archie service:

1. **FLAT JSON only** - Skip canonical format
2. **No archetype validation** - Trust input data
3. **Basic RM structures** - Just for storage
4. **Future migration** - Add Archie service later when needed

This reduces complexity but sacrifices openEHR conformance.

### Dependencies

**Archie Service:**
- Java 11+
- Archie library v3.17.0 (Apache 2.0)
- Spring Boot 3.x

**GitEHR (Rust):**
- `serde` + `serde_json` - JSON handling
- `reqwest` - HTTP client for Archie service
- `tokio` - Async runtime

### Resources

- **Archie GitHub**: https://github.com/openEHR/archie
- **openEHR Specifications**: https://specifications.openehr.org
- **openEHR REST API**: https://specifications.openehr.org/releases/ITS-REST/latest
- **openEHR Discourse**: https://discourse.openehr.org (Rust discussions available)

### Migration Path

**Short-term (0-6 months):**
- Archie service handles all complex operations
- Rust handles basic RM and REST API

**Medium-term (6-12 months):**
- Implement native validation for common archetypes
- Reduce Archie dependency to edge cases

**Long-term (12+ months):**
- Evaluate full Rust implementation
- Consider creating/contributing to `openehr-rs` library
- Potentially replace Archie entirely

This approach balances pragmatism (quick delivery, openEHR compliance) with long-term vision (Rust-native implementation).

---

This document describes the requirements and plan for native openEHR entity support in GitEHR, ensuring full conformance and interoperability. For FHIR integration, see `fhir.md`.
