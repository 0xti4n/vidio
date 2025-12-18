use crate::error::{Error, Result};
use async_openai::{
    self,
    types::responses::{
        CreateResponseArgs, EasyInputMessageArgs, InputItem, InputParam, OutputItem,
        OutputMessageContent, ReasoningArgs, ReasoningEffort, Role,
    },
};

use std::env;
use yt_transcript_rs::FetchedTranscript;

const SYSTEM_PROMPT: &str = r#"Eres un ANALISTA DE CONTENIDO ULTRA-DETALLISTA"#;
const OPENAI_OPT_IN_ENV: &str = "YTRANSCRIPT_ALLOW_OPENAI";

#[derive(Clone)]
pub struct ReportService {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
}

impl ReportService {
    pub fn new() -> Self {
        Self {
            client: async_openai::Client::new(),
        }
    }

    pub async fn generate_report(&self, transcript: &FetchedTranscript) -> Result<String> {
        let formatted = crate::core::transcript::TranscriptService::format_transcript(transcript);
        let formatted_text = formatted.join("\n");
        self.generate_report_text(&formatted_text).await
    }

    pub async fn generate_report_text(&self, transcript_text: &str) -> Result<String> {
        enforce_openai_opt_in()?;

        let request = CreateResponseArgs::default()
            .max_output_tokens(128000_u32)
            .model("gpt-5.2")
            .reasoning(ReasoningArgs::default()
                .effort(ReasoningEffort::High)
                // .summary(ReasoningSummary::Detailed)
                .build()?
            )
            .input(InputParam::Items(vec![
                InputItem::EasyMessage(
                    EasyInputMessageArgs::default()
                        .role(Role::System)
                        .content(SYSTEM_PROMPT)
                        .build()?,
                ),
                InputItem::EasyMessage(
                    EasyInputMessageArgs::default()
                        .role(Role::User)
                        .content(format!(
                            "### rol
Tu misión: extraer **cada** elemento significativo del vídeo sin omitir nada, con precisión milimétrica.

### Entrada
A continuación recibirás la transcripción completa entre las marcas <TRANSCRIPT> … </TRANSCRIPT>.
No añadas contexto externo: todo debe provenir del texto entregado.

### Reglas de extracción
1. **Cero resúmenes.** No sintetices. Incluye cada idea tal como aparece.
2. Mantén el **orden cronológico** original.
3. Si el transcript incluye marcas de tiempo, consérvalas; si no, indica \"⏱ n/a\".
4. Preserva las citas literales relevantes (\"texto exacto\").
5. No añadas opiniones ni interpretación subjetiva.

### Formato de salida
Devuelve un reporte en Markdown con los siguientes bloques:

#### 1. Metadata
| Campo | Valor |
|-------|-------|
| Duración aproximada | X min |
| Número de líneas | N |
| Idioma predominante | … |
| Voz principal (si se infiere) | … |
| Otros participantes | … |

#### 2. Índice cronológico de secciones
Lista cada cambio de tema o segmento importante tal como se detecte en la transcripción.
*Ejemplo*:
- 00:00 - 01:42 Introducción del canal
- 01:43 - 05:20 Historia personal sobre productividad

#### 3. Desglose línea por línea
| # | ⏱ | Orador* | Texto literal | Palabras clave | Tonalidad** |
|---|----|---------|---------------|----------------|-------------|
| 1 | 00:00 | Host | \"Bienvenidos…\" | bienvenidos, canal | amigable |
| 2 | 00:08 | Host | … | … | … |

* Si no hay speaker tags, usa \"Unk\".
** Tonalidad: informativo, persuasivo, anecdótico, humor, etc.

#### 4. Entidades y conceptos mencionados
| Entidad | Tipo (persona, marca, lugar…) | Nº de menciones | Primera mención ⏱ |
|---------|------------------------------|-----------------|-------------------|

#### 5. Preguntas planteadas
Lista literal de todas las preguntas que formula el orador, con su timestamp.

#### 6. Citas \"clave\" (≥ 15 palabras)
Incluye cada cita textual larga; útil para captions o destacados.

#### 7. Llamados a la acción (CTA)
Cada vez que se invita al espectador a suscribirse, comentar, comprar, etc., con su timestamp y texto exacto.

#### 8. Recursos externos
Links, referencias a libros, cursos, herramientas, etc. (solo si aparecen en la transcripción).

#### 9. Estructura retórica
- **Hook inicial**: ⏱ …
- **Conflicto / Problema expuesto**: ⏱ …
- **Solución / Clímax**: ⏱ …
- **Cierre**: ⏱ …

#### 10. Lista completa de palabras clave (frecuencia ≥ 2)
Ordenadas por frecuencia descendente.

#### 11. resumen ejecutivo detallado de todo el contenido, sin omitir nada.
---

### Ejecución
Analiza ahora el contenido entre las etiquetas:

<TRANSCRIPT>
{}
</TRANSCRIPT>
",
                            transcript_text
                        ))
                        .build()?,
                ),
            ]))
            .build()?;

        let response = self.client.responses().create(request).await?;

        let mut content = String::new();
        for output in response.output {
            if let OutputItem::Message(out) = output {
                for c in out.content {
                    match c {
                        OutputMessageContent::OutputText(text) => content.push_str(&text.text),
                        _ => {
                            eprintln!("Unexpected content type: {c:?}");
                            continue;
                        }
                    }
                }
            }
        }

        Ok(ensure_table_headers(&content))
    }
}

fn enforce_openai_opt_in() -> Result<()> {
    match env::var(OPENAI_OPT_IN_ENV) {
        Ok(val)
            if matches!(
                val.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            ) =>
        {
            Ok(())
        }
        _ => Err(Error::custom(format!(
            "Report generation requires explicit opt-in. Set {OPENAI_OPT_IN_ENV}=1 to enable uploads to OpenAI."
        ))),
    }
}

struct TableTemplate {
    section_marker: &'static str,
    header_lines: &'static [&'static str],
    header_keywords: &'static [&'static str],
}

fn ensure_table_headers(report: &str) -> String {
    const TABLES: &[TableTemplate] = &[
        TableTemplate {
            section_marker: "#### 1. Metadata",
            header_lines: &["| Campo | Valor |", "|-------|-------|"],
            header_keywords: &["campo", "valor"],
        },
        TableTemplate {
            section_marker: "#### 3. Desglose",
            header_lines: &[
                "| # | ⏱ | Orador* | Texto literal | Palabras clave | Tonalidad** |",
                "|---|----|---------|---------------|----------------|-------------|",
            ],
            header_keywords: &["#", "⏱", "orador", "texto", "palabras", "tonalidad"],
        },
        TableTemplate {
            section_marker: "#### 4. Entidades",
            header_lines: &[
                "| Entidad | Tipo (persona, marca, lugar…) | Nº de menciones | Primera mención ⏱ |",
                "|---------|------------------------------|-----------------|-------------------|",
            ],
            header_keywords: &["entidad", "tipo", "mención"],
        },
        TableTemplate {
            section_marker: "#### 5. Preguntas",
            header_lines: &["| Pregunta | Timestamp |", "|----------|-----------|"],
            header_keywords: &["pregunta", "timestamp"],
        },
    ];

    let mut lines: Vec<String> = report.lines().map(|l| l.to_string()).collect();

    for table in TABLES {
        if let Some(section_idx) = lines
            .iter()
            .position(|line| line.trim_start().starts_with(table.section_marker))
        {
            let mut insert_idx = section_idx + 1;
            while insert_idx < lines.len() && lines[insert_idx].trim().is_empty() {
                insert_idx += 1;
            }

            let header_present =
                if insert_idx < lines.len() && lines[insert_idx].trim_start().starts_with('|') {
                    let normalized = lines[insert_idx].to_lowercase();
                    table
                        .header_keywords
                        .iter()
                        .all(|kw| normalized.contains(kw))
                } else {
                    false
                };

            if !header_present {
                let insert_lines: Vec<String> =
                    table.header_lines.iter().map(|s| s.to_string()).collect();
                lines.splice(insert_idx..insert_idx, insert_lines);
            }
        }
    }

    lines.join("\n")
}
