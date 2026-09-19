# OCULUS UI Specification

## Design direction
Premium, minimal, technical, calm. The UI should feel like a professional native workstation rather than a generic web dashboard.

## Primary screen
1. Top navigation / product identity
2. Large drop zone
3. Recent jobs
4. Processing queue
5. Results
6. Settings

## Document workspace
Two-pane layout:
- left: page/document navigation
- center/right: original preview + extracted text

## Batch processing
Show:
- total files
- completed
- processing
- failed
- remaining
- progress

Never hide a long-running operation behind an indeterminate spinner.

## Accessibility
- keyboard navigation
- visible focus states
- readable contrast
- scalable text
- clear status messages
- no color-only status communication

## UX principles
- First OCR should require as few decisions as possible.
- Advanced options should be progressive disclosure.
- Destructive operations require confirmation.
- Never silently overwrite output.
- Explain privacy locally and plainly.
