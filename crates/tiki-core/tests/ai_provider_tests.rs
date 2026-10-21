use tiki_core::ai::AiProvider;

#[test]
fn each_key_goes_only_to_its_own_api() {
    assert_eq!(
        AiProvider::for_url("https://api.anthropic.com/v1/messages"),
        Some(AiProvider::Anthropic)
    );
    assert_eq!(
        AiProvider::for_url("https://api.openai.com/v1/responses"),
        Some(AiProvider::OpenAi)
    );
    assert_eq!(
        AiProvider::for_url("http://api.anthropic.com/v1/messages"),
        None
    );
    assert_eq!(
        AiProvider::for_url("https://api.anthropic.com.evil.example/v1/messages"),
        None
    );
    assert_eq!(
        AiProvider::for_url("https://evil.example/?https://api.openai.com/v1/"),
        None
    );
}

#[test]
fn each_provider_uses_its_own_auth_header() {
    assert_eq!(
        AiProvider::Anthropic.auth_header("secret"),
        ("x-api-key", "secret".to_owned())
    );
    assert_eq!(
        AiProvider::OpenAi.auth_header("secret"),
        ("authorization", "Bearer secret".to_owned())
    );
}
