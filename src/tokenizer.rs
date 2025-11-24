

struct TokenizerConfig {

}


pub struct Tokenizer {
    config : TokenizerConfig,
}


impl Tokenizer {
    pub fn new(_config_path: &str) -> Self {

        let config = TokenizerConfig {};


        Self {
            config
        }
    }


    pub fn encode(&self, _text: &str) -> Vec<u32> {


        vec![1, 2, 3]
    }


    pub fn decode(&self, _tokens : &[u32]) -> String {


        "Hello, world".to_string()
    }
}