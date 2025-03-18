use ollama_rs::error::OllamaError;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::parameters::{self, JsonStructure};
use ollama_rs::Ollama;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use snafu::{Location, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum LLMError {
    #[snafu(display("Failed to run Ollama"))]
    Ollama { source: OllamaError },

    #[snafu(display("Serde json error at {loc}"))]
    #[snafu(context(false))]
    SerdeJson {
        source: serde_json::Error,
        #[snafu(implicit)]
        loc: Location,
    },
}

#[allow(clippy::upper_case_acronyms)]
pub struct LLM {
    ollama: Ollama,
    model: String,
}

impl LLM {
    pub fn new() -> Self {
        let ollama = Ollama::default();
        let model = "llama3:latest".to_string();
        LLM { ollama, model }
    }

    pub async fn generate<T: JsonSchema + DeserializeOwned>(
        &self,
        prompt: &str,
    ) -> Result<T, LLMError> {
        let json_structure = JsonStructure::new::<T>();
        let format = parameters::FormatType::StructuredJson(json_structure);

        let res = self
            .ollama
            .generate(GenerationRequest::new(self.model.to_string(), prompt).format(format))
            .await
            .context(OllamaSnafu)?;

        let notes: T = serde_json::from_str(&res.response)?;
        Ok(notes)
    }
}
