mod error;

use async_openai::{
    self,
    types::responses::{
        Content, CreateResponseArgs, Input, InputItem, InputMessageArgs, OutputContent, Role,
    },
};
use error::Result;
use yt_transcript_rs::{FetchedTranscript, api::YouTubeTranscriptApi};

const SYSTEM_PROMPT: &str = r#"Eres un ANALISTA DE CONTENIDO ULTRA-DETALLISTA"#;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    // This creates a new instance without proxy or cookie authentication
    let api: YouTubeTranscriptApi = YouTubeTranscriptApi::new(None, None, None)?;

    // https://youtu.be/wXVvfFMTyzY?feature=shared
    // https://youtu.be/5_EJwYeQusM?feature=shared
    let video_id: &'static str = "5_EJwYeQusM";

    // Language preference (English)
    let languages: &[&'static str; 2] = &["en", "es"];

    // Don't preserve formatting (remove line breaks, etc.)
    let preserve_formatting: bool = true;

    let transcript: FetchedTranscript = get_transcript(&api, video_id, languages, preserve_formatting).await?;
    // println!("{:?}", transcript);

    generate_report(Some(transcript)).await?;
    // generate_report(None).await?;
    Ok(())
}

async fn get_transcript(
    api: &YouTubeTranscriptApi,
    video_id: &str,
    languages: &[&str],
    preserve_formatting: bool,
) -> Result<FetchedTranscript> {
    // Fetch the transcript
    println!("Fetching transcript for video ID: {}", video_id);

    match api
        .fetch_transcript(video_id, languages, preserve_formatting)
        .await
    {
        Ok(transcript) => {
            println!("Successfully fetched transcript!");
            println!("Video ID: {}", transcript.video_id);
            println!(
                "Language: {} ({})",
                transcript.language, transcript.language_code
            );
            println!("Is auto-generated: {}", transcript.is_generated);
            println!("Number of snippets: {}", transcript.snippets.len());
            println!("\nTranscript content:");

            let file_name = format!("transcript_{}.txt", transcript.video_id);
            std::fs::write(file_name, transcript.text()).expect("Unable to write file");
            return Ok(transcript);
        }
        Err(e) => {
            return Err(error::Error::custom(format!(
                "Failed to fetch transcript: {}",
                e
            )));
        }
    }
}

async fn generate_report(transcript: Option<FetchedTranscript>) -> Result<()> {
    // Generate a report from the transcript

    let (transcript, video_id) = match transcript {
        Some(t) => (t.text(), t.video_id),
        None => {
            let video_id = "S6w-UEOK7aI"; // Default video ID if no transcript is provided
            // If no transcript is provided, read from the file
            let file_name = format!("transcript_{}.txt", video_id);
            let content = std::fs::read_to_string(file_name).expect("Unable to read file");
            (content, video_id.to_string())
        }
    };

    println!("\nGenerating report...");
    let client = async_openai::Client::new();
    let request = CreateResponseArgs::default()
        .max_output_tokens(32768_u32)
        .model("gpt-4.1-mini")
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
{:?}
</TRANSCRIPT>
", transcript))
    .build()?,
                )]))
        .build()?;

    let response = client.responses().create(request).await?;

    for output in response.output {
        if let OutputContent::Message(out) = output {
            //save the response to a file .md
            let file_name = format!("report_{}.md", video_id);
            let mut content = String::new();
            for c in out.content {
                match c {
                    Content::OutputText(text) => content.push_str(&text.text),
                    _ => {
                        eprintln!("Unexpected content type: {:?}", c);
                        continue;
                    }
                }
            }

            std::fs::write(file_name, content).expect("Unable to write file");
        }
        // println!("{:?}", choice.message.content);
    }

    Ok(())
}