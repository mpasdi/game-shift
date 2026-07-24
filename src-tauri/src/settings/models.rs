#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OnlineCoverConfigState {
    Disabled,
    MissingApiKey,
    Ready,
    InvalidApiKey,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OnlineCoverSettings {
    pub(crate) enabled: bool,
    pub(crate) has_api_key: bool,
    pub(crate) api_key_hint: Option<String>,
    pub(crate) state: OnlineCoverConfigState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StoredApiKeyStatus {
    Unknown,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StoredOnlineCoverSettings {
    pub(super) enabled: bool,
    pub(super) api_key: Option<String>,
    pub(super) api_key_status: StoredApiKeyStatus,
}

impl StoredOnlineCoverSettings {
    pub(super) fn to_public(&self) -> OnlineCoverSettings {
        let has_api_key = self.api_key.is_some();
        let state = if !self.enabled {
            OnlineCoverConfigState::Disabled
        } else if !has_api_key {
            OnlineCoverConfigState::MissingApiKey
        } else if self.api_key_status == StoredApiKeyStatus::Invalid {
            OnlineCoverConfigState::InvalidApiKey
        } else {
            OnlineCoverConfigState::Ready
        };

        OnlineCoverSettings {
            enabled: self.enabled,
            has_api_key,
            api_key_hint: self.api_key.as_deref().map(mask_api_key),
            state,
        }
    }
}

fn mask_api_key(api_key: &str) -> String {
    if api_key.chars().count() <= 4 {
        return "••••".to_string();
    }
    let suffix: String = api_key
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("•••• {suffix}")
}

#[cfg(test)]
mod tests {
    use super::{OnlineCoverConfigState, StoredApiKeyStatus, StoredOnlineCoverSettings};

    #[test]
    fn masks_the_saved_key_without_returning_the_secret() {
        let settings = StoredOnlineCoverSettings {
            enabled: true,
            api_key: Some("secret-api-key-1234".to_string()),
            api_key_status: StoredApiKeyStatus::Valid,
        }
        .to_public();

        assert_eq!(settings.api_key_hint.as_deref(), Some("•••• 1234"));
        assert_eq!(settings.state, OnlineCoverConfigState::Ready);

        let short_key = StoredOnlineCoverSettings {
            enabled: true,
            api_key: Some("key".to_string()),
            api_key_status: StoredApiKeyStatus::Unknown,
        }
        .to_public();
        assert_eq!(short_key.api_key_hint.as_deref(), Some("••••"));
    }

    #[test]
    fn derives_all_user_facing_states() {
        let disabled = StoredOnlineCoverSettings {
            enabled: false,
            api_key: Some("key".to_string()),
            api_key_status: StoredApiKeyStatus::Invalid,
        }
        .to_public();
        assert_eq!(disabled.state, OnlineCoverConfigState::Disabled);

        let missing = StoredOnlineCoverSettings {
            enabled: true,
            api_key: None,
            api_key_status: StoredApiKeyStatus::Unknown,
        }
        .to_public();
        assert_eq!(missing.state, OnlineCoverConfigState::MissingApiKey);

        let invalid = StoredOnlineCoverSettings {
            enabled: true,
            api_key: Some("key".to_string()),
            api_key_status: StoredApiKeyStatus::Invalid,
        }
        .to_public();
        assert_eq!(invalid.state, OnlineCoverConfigState::InvalidApiKey);
    }
}
