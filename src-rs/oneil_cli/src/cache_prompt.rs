//! Interactive stdin prompts for Python call cache policy.

use std::io::{self, Write};

use anstream::eprint;
use oneil_runtime::{CachePromptContext, CachePromptKind, CachePrompter};

use crate::stylesheet;

/// Prompts on stderr and reads yes/no answers from stdin.
#[derive(Debug, Clone, Copy, Default)]
pub struct CliCachePrompter;

impl CachePrompter for CliCachePrompter {
    fn prompt(&self, kind: CachePromptKind, context: &CachePromptContext) -> bool {
        let message = prompt_message(kind, context);
        let message_styled = stylesheet::CACHE_PROMPT_MESSAGE.style(message);

        let yn_styled = stylesheet::CACHE_PROMPT_YN.style("[y/N]");
        eprint!("{message_styled} {yn_styled} ");

        let _ = io::stderr().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return false;
        }

        matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
    }
}

fn prompt_message(kind: CachePromptKind, context: &CachePromptContext) -> String {
    let path = context.python_path.as_path().display();

    match kind {
        CachePromptKind::UseStaleCacheOnRead => {
            let function = context
                .function_name
                .as_ref()
                .map(|name| format!("::{}", name.as_str()))
                .unwrap_or_default();
            format!("use outdated cached result for `{path}{function}`?")
        }
        CachePromptKind::OverwriteCacheOnHashMismatch => {
            format!("`{path}` has changed. clear outdated cache?")
        }
        CachePromptKind::OverwriteCacheOnOutputMismatch => {
            let function = context
                .function_name
                .as_ref()
                .map(|name| format!("::{}", name.as_str()))
                .unwrap_or_default();
            format!("update cached result for `{path}{function}`?")
        }
    }
}
