# Feature Specification: Console Exporter for Kit Logger

**Feature Branch**: `009-kit-009-console`  
**Created**: 2026-06-10  
**Status**: Draft  
**Input**: User description: "KIT-009 — Console Exporter — Proveer un exporter de consola para Kit Logger que permita emitir logs estructurados a stdout o stderr, optimizado para: Desarrollo local, Debugging, Containers (Docker), Kubernetes, CI/CD pipelines, Integración con sistemas de recolección de logs (Fluent Bit, Vector, Loki, Datadog, etc.). Debe ser totalmente desacoplado del resto de exporters y cumplir con las interfaces definidas en las specs anteriores."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Console Logging for Local Development (Priority: P1)

Como desarrollador, quiero visualizar logs en la consola durante el desarrollo local para poder diagnosticar y monitorear aplicaciones sin necesidad de configurar exporters externos.

**Why this priority**: Es el caso de uso fundamental y más común. Todo desarrollador necesita ver logs en su terminal durante el desarrollo.

**Independent Test**: Puede probarse independientemente ejecutando la aplicación con el exporter de consola y verificando que los logs aparecen en stdout/stderr con el formato seleccionado.

**Acceptance Scenarios**:

1. **Given** una aplicación configurada con ConsoleExporter, **When** se emite un log con nivel INFO, **Then** el mensaje aparece en la consola configurada (stdout o stderr) con el formato especificado.
2. **Given** ConsoleFormat::Json, **When** se emite un log con campos estructurados, **Then** la salida es un JSON válido con todos los campos incluyendo los personalizados.
3. **Given** ConsoleFormat::Pretty, **When** se emite un log, **Then** la salida es legible para humanos con timestamp, nivel y mensaje visibles.
4. **Given** configuración de nivel WARN, **When** se emite un log con nivel DEBUG, **Then** el evento es ignorado y no aparece en la consola.

---

### User Story 2 - Container and Container Orchestrator Logging (Priority: P1)

Como operador de containers, quiero que los logs se emitan a stdout/stderr en formato estructurado para que sean correctamente recolectados por sistemas como Fluent Bit, Vector, Loki o Datadog en entornos Docker, Kubernetes y CI/CD.

**Why this priority**: La integración con sistemas de recolección de logs es esencial para aplicaciones en producción en entornos containerizados.

**Independent Test**: Puede probarse ejecutaando la aplicación en un container y verificando que los logs son recolectados correctamente por el sistema de logs del orchestrator.

**Acceptance Scenarios**:

1. **Given** una aplicación corriendo en Kubernetes, **When** se emite un log estructurado en formato JSON, **Then** los logs son recolectados por el agente de logs del cluster y pueden consultarse en la interfaz del sistema de monitoreo.
2. **Given** stdout configurado como target, **When** los logs se emite desde múltiples hilos simultáneos, **Then** no hay interleaving de líneas y cada log aparece completo.
3. **Given** aplicación en shutdown, **When** se destruir el exporter, **Then** se realiza flush de la cola y los mensajes pendientes se escriben antes de cerrar.

---

### User Story 3 - Non-Blocking High-Throughput Logging (Priority: P2)

Como desarrollador necesito que el logging no bloquee el hilo principal de la aplicación para mantener alto rendimiento incluso con altos volúmenes de logs.

**Why this priority**: En aplicaciones de alto throughput, el logging sincrónico puede convertirse en un cuello de botella. El modo async permite continuar la ejecución sin esperar la escritura.

**Independent Test**: Puede probarse emitiendo 100,000+ eventos por segundo y midiendo que el tiempo de la aplicación no aumenta significativamente.

**Acceptance Scenarios**:

1. **Given** ConsoleMode::Async con queue_capacity=8192, **When** se emite un volumen alto de logs, **Then** los logs se encolan y un worker thread los escribe asincrónicamente.
2. **Given** OverflowPolicy::DropOldest y cola llena, **When** llega un nuevo evento, **Then** el evento más antiguo se descarta para hacer espacio.
3. **Given** OverflowPolicy::DropNewest y cola llena, **When** llega un nuevo evento, **Then** el nuevo evento se descarta.
4. **Given** OverflowPolicy::Block y cola llena, **When** llega un nuevo evento, **Then** el hilo se bloquea hasta que haya espacio.

---

### User Story 4 - Human-Readable Console Output with Colors (Priority: P3)

Como desarrollador Quiero salida.colorida en modo Pretty para distinguir rápidamente los niveles de log durante debugging.

**Why this priority**: Los colores facilitan la identificación visual rápida de niveles de log en desarrollo local.

**Independent Test**: Puede probarse ejecutando la aplicación en una terminal que soporte colores y verificando que los niveles tienen los colores correctos.

**Acceptance Scenarios**:

1. **Given** enable_colors=true y terminal compatible, **When** se emite un log de nivel ERROR, **Then** el mensaje aparece en rojo.
2. **Given** variable de entorno NO_COLOR=1, **When** se configura enable_colors=true, **Then** los colores se desactivan automáticamente.
3. **Given** terminal que no soporta colores, **When** se configura enable_colors=true, **Then** los colores se detectan y desactivan automáticamente.

---

### Edge Cases

- ¿Qué sucede cuando el evento es mayor a 256KB? Debe soportarse sin truncamiento.
- ¿Cómo se manejan caracteres UTF-8 especiales (emojis, español, chino, japonés, coreano)?
- ¿Qué sucede cuando elthread worker de async mode falla? Los errores deben manejarse y registrarse sin panic.
- ¿Qué sucede cuando el sistema de archivos no está disponible para stderr (en containeres)?
- ¿Cómo afectan las condiciones de carrera en métricas cuando múltiples hilos escriben simultáneamente?

## Requirements _(mandatory)_

### Functional Requirements

Basados en los requisitos del usuario:

- **FR-001**: El exporter debe soportar ConsoleTarget::Stdout y ConsoleTarget::Stderr como destinos de salida.
- **FR-002**: El exporter debe soportar ConsoleFormat::Json (salida estructurada JSON) y ConsoleFormat::Pretty (salida legible para humanos).
- **FR-003**: El exporter debe imprimir todos los campos estructurados (fields) del LogEvent en la salida.
- **FR-004**: El exporter debe respetar la configuración global de TimestampFormat (Iso8601, Unix, UnixMillis).
- **FR-005**: El exporter no debe emitir eventos por debajo del nivel de log configurado.
- **FR-006**: Múltiples hilos deben poder escribir simultáneamente sin líneas corruptas o interleaving.
- **FR-007**: El exporter debe exponer flush() para garantizar que los mensajes pendientes se escriban.
- **FR-008**: El exporter debe soportar ConsoleMode::Blocking y ConsoleMode::Async.
- **FR-009**: El modo async debe tener queue_capacity configurable (default: 8192).
- **FR-010**: El exporter debe soportar OverflowPolicy::Block, DropNewest, DropOldest.
- **FR-011**: El exporter debe exponer métricas: events_written, events_dropped, write_errors, queue_depth.
- **FR-012**: Errores de escritura no deben provocar panic; deben incrementa métricas y registrarse internamente.
- **FR-013**: Modo Pretty debe soportar enable_colors con mapeo: TRACE= Gray, DEBUG= Blue, INFO= Green, WARN= Yellow, ERROR= Red.
- **FR-014**: Debe detectar NO_COLOR=1 o terminal no compatible y desactivar colores automáticamente.
- **FR-015**: Campos especiales (request_id, trace_id, span_id, tenant_id) deben mostrarse al inicio de la línea en modo Pretty.
- **FR-016**: Debe soportar eventos de al menos 256KB sin truncamiento.
- **FR-017**: Debe soportar UTF-8 multilenguaje (español, chino, japonés, coreano, emojis).
- **FR-018**: Al hacer drop del exporter, debe: detener workers, vaciar cola, ejecutar flush() (graceful shutdown).
- **FR-019**: Debe existir una estructura ConsoleExporterConfig con: target, format, mode, queue_capacity, overflow_policy, enable_colors.
- **FR-020**: Cada LogEvent debe producir exactamente una línea de salida (un newline al final). En JSON, el objeto va seguido de newline. En Pretty, la línea completa va seguida de newline. Esto es crítico para integración con Fluent Bit, Vector, Loki, Datadog y Kubernetes.
- **FR-021**: El orden de campos en la salida JSON debe ser determinístico para facilitar snapshots y golden tests. Orden: timestamp, level, message, request_id, trace_id, span_id, tenant_id, correlation_id, luego campos custom ordenados lexicográficamente.

### Non-Functional Requirements

- **NFR-001**: Modo JSON no debe realizar serializaciones redundantes (allocation efficient).
- **NFR-002**: Throughput objetivo: ≥ 100,000 eventos/segundo en hardware moderno.
- **NFR-003**: Zero Panic: no debe haber panic en runtime.
- **NFR-004**: Memory Bounded: modo async debe respetar queue_capacity sin crecimiento ilimitado.
- **NFR-005**: Platform Support: debe ser compatible con Linux, macOS y Windows.

### Key Entities

- **ConsoleExporterConfig**: Estructura de configuración con todos los parámetros ajustables.
- **ConsoleTarget**: Enum para seleccionar stdout o stderr.
- **ConsoleFormat**: Enum para seleccionar Json o Pretty.
- **ConsoleMode**: Enum para seleccionar Blocking o Async.
- **OverflowPolicy**: Enum para política de desbordamiento de cola.
- **ConsoleMetrics**: Estructura que exponer las métricas del exporter.
- **ConsoleWriter**: Componente thread-safe para escribir a la consola.

## Clarifications & Design Decisions

Los siguientes gaps fueron identificados durante la revisión de la especificación y las decisiones de diseño tomadas:

### Gap 1 — Comportamiento exacto de flush() en modo Async

**Pregunta**: ¿flush() espera solo a vaciar la cola o también a que el writer confirme la escritura física?

**Decisión**: flush() bloquea hasta que todos los eventos presentes en la cola al momento de la llamada hayan sido escritos por el worker. No retorna hasta que el worker confirma la escritura física.

### Gap 2 — ConsoleWriter Abstraction

**Decisión**: Se introduce una abstracción para facilitar testing:

```
pub trait ConsoleWriter: Send + Sync {
    fn write(&self, bytes: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
}
```

Implementaciones:
- `StdoutWriter`: escribe a stdout
- `StderrWriter`: escribe a stderr
- `TestWriter`: mock para tests

### Gap 3 — Métricas Thread-Safe

**Decisión**: Todas las métricas serán lock-free usando AtomicU64/AtomicUsize:

```
events_written  -> AtomicU64
events_dropped  -> AtomicU64
write_errors    -> AtomicU64
queue_depth     -> AtomicUsize (loaded en el momento de consulta)
```

### Gap 4 — Detección de Terminal con Colores

**Decisión**: La lógica final de colores será:

```
enable_colors_final = config.enable_colors && !no_env_set && is_tty
```

Donde:
- `config.enable_colors` es el valor booleano de configuración
- `no_env_set` es true cuando la variable de entorno NO_COLOR=1 está presente
- `is_tty` es true cuando el target (stdout/stderr) está conectado a una terminal

### Gap 5 — Shutdown Timeout

**Decisión**: El graceful shutdown tendrá un timeout de 30 segundos. Pasado ese tiempo:
- Se registra un error de timeout
- Se fuerza la terminación del worker
- Nunca se ejecuta panic

### Gap 6 — Formato JSON Estable

**Decisión**: Se definen los campos del JSON para garantizar estabilidad:

**Campos obligatorios** (siempre presentes):
- `timestamp` - marca de tiempo formateada según TimestampFormat
- `level` - nivel del log
- `message` - mensaje del log

**Campos opcionales** (presentes si existen):
- `request_id` - ID de request
- `trace_id` - ID de traza
- `span_id` - ID de span
- `tenant_id` - ID de tenant
- `correlation_id` - ID de correlación

**Campos custom**:
- Todos los campos estructurados del LogEvent (fields)

### Arquitectura Técnica Recomendada

La implementación recomendada sigue esta estructura modular:

```
ConsoleExporter
├── ConsoleWriter (trait)
│     ├── StdoutWriter
│     ├── StderrWriter
│     └── TestWriter
├── Formatter
│     ├── JsonFormatter (con FR-020 y FR-021)
│     └── PrettyFormatter
├── Sync Mode (escritura directa)
└── Async Mode
      ├── bounded queue (mpsc o custom)
      ├── worker thread
      └── flush barrier (sincronización)
```

**Principios de diseño**:
- **Desacoplamiento**: El exporter no depende del resto de exporters
- **Testabilidad**: ConsoleWriter trait permite mock en tests
- **Composabilidad**: Formatter es reutilizable
- **Thread-safety**: Async mode con colas limitadas y worker dedicado

Esta arquitectura es compatible con las specs previas KIT-001 a KIT-006 sin introducir acoplamientos extras.

---

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Los desarrolladores pueden completar la configuración del exporter de consola en menos de 5 minutos siguiendo la guía de inicio rápido.
- **SC-002**: El sistema maneja 100,000 eventos por segundo sin pérdida de datos en modo Blocking en hardware moderno.
- **SC-003**: El modo Async con cola llena y OverflowPolicy::DropOldest descarta eventos antiguos y acepta nuevos sin bloquear el hilo principal.
- **SC-004**: El exporter funciona correctamente en Linux, macOS y Windows sin cambios de código.
- **SC-005**: El shutdown graceful garantiza que menos del 1% de los eventos se pierden cuando se cierra la aplicación con eventos pendientes en la cola.
- **SC-006**: La salida JSON es parseable correctamente por sistemas de recolección de logs estándar.
- **SC-007**: Los colores se muestran correctamente en terminals que lo soportan y se desactivan cuando NO_COLOR=1 o el terminal no es compatible.
