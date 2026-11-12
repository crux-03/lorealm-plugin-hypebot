use std::collections::HashMap;

use lorealm_plugin::host::{generate, log, messages, register_hook, send_frontend, settings};
use lorealm_plugin::{
    ChatMessage, FrontendCall, Generated, Hook, Level, Message, MessageRange, Plugin, Result, Role,
    Value, json,
};

/// How much of the story HypeBot reads, from the end, in characters.
const READ: usize = 6000;

#[derive(Default)]
struct HypeBot {
    /// The last comment in each conversation
    last: HashMap<i64, String>,
}

impl Plugin for HypeBot {
    fn init() -> Result<Self> {
        register_hook(Hook::AfterGenerate)?;
        Ok(Self::default())
    }

    fn after_generate(&mut self, Generated { conversation, .. }: Generated) -> Result<()> {
        let settings: Value = settings()?;
        if settings["auto"] == true {
            send_frontend(
                "thinking",
                &json!({ "conversation": conversation.to_string() }),
            )?;
            let payload = match self.comment(conversation) {
                Ok(text) => json!({ "conversation": conversation.to_string(), "text": text }),
                Err(error) => {
                    log(Level::Warn, error.message());
                    json!({ "conversation": conversation.to_string(), "error": error.message() })
                }
            };
            send_frontend("comment", &payload)?;
        }
        Ok(())
    }

    fn frontend(&mut self, call: FrontendCall) -> Result<Value> {
        let Some(conversation) = call.conversation else {
            return Err("Open a chat first.".into());
        };
        Ok(match call.name.as_str() {
            "last" => json!(self.last.get(&conversation)),
            "comment" => json!(self.comment(conversation)?),
            other => return Err(format!("unknown call `{other}`").into()),
        })
    }
}

impl HypeBot {
    /// A fresh comment on the end of `conversation`.
    fn comment(&mut self, conversation: i64) -> Result<String> {
        let settings: Value = settings()?;
        // `null` asks the user's active profile.
        let profile = settings["profile"].as_str().and_then(|id| id.parse().ok());
        let story = transcript(&messages(conversation, MessageRange::Latest(8))?);
        if story.is_empty() {
            return Err("There's nothing to read yet.".into());
        }
        let reply = generate(
            &[
                Message {
                    role: Role::System,
                    content: settings["prompt"].as_str().unwrap_or_default().into(),
                },
                Message {
                    role: Role::User,
                    content: story,
                },
            ],
            profile,
        )?;
        let text = reply.trim().trim_matches('"').trim().to_owned();
        if text.is_empty() {
            return Err("HypeBot is speechless.".into());
        }
        self.last.insert(conversation, text.clone());
        Ok(text)
    }
}

/// The story so far as plain text, cut to its last `READ` characters.
fn transcript(messages: &[ChatMessage]) -> String {
    let story = messages
        .iter()
        .filter(|m| !matches!(m.role, Role::System))
        .map(|m| m.content.trim())
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let skip = story.chars().count().saturating_sub(READ);
    story.chars().skip(skip).collect()
}

lorealm_plugin::export!(HypeBot);
