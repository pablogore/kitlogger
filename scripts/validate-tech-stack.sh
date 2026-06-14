#!/usr/bin/env bash

set -euo pipefail

usage() {
    cat <<'EOF'
Usage: validate-tech-stack.sh --feature-dir PATH --phase plan|tasks|implement

Validates that the active specification declares its technology stack and that
all recognized technology references in specification and planning artifacts
are declared. The validator never writes files.
EOF
}

FEATURE_DIR=""
PHASE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --phase)
            PHASE="${2:-}"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "ERROR: Unknown option '$1'" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ -z "$FEATURE_DIR" || -z "$PHASE" ]]; then
    usage >&2
    exit 2
fi

case "$PHASE" in
    plan|tasks|implement) ;;
    *)
        echo "ERROR: Unsupported phase '$PHASE'" >&2
        exit 2
        ;;
esac

FEATURE_DIR="$(CDPATH="" cd "$FEATURE_DIR" 2>/dev/null && pwd)" || {
    echo "ERROR: Specification directory not found: $FEATURE_DIR" >&2
    exit 1
}

REPO_ROOT="$(git -C "$FEATURE_DIR" rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$REPO_ROOT" ]]; then
    echo "ERROR: Could not resolve repository root from $FEATURE_DIR" >&2
    exit 1
fi

TECH_STACK="$FEATURE_DIR/tech-stack.yaml"
FEATURE_SPEC="$FEATURE_DIR/spec.md"
CONSTITUTION="$REPO_ROOT/.specify/memory/constitution.md"

if [[ ! -f "$TECH_STACK" ]]; then
    cat >&2 <<EOF
TECHNOLOGY CLARIFICATION REQUIRED

Missing declarations:
- language
- runtime
- testing

Missing mandatory file:
$TECH_STACK

No files may be written.
EOF
    exit 1
fi

if [[ ! -f "$FEATURE_SPEC" ]]; then
    echo "ERROR: Specification not found: $FEATURE_SPEC" >&2
    exit 1
fi

if [[ ! -f "$CONSTITUTION" ]]; then
    echo "ERROR: Constitution not found: $CONSTITUTION" >&2
    exit 1
fi

declare -a DECLARED_KEYS=()
declare -a DECLARED_VALUES=()
current_key=""

trim_yaml_value() {
    local value="$1"
    value="${value%%#*}"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    if [[ "$value" == \"*\" && "$value" == *\" ]]; then
        value="${value:1:${#value}-2}"
    elif [[ "$value" == \'*\' && "$value" == *\' ]]; then
        value="${value:1:${#value}-2}"
    fi
    printf '%s' "$value"
}

append_declaration() {
    local key="$1"
    local value="$2"
    local item
    local -a inline_items=()

    if [[ "$value" == "[]" || "$value" == "{}" || -z "$value" ]]; then
        return 0
    fi

    if [[ "$value" == \[*\] ]]; then
        value="${value:1:${#value}-2}"
        IFS=',' read -r -a inline_items <<< "$value"
        for item in "${inline_items[@]}"; do
            item="$(trim_yaml_value "$item")"
            if [[ -n "$item" ]]; then
                DECLARED_KEYS+=("$key")
                DECLARED_VALUES+=("$item")
            fi
        done
        return 0
    fi

    DECLARED_KEYS+=("$key")
    DECLARED_VALUES+=("$value")
}

while IFS= read -r raw_line || [[ -n "$raw_line" ]]; do
    [[ "$raw_line" =~ ^[[:space:]]*$ ]] && continue
    [[ "$raw_line" =~ ^[[:space:]]*# ]] && continue

    if [[ "$raw_line" =~ ^([A-Za-z][A-Za-z0-9_-]*):[[:space:]]*(.*)$ ]]; then
        current_key="${BASH_REMATCH[1]}"
        value="$(trim_yaml_value "${BASH_REMATCH[2]}")"
        append_declaration "$current_key" "$value"
    elif [[ "$raw_line" =~ ^[[:space:]]*-[[:space:]]+(.+)$ && -n "$current_key" ]]; then
        value="$(trim_yaml_value "${BASH_REMATCH[1]}")"
        append_declaration "$current_key" "$value"
    else
        echo "ERROR: Unsupported tech-stack.yaml syntax: $raw_line" >&2
        echo "Use top-level scalar values or YAML lists." >&2
        exit 1
    fi
done < "$TECH_STACK"

declare -a MISSING_KEYS=()
for required_key in language runtime testing; do
    found=false
    for index in "${!DECLARED_KEYS[@]}"; do
        key_lower="$(printf '%s' "${DECLARED_KEYS[$index]}" | tr '[:upper:]' '[:lower:]')"
        value_upper="$(printf '%s' "${DECLARED_VALUES[$index]}" | tr '[:lower:]' '[:upper:]')"
        if [[ "$key_lower" == "$required_key" ]] &&
           [[ -n "${DECLARED_VALUES[$index]}" ]] &&
           [[ "$value_upper" != "REQUIRED" ]] &&
           [[ "$value_upper" != "TBD" ]] &&
           [[ "$value_upper" != "NEEDS CLARIFICATION" ]]; then
            found=true
            break
        fi
    done
    $found || MISSING_KEYS+=("$required_key")
done

if [[ ${#MISSING_KEYS[@]} -gt 0 ]]; then
    echo "TECHNOLOGY CLARIFICATION REQUIRED" >&2
    echo >&2
    echo "Missing declarations:" >&2
    printf -- '- %s\n' "${MISSING_KEYS[@]}" >&2
    echo >&2
    echo "No files may be written." >&2
    exit 1
fi

parent_spec_id="$(
    sed -nE \
        's/.*PARENT_SPEC_ID[^A-Za-z0-9_-]*([A-Za-z0-9][A-Za-z0-9_-]*).*/\1/p;
         s/.*Parent Capability[^A-Za-z0-9_-]*([A-Za-z0-9][A-Za-z0-9_-]*).*/\1/p' \
        "$FEATURE_SPEC" | head -n 1
)"

PARENT_DIR=""
if [[ -n "$parent_spec_id" && -d "$REPO_ROOT/specs/$parent_spec_id" ]]; then
    PARENT_DIR="$REPO_ROOT/specs/$parent_spec_id"
fi

if [[ -z "$PARENT_DIR" ]]; then
    current_spec_id="$(basename "$FEATURE_DIR")"
    declare -a parent_matches=()
    while IFS= read -r index_file; do
        if grep -Eq "Specification ID[^A-Za-z0-9_-]+${current_spec_id}([^A-Za-z0-9_-]|$)" "$index_file"; then
            parent_matches+=("$(dirname "$index_file")")
        fi
    done < <(find "$REPO_ROOT/specs" -mindepth 2 -maxdepth 2 -type f \
        \( -name feature-index.md -o -name decomposition.md \) -print | sort)

    if [[ ${#parent_matches[@]} -gt 0 ]]; then
        PARENT_DIR="$(printf '%s\n' "${parent_matches[@]}" | LC_ALL=C sort -u | head -n 1)"
        unique_parent_count="$(
            printf '%s\n' "${parent_matches[@]}" | LC_ALL=C sort -u | wc -l | tr -d ' '
        )"
        if [[ "$unique_parent_count" -ne 1 ]]; then
            echo "TECHNOLOGY CONTEXT ERROR" >&2
            echo >&2
            echo "Atomic specification is indexed by multiple capabilities:" >&2
            printf -- '- %s\n' "${parent_matches[@]}" | LC_ALL=C sort -u >&2
            echo >&2
            echo "No files may be written." >&2
            exit 1
        fi
    fi
fi

ARCHITECTURE_FILE=""
CAPABILITY_FILE=""

if [[ -f "$FEATURE_DIR/architecture.md" ]]; then
    ARCHITECTURE_FILE="$FEATURE_DIR/architecture.md"
    if [[ -f "$FEATURE_DIR/capability.md" ]]; then
        CAPABILITY_FILE="$FEATURE_DIR/capability.md"
    else
        CAPABILITY_FILE="$FEATURE_SPEC"
    fi
elif [[ -n "$PARENT_DIR" && -f "$PARENT_DIR/architecture.md" ]]; then
    ARCHITECTURE_FILE="$PARENT_DIR/architecture.md"
    if [[ -f "$PARENT_DIR/capability.md" ]]; then
        CAPABILITY_FILE="$PARENT_DIR/capability.md"
    elif [[ -f "$PARENT_DIR/spec.md" ]]; then
        CAPABILITY_FILE="$PARENT_DIR/spec.md"
    fi
fi

if [[ -z "$ARCHITECTURE_FILE" || -z "$CAPABILITY_FILE" ]]; then
    cat >&2 <<EOF
TECHNOLOGY CONTEXT ERROR

Required context could not be resolved:
- architecture.md
- capability specification

Active specification:
$FEATURE_DIR

Declare the parent capability in spec.md or index the specification from one
capability feature-index.md/decomposition.md.

No files may be written.
EOF
    exit 1
fi

declare -a INPUT_FILES=(
    "$CONSTITUTION"
    "$ARCHITECTURE_FILE"
    "$CAPABILITY_FILE"
    "$FEATURE_SPEC"
)

add_file_if_present() {
    local file="$1"
    if [[ -f "$file" ]]; then
        INPUT_FILES+=("$file")
    fi
    return 0
}

add_file_if_present "$FEATURE_DIR/plan.md"
add_file_if_present "$FEATURE_DIR/tasks.md"
add_file_if_present "$FEATURE_DIR/research.md"
add_file_if_present "$FEATURE_DIR/data-model.md"
add_file_if_present "$FEATURE_DIR/quickstart.md"

if [[ -d "$FEATURE_DIR/contracts" ]]; then
    while IFS= read -r contract_file; do
        INPUT_FILES+=("$contract_file")
    done < <(find "$FEATURE_DIR/contracts" -type f -print | sort)
fi

is_declared() {
    local aliases_csv="$1"
    local declared_lower alias
    IFS=',' read -r -a aliases <<< "$aliases_csv"
    for declared in "${DECLARED_VALUES[@]}"; do
        declared_lower="$(printf '%s' "$declared" | tr '[:upper:]' '[:lower:]')"
        for alias in "${aliases[@]}"; do
            alias="$(printf '%s' "$alias" | tr '[:upper:]' '[:lower:]')"
            if [[ "$declared_lower" == *"$alias"* ]]; then
                return 0
            fi
        done
    done
    return 1
}

declare -a REFERENCED=()
declare -a UNDECLARED=()

scan_technology() {
    local canonical="$1"
    local source_regex="$2"
    local declaration_aliases="$3"
    local file

    for file in "${INPUT_FILES[@]}"; do
        if LC_ALL=C grep -Eq "$source_regex" "$file"; then
            REFERENCED+=("$canonical")
            if ! is_declared "$declaration_aliases"; then
                UNDECLARED+=("$canonical")
            fi
            return
        fi
    done
}

# Catalog entries are recognition rules, not inferred project choices.
# Add aliases here when the repository adopts another explicitly named technology.
while IFS=$'\t' read -r canonical source_regex declaration_aliases; do
    [[ -z "$canonical" ]] && continue
    scan_technology "$canonical" "$source_regex" "$declaration_aliases"
done <<'CATALOG'
Rust	(^|[^[:alnum:]_])(Rust|rustc)([^[:alnum:]_]|$)	rust,rustc
Go	(^|[^[:alnum:]_])(Go [0-9]|Go Runtime|Golang|golang)([^[:alnum:]_]|$)	go,golang
TypeScript	(^|[^[:alnum:]_])(TypeScript|typescript|ts-node)([^[:alnum:]_]|$)	typescript,ts-node
JavaScript	(^|[^[:alnum:]_])(JavaScript|javascript)([^[:alnum:]_]|$)	javascript
Node.js	(^|[^[:alnum:]_])(Node\.js|nodejs|node |node$)([^[:alnum:]_]|$)	node.js,nodejs,node
Python	(^|[^[:alnum:]_])(Python|python[0-9.]*)($|[^[:alnum:]_])	python
Java	(^|[^[:alnum:]_])(Java|JDK|JVM)([^[:alnum:]_]|$)	java,jdk,jvm
Kotlin	(^|[^[:alnum:]_])(Kotlin|kotlin)([^[:alnum:]_]|$)	kotlin
Swift	(^|[^[:alnum:]_])(Swift|swiftc)([^[:alnum:]_]|$)	swift,swiftc
.NET	(^|[^[:alnum:]_])(\.NET|dotnet)([^[:alnum:]_]|$)	.net,dotnet
C#	(^|[^[:alnum:]_])C#([^[:alnum:]_]|$)	c#
C++	(^|[^[:alnum:]_])C\+\+([^[:alnum:]_]|$)	c++
Ruby	(^|[^[:alnum:]_])(Ruby|ruby)([^[:alnum:]_]|$)	ruby
PHP	(^|[^[:alnum:]_])(PHP|php)([^[:alnum:]_]|$)	php
Tokio	(^|[^[:alnum:]_])(Tokio|tokio)([^[:alnum:]_]|$)	tokio
serde	(^|[^[:alnum:]_])(serde|Serde)([^[:alnum:]_]|$)	serde
OpenTelemetry	(^|[^[:alnum:]_])(OpenTelemetry|opentelemetry|OTel)([^[:alnum:]_]|$)	opentelemetry,otel
HTTP	(^|[^[:alnum:]_])(HTTP|HTTPS|http://|https://)([^[:alnum:]_]|$)	http,https
gRPC	(^|[^[:alnum:]_])(gRPC|grpc)([^[:alnum:]_]|$)	grpc
GraphQL	(^|[^[:alnum:]_])(GraphQL|graphql)([^[:alnum:]_]|$)	graphql
Kafka	(^|[^[:alnum:]_])(Kafka|kafka)([^[:alnum:]_]|$)	kafka
RabbitMQ	(^|[^[:alnum:]_])(RabbitMQ|rabbitmq)([^[:alnum:]_]|$)	rabbitmq
AMQP	(^|[^[:alnum:]_])(AMQP|amqp)([^[:alnum:]_]|$)	amqp
PostgreSQL	(^|[^[:alnum:]_])(PostgreSQL|Postgres|postgresql|postgres)([^[:alnum:]_]|$)	postgresql,postgres
MySQL	(^|[^[:alnum:]_])(MySQL|mysql)([^[:alnum:]_]|$)	mysql
SQLite	(^|[^[:alnum:]_])(SQLite|sqlite)([^[:alnum:]_]|$)	sqlite
MongoDB	(^|[^[:alnum:]_])(MongoDB|mongodb)([^[:alnum:]_]|$)	mongodb
Redis	(^|[^[:alnum:]_])(Redis|redis)([^[:alnum:]_]|$)	redis
AWS	(^|[^[:alnum:]_])(AWS|Amazon Web Services)([^[:alnum:]_]|$)	aws,amazon web services
Azure	(^|[^[:alnum:]_])(Azure|Microsoft Azure)([^[:alnum:]_]|$)	azure,microsoft azure
Google Cloud	(^|[^[:alnum:]_])(Google Cloud|GCP)([^[:alnum:]_]|$)	google cloud,gcp
Docker	(^|[^[:alnum:]_])(Docker|docker)([^[:alnum:]_]|$)	docker
Kubernetes	(^|[^[:alnum:]_])(Kubernetes|kubernetes|kubectl)([^[:alnum:]_]|$)	kubernetes,kubectl
Jest	(^|[^[:alnum:]_])(Jest|jest)([^[:alnum:]_]|$)	jest
pytest	(^|[^[:alnum:]_])(pytest|Pytest)([^[:alnum:]_]|$)	pytest
cargo test	(^|[^[:alnum:]_])cargo test([^[:alnum:]_]|$)	cargo test,cargo
npm	(^|[^[:alnum:]_])npm([^[:alnum:]_]|$)	npm
pnpm	(^|[^[:alnum:]_])pnpm([^[:alnum:]_]|$)	pnpm
Yarn	(^|[^[:alnum:]_])(Yarn|yarn)([^[:alnum:]_]|$)	yarn
Maven	(^|[^[:alnum:]_])(Maven|mvn)([^[:alnum:]_]|$)	maven,mvn
Gradle	(^|[^[:alnum:]_])(Gradle|gradle)([^[:alnum:]_]|$)	gradle
Express	(^|[^[:alnum:]_])(Express\.js|ExpressJS|express)([^[:alnum:]_]|$)	express,express.js
FastAPI	(^|[^[:alnum:]_])(FastAPI|fastapi)([^[:alnum:]_]|$)	fastapi
React	(^|[^[:alnum:]_])(React|ReactJS|react)([^[:alnum:]_]|$)	react,reactjs
Vue	(^|[^[:alnum:]_])(Vue\.js|VueJS)([^[:alnum:]_]|$)	vue,vue.js,vuejs
Angular	(^|[^[:alnum:]_])(Angular|angular)([^[:alnum:]_]|$)	angular
CATALOG

unique_sorted() {
    if [[ $# -gt 0 ]]; then
        printf '%s\n' "$@" | LC_ALL=C sort -u
    fi
}

if [[ ${#UNDECLARED[@]} -gt 0 ]]; then
    first_undeclared="$(unique_sorted "${UNDECLARED[@]}" | head -n 1)"
    echo "TECHNOLOGY VIOLATION" >&2
    echo >&2
    echo "Undeclared technology detected:" >&2
    echo "$first_undeclared" >&2
    echo >&2
    echo "Declared technologies:" >&2
    unique_sorted "${DECLARED_VALUES[@]}" | sed 's/^/- /' >&2
    echo >&2
    echo "Referenced technologies:" >&2
    unique_sorted "${REFERENCED[@]}" | sed 's/^/- /' >&2
    echo >&2
    echo "No files may be written." >&2
    exit 1
fi

echo "Technology declaration validation: PASS"
echo "Phase: $PHASE"
echo "Specification: $FEATURE_DIR"
echo "Declared technologies:"
unique_sorted "${DECLARED_VALUES[@]}" | sed 's/^/- /'
echo "Referenced technologies:"
if [[ ${#REFERENCED[@]} -eq 0 ]]; then
    echo "- none"
else
    unique_sorted "${REFERENCED[@]}" | sed 's/^/- /'
fi
