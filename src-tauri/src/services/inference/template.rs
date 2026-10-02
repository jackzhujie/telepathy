use super::Message;

#[derive(Debug, Clone, PartialEq)]
pub enum ChatTemplateType {
    ChatML,
    Gemma,
    Llama3,
    Phi3,
    // 预留给未来的 Jinja 自定义渲染，或者其它通用模板引擎
    // Jinja(String),
}

impl ChatTemplateType {
    /// 启发式：根据模型名称或者路径推断应该使用的通用对话模板类型。
    /// 当后续接入真正的 Jinja 或者从 GGUF 内部读取时，可进一步扩展此逻辑。
    pub fn detect_from_model_path(path: &str) -> Self {
        let name = path.to_lowercase();
        if name.contains("gemma") {
            Self::Gemma
        } else if name.contains("llama3") || name.contains("llama-3") {
            Self::Llama3
        } else if name.contains("phi") {
            Self::Phi3
        } else {
            Self::ChatML
        }
    }

    /// 应用对应模型的专属模板规则，将一组通用消息渲染为最终用于推理的文本。
    pub fn apply(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();
        match self {
            Self::Gemma => {
                // Gemma 格式
                for msg in messages {
                    let role = match msg.role.as_str() {
                        "assistant" => "model",
                        "system" => "user", // Gemma typically merges system into user
                        _ => "user",
                    };
                    prompt.push_str(&format!(
                        "<start_of_turn>{}\n{}<end_of_turn>\n",
                        role, msg.content
                    ));
                }
                prompt.push_str("<start_of_turn>model\n");
            }
            Self::Llama3 => {
                // Llama-3 格式
                for msg in messages {
                    let role = match msg.role.as_str() {
                        "system" => "system",
                        "user" => "user",
                        "assistant" => "assistant",
                        _ => "user",
                    };
                    prompt.push_str(&format!(
                        "<|start_header_id|>{role}<|end_header_id|>\n\n{content}<|eot_id|>\n",
                        role = role,
                        content = msg.content
                    ));
                }
                prompt.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
            }
            Self::Phi3 => {
                // Phi-3 格式
                for msg in messages {
                    let role = match msg.role.as_str() {
                        "system" => "system",
                        "user" => "user",
                        "assistant" => "assistant",
                        _ => "user",
                    };
                    prompt.push_str(&format!(
                        "<|{}|>\n{}<|end|>\n",
                        role, msg.content
                    ));
                }
                prompt.push_str("<|assistant|>\n");
            }
            Self::ChatML => {
                // ChatML 格式 (适用于 Qwen, Yi, DeepSeek 等)
                for msg in messages {
                    let role = match msg.role.as_str() {
                        "system" => "system",
                        "user" => "user",
                        "assistant" => "assistant",
                        _ => "user",
                    };
                    prompt.push_str(&format!(
                        "<|im_start|>{}\n{}<|im_end|>\n",
                        role, msg.content
                    ));
                }
                prompt.push_str("<|im_start|>assistant\n");
            }
        }
        prompt
    }
}
