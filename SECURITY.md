# OCULUS Security and Privacy

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub's
**[private vulnerability reporting](https://github.com/ARASKOVA-labs/OCULUS/security/advisories/new)**,
or email dev@araskova.com. Do not open a public issue for security problems.

Include what you can: the affected component (UI, Rust layer, native engine, installer), the version,
steps to reproduce, and the impact.

| Stage | Target |
|---|---|
| Acknowledgement | 3 business days |
| Triage and severity | 7 days |

Supported versions: the latest release.

## Threat model
OCULUS processes potentially sensitive documents. Treat all document content as untrusted.

## Rules
- No unrestricted shell execution.
- No arbitrary frontend-to-native command bridge.
- Validate file paths.
- Prevent path traversal.
- Use least-privilege filesystem access.
- Never log document contents by default.
- Never log secrets.
- Never upload files silently.
- Keep telemetry disabled by default.
- Secure temporary files.
- Delete temporary artifacts when no longer required.
- Clearly disclose any future network feature.

## Native boundary
All C++ inputs must be validated by Rust before crossing the FFI boundary where practical.

## Supply chain
- Pin dependencies for releases.
- Review native dependencies.
- Track licenses.
- Prefer maintained libraries.
- Record model licenses and redistribution terms.
