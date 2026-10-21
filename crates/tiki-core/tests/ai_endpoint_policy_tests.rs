use tiki_core::ai::AiEndpointPolicy;

#[test]
fn only_the_anthropic_api_receives_the_key() {
    assert!(AiEndpointPolicy::is_allowed(
        "https://api.anthropic.com/v1/messages"
    ));
    assert!(!AiEndpointPolicy::is_allowed(
        "http://api.anthropic.com/v1/messages"
    ));
    assert!(!AiEndpointPolicy::is_allowed(
        "https://api.anthropic.com.evil.example/v1/messages"
    ));
    assert!(!AiEndpointPolicy::is_allowed(
        "https://evil.example/?https://api.anthropic.com/v1/"
    ));
    assert!(!AiEndpointPolicy::is_allowed(
        "https://api.anthropic.com/admin"
    ));
}
