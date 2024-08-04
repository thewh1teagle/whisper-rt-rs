use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

pub struct Llm {
    ollama: Ollama
}

impl Llm {
    pub fn new() -> Self {
        let ollama = Ollama::default();
        Self {ollama}
    }

    pub fn ask(&mut self, prompt: String) -> String {
        let model = "qwen2:1.5b-instruct-q4_0".to_string();        
        // Create a Tokio runtime
        let runtime = tokio::runtime::Runtime::new().unwrap();
    
        // Run the async code within the runtime
        let res = runtime.block_on(async {
            self.ollama.generate(GenerationRequest::new(model, prompt).system("Provide short answers in single sentence only".into())).await
        }).unwrap();
        res.response
    }
}