use crate::error::Result;
use async_openai::{
    self,
    types::{
        ReasoningEffort,
        responses::{
            Content, CreateResponseArgs, Input, InputItem, InputMessageArgs, OutputContent,
            ReasoningConfigArgs, Role,
        },
    },
};

use yt_transcript_rs::FetchedTranscript;

const SYSTEM_PROMPT: &str = r#"Eres un ANALISTA DE CONTENIDO ULTRA-DETALLISTA"#;

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
        self.generate_report_text(&transcript.text()).await
    }

    pub async fn generate_report_text(&self, transcript_text: &str) -> Result<String> {
        let request = CreateResponseArgs::default()
            .max_output_tokens(128000_u32)
            .model("gpt-5")
            .reasoning(ReasoningConfigArgs::default()
                .effort(ReasoningEffort::High)
                // .summary(ReasoningSummary::Detailed)
                .build()?
            )
            .input(Input::Items(vec![
                InputItem::Message(
                    InputMessageArgs::default()
                        .role(Role::System)
                        .content(SYSTEM_PROMPT.to_string())
                        .build()?,
                ),
                InputItem::Message(
                    InputMessageArgs::default()
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
            if let OutputContent::Message(out) = output {
                for c in out.content {
                    match c {
                        Content::OutputText(text) => content.push_str(&text.text),
                        _ => {
                            eprintln!("Unexpected content type: {c:?}");
                            continue;
                        }
                    }
                }
            }
        }

        Ok(content)
    }
}
