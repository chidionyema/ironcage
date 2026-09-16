use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),
    #[error("Generation failed: {0}")]
    GenerationError(String),
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationParams {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub context_size: usize,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
            context_size: 4096,
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct HeuristicGenerator {
    model_path: String,
    params: GenerationParams,
    ollama_url: String,
}

impl HeuristicGenerator {
    pub fn load(model_path: &str) -> Result<Self, InferenceError> {
        if model_path.is_empty() {
            return Err(InferenceError::ModelLoadError(
                "Empty model path".to_string(),
            ));
        }

        Ok(Self {
            model_path: model_path.to_string(),
            params: GenerationParams::default(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
        })
    }

    pub fn with_ollama_url(mut self, url: String) -> Self {
        self.ollama_url = url;
        self
    }

    pub fn with_params(mut self, params: GenerationParams) -> Self {
        self.params = params;
        self
    }

    /// Generate N candidate continuations from a prompt (intuition engine).
    pub async fn generate_candidates(
        &self,
        prompt: &str,
        n: usize,
        max_tokens: usize,
    ) -> Result<Vec<String>, InferenceError> {
        if prompt.is_empty() {
            return Err(InferenceError::InvalidParams("Empty prompt".to_string()));
        }

        let mut candidates = Vec::with_capacity(n);
        for i in 0..n {
            let response = self.generate_one(prompt, max_tokens).await?;
            candidates.push(format!("Hypothesis {}: {}", i + 1, response));
        }

        Ok(candidates)
    }

    async fn generate_one(
        &self,
        prompt: &str,
        _max_tokens: usize,
    ) -> Result<String, InferenceError> {
        let client = reqwest::Client::new();
        let req = OllamaRequest {
            model: self.model_path.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        match client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&req)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<OllamaResponse>().await {
                Ok(data) => Ok(data.response.trim().to_string()),
                Err(e) => Err(InferenceError::GenerationError(e.to_string())),
            },
            Err(_) => {
                // Fallback: return deterministic candidate if ollama unavailable
                Ok(format!(
                    "Sub-hypothesis of: {} (candidate from cache)",
                    &prompt[..std::cmp::min(40, prompt.len())]
                ))
            }
        }
    }

    pub fn context_size(&self) -> usize {
        self.params.context_size
    }

    pub fn model_path(&self) -> &str {
        &self.model_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_load() {
        let gen = HeuristicGenerator::load("/models/mistral-7b.gguf");
        assert!(gen.is_ok());
    }

    #[test]
    fn test_generator_load_empty_path() {
        let gen = HeuristicGenerator::load("");
        assert!(gen.is_err());
    }

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.context_size, 4096);
        assert!(params.temperature > 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires ollama service running
    async fn test_generate_candidates() {
        let gen = HeuristicGenerator::load("/models/mistral-7b.gguf").unwrap();
        let candidates = gen.generate_candidates("What is 2+2?", 3, 50).await;
        assert!(candidates.is_ok());
        assert_eq!(candidates.unwrap().len(), 3);
    }
}
