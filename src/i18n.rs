use anyhow::{Context, Result};
use fluent::{FluentArgs, FluentBundle, FluentResource};
use std::collections::HashMap;
use std::env;
use unic_langid::LanguageIdentifier;

/// A type alias for translation arguments.
pub type Args<'a> = &'a [(&'a str, &'a str)];

/// Handles internationalization using the Fluent localization system.
///
/// This struct manages translations for the application, providing:
/// - Automatic locale detection from environment variables
/// - Fallback to English (en-US) for unsupported locales
/// - Thread-local storage for efficient access
/// - Type-safe message formatting with arguments
///
/// # Supported Languages
///
/// - English (en-US) - Default
/// - British English (en-GB)
/// - German (de-DE)
/// - French (fr-FR)
///
/// # Environment Variables
///
/// The locale is determined by checking these variables in order:
/// 1. LANG
/// 2. LC_ALL
/// 3. LC_MESSAGES
///
/// If none are set, defaults to English (en-US).
pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    /// Creates a new I18n instance with the appropriate locale bundle.
    ///
    /// The locale is determined from environment variables, with English
    /// being used as a fallback if no supported locale is found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use git_wippy::I18n;
    ///
    /// let i18n = I18n::new();
    /// ```
    pub fn new() -> Self {
        let lang = env::var("LANG")
            .or_else(|_| env::var("LC_ALL"))
            .or_else(|_| env::var("LC_MESSAGES"))
            .unwrap_or_else(|_| String::from("en"));

        let lang_id: LanguageIdentifier = lang
            .split('.')
            .next()
            .unwrap_or("en-US")
            .parse()
            .unwrap_or_else(|_| "en-US".parse().unwrap());

        let resource_path = match (
            lang_id.language.as_str(),
            lang_id.region.as_ref().map(|r| r.as_str()),
        ) {
            ("en", Some("GB")) => include_str!("../locales/en-GB.ftl"),
            ("de", Some("DE") | None) => include_str!("../locales/de-DE.ftl"),
            ("fr", Some("FR") | None) => include_str!("../locales/fr-FR.ftl"),
            _ => include_str!("../locales/en-US.ftl"),
        };

        let res = FluentResource::try_new(resource_path.to_string())
            .expect("Failed to parse FluentResource");

        let mut bundle = FluentBundle::new(vec![lang_id]);
        bundle.set_use_isolating(false);
        bundle
            .add_resource(res)
            .expect("Failed to add FluentResource to bundle");

        Self { bundle }
    }

    pub fn gettext(&self, key: &str, args: Option<HashMap<&str, &str>>) -> Result<String> {
        let msg = self
            .bundle
            .get_message(key)
            .with_context(|| format!("Message '{}' not found in bundle", key))?;

        let pattern = msg
            .value()
            .with_context(|| format!("No value for message '{}'", key))?;

        let mut fluent_args = FluentArgs::new();
        if let Some(args) = args {
            for (k, v) in args {
                fluent_args.set(k, v);
            }
        }

        let mut errors = vec![];
        let formatted = self
            .bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors);

        if !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Error formatting message '{}': {:?}",
                key,
                errors
            ));
        }

        Ok(formatted.to_string())
    }
}

// Create a thread-local instance of I18n
thread_local! {
    static I18N: I18n = I18n::new();
}

// Single t() function with optional args
pub fn t(key: &str) -> String {
    t_with_args(key, &[])
}

// t() function with args
pub fn t_with_args(key: &str, args: Args) -> String {
    let args_map: HashMap<&str, &str> = args.iter().cloned().collect();
    I18N.with(|i18n| {
        i18n.gettext(
            key,
            if args_map.is_empty() {
                None
            } else {
                Some(args_map)
            },
        )
        .unwrap_or_default()
    })
}
