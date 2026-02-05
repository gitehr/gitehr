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

## Implementation Steps

1. Design data storage schema and map RM classes to storage.
2. Implement REST endpoints and data serialization.
3. Integrate archetype/template validation.
4. Add conformance manifest and OPTIONS endpoint.
5. Run and pass openEHR conformance tests.
6. Document and maintain implementation.

---

This document describes the requirements and plan for native openEHR entity support in GitEHR, ensuring full conformance and interoperability. For FHIR integration, see `fhir.md`.
