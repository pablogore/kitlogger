# Decomposition: Formatting Pipeline

**PARENT_SPEC_ID**: `004-formatting-pipeline`

| Key | Name | Responsibility | Dependencies | SPEC_ID |
|-----|------|----------------|--------------|---------|
| AS-01 | Formatter Contract | Define the Formatter trait, FormattedRecord output type, and the formatting pipeline abstraction. | 003-structured-logging-core | `004-formatting-pipeline-as-01-formatter-contract` |
| AS-02 | Text Formatter | Implement human-readable plain text formatting of LogRecord with ordered fields and inline attribute rendering. | AS-01 | `004-formatting-pipeline-as-02-text-formatter` |
| AS-03 | JSON Formatter | Implement JSON object formatting of LogRecord with all attributes as typed JSON values. | AS-01 | `004-formatting-pipeline-as-03-json-formatter` |
| AS-04 | Logfmt Formatter | Implement logfmt key=value formatting of LogRecord with space-separated tokens. | AS-01 | `004-formatting-pipeline-as-04-logfmt-formatter` |
