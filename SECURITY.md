# OCULUS Security and Privacy

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
