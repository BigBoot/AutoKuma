mod engine;

use engine::{
    parse_compose_targets, parse_config, ConfigFormat, PlaygroundEngine,
};
use kuma_client::build::{LONG_VERSION, SHORT_VERSION};
use leptos::{ev::Event, prelude::*};
use serde_json::Value;

const DEFAULT_COMPOSE: &str = r#"services:
  homepage:
    image: ghcr.io/gethomepage/homepage:latest
    container_name: homepage
    labels:
      kuma.home.http.name: Homepage
      kuma.home.http.url: https://homepage.example.com
      kuma.home.http.max_retries: 5
      kuma.home.http.parent_name: core
      kuma.__WEB: homepage.example.com,443
      web: homepage.example.com,443

  api:
    image: ghcr.io/acme/api:latest
    labels:
      kuma.api.http.name: API
      kuma.api.http.url: https://api.example.com/health
    deploy:
      labels:
        kuma.api_service.group.name: API Service Group

  dockerhost:
    image: docker:dind
    labels:
      kuma.docker_a.docker_host.name: Docker A
      kuma.docker_a.docker_host.connection_type: socket
      kuma.docker_a.docker_host.host: unix:///var/run/docker.sock

  core-group:
    image: busybox
    labels:
      kuma.core.group.name: Core Services
"#;

const DEFAULT_CONFIG: &str = r#"[docker]
label_prefix = "kuma"

default_settings = """
*.interval: 60
http.max_retries: 3
"""

[snippets]
WEB = """
{{container_name}}_https.http.name: {{container_name}} HTTPS
{{container_name}}_https.http.url: https://{{ args[0] }}:{{ args[1] }}
"""
"#;

const DEFAULT_SNIPPET: &str = r#"
{% if container_name is defined %}
{% set base_name = container_name %}
{% else %}
{% set base_name = service.Spec.Name %}
{% endif %}
{{ base_name }}_preview.http.name: {{ base_name }} Preview
{{ base_name }}_preview.http.url: https://{{ base_name }}.example.com
{{ base_name }}_preview.http.parent_name: core
"#;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[derive(Clone)]
struct VariableRow {
    template: &'static str,
    description: &'static str,
    value: Value,
}

#[derive(Clone, PartialEq)]
struct EntityRow {
    id: String,
    entity_type: String,
    value: Value,
}

fn variable_value(value: Option<&Value>) -> Value {
    match value {
        Some(value) => value.clone(),
        None => Value::String("<not available>".to_owned()),
    }
}

fn format_scalar(value: &Value) -> String {
    value.to_string()
}

fn collapsed_preview(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                "{}".to_owned()
            } else {
                "{ ... }".to_owned()
            }
        }
        Value::Array(items) => {
            if items.is_empty() {
                "[]".to_owned()
            } else {
                "[ ... ]".to_owned()
            }
        }
        _ => format_scalar(value),
    }
}

fn render_entries(entries: Vec<(String, Value)>) -> AnyView {
    let last_index = entries.len().saturating_sub(1);
    view! {
        <div class="json-block">
            {entries
                .into_iter()
                .enumerate()
                .map(|(index, (label, value))| {
                    let is_last = index == last_index;
                    let key = label;
                    view! {
                        <div class="json-line">
                            <span class="json-key">{key}</span>
                            <span class="json-colon">": "</span>
                            <div class="json-line-value">{render_value(&value, !is_last)}</div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

fn render_nested(value: &Value, trailing_comma: bool) -> AnyView {
    let (preview, opening, closing, entries) = match value {
        Value::Object(map) => (
            collapsed_preview(value),
            "{",
            if trailing_comma { "}," } else { "}" },
            map.iter()
                .map(|(key, nested)| (format!("\"{key}\""), nested.clone()))
                .collect::<Vec<_>>(),
        ),
        Value::Array(items) => (
            collapsed_preview(value),
            "[",
            if trailing_comma { "]," } else { "]" },
            items
                .iter()
                .enumerate()
                .map(|(index, nested)| (format!("[{index}]"), nested.clone()))
                .collect::<Vec<_>>(),
        ),
        _ => return view! {}.into_any(),
    };

    view! {
        <details class="json-details">
            <summary class="json-summary">
                <span class="json-toggle json-toggle-collapsed">"+"</span>
                <span class="json-toggle json-toggle-expanded">"-"</span>
                <span class="json-preview">{preview}<span class="json-comma">{if trailing_comma { "," } else { "" }}</span></span>
                <span class="json-opening-inline">{opening}</span>
            </summary>
            <div class="json-expanded">
                <div class="json-nested">{render_entries(entries)}</div>
                <div class="json-line json-closing">{closing}</div>
            </div>
        </details>
    }
    .into_any()
}

fn render_value(value: &Value, trailing_comma: bool) -> AnyView {
    match value {
        Value::Object(_) | Value::Array(_) => render_nested(value, trailing_comma),
        _ => view! {
            <span class="scalar-value">{format_scalar(value)}</span>
            <span class="json-comma">{if trailing_comma { "," } else { "" }}</span>
        }
        .into_any(),
    }
}

fn variable_rows(target: &engine::MockTarget) -> Vec<VariableRow> {
    let context = target.context_json.as_object();
    match target.kind {
        engine::TargetKind::Container => vec![
            VariableRow {
                template: "container_id",
                description: "The container id",
                value: variable_value(context.and_then(|ctx| ctx.get("container_id"))),
            },
            VariableRow {
                template: "image_id",
                description: "Sha256 of the container image",
                value: variable_value(context.and_then(|ctx| ctx.get("image_id"))),
            },
            VariableRow {
                template: "image",
                description: "Name of the container image",
                value: variable_value(context.and_then(|ctx| ctx.get("image"))),
            },
            VariableRow {
                template: "container_name",
                description: "Name of the container",
                value: variable_value(context.and_then(|ctx| ctx.get("container_name"))),
            },
            VariableRow {
                template: "container",
                description: "Nested structure with container details",
                value: variable_value(context.and_then(|ctx| ctx.get("container"))),
            },
            VariableRow {
                template: "system_info",
                description: "Nested structure with host details",
                value: variable_value(context.and_then(|ctx| ctx.get("system_info"))),
            },
        ],
        engine::TargetKind::Service => vec![
            VariableRow {
                template: "service",
                description: "Nested structure with service details",
                value: variable_value(context.and_then(|ctx| ctx.get("service"))),
            },
            VariableRow {
                template: "system_info",
                description: "Nested structure with host details",
                value: variable_value(context.and_then(|ctx| ctx.get("system_info"))),
            },
        ],
    }
}

fn empty_variable_rows() -> Vec<VariableRow> {
    vec![VariableRow {
        template: "No Selection",
        description: "Select a service or container to inspect the documented template variables.",
        value: Value::String("No target available".to_owned()),
    }]
}

fn entity_rows(entities: Vec<engine::ParsedEntity>) -> Result<Vec<EntityRow>, engine::PlaygroundError> {
    entities
        .into_iter()
        .map(|parsed| {
            let value = serde_json::to_value(parsed.entity)
                .map_err(|err| engine::PlaygroundError::DeserializeError(err.to_string()))?;

            Ok(EntityRow {
                id: parsed.id,
                entity_type: parsed.entity_type,
                value,
            })
        })
        .collect()
}

fn build_info_lines() -> Vec<String> {
    LONG_VERSION.lines().map(str::to_owned).collect()
}

#[component]
fn App() -> impl IntoView {
    let (compose_input, set_compose_input) = signal(DEFAULT_COMPOSE.to_owned());
    let (config_input, set_config_input) = signal(DEFAULT_CONFIG.to_owned());
    let (config_format, set_config_format) = signal(ConfigFormat::Toml);
    let (snippet_input, set_snippet_input) = signal(DEFAULT_SNIPPET.to_owned());
    let (selected_target_id, set_selected_target_id) = signal(String::new());

    let parsed_config = Memo::new(move |_| parse_config(&config_input.get(), &config_format.get()));

    let engine = Memo::new(move |_| {
        parsed_config
            .get()
            .and_then(PlaygroundEngine::new)
    });

    let targets = Memo::new(move |_| parse_compose_targets(&compose_input.get()));

    let compose_input_error = Memo::new(move |_| targets.get().err().map(|err| err.to_string()));

    let config_input_error = Memo::new(move |_| engine.get().err().map(|err| err.to_string()));

    let active_target = Memo::new(move |_| {
        targets.get().ok().and_then(|targets| {
            let selected_id = selected_target_id.get();
            targets
                .iter()
                .find(|target| target.id == selected_id)
                .cloned()
                .or_else(|| targets.first().cloned())
        })
    });

    let compose_entities = Memo::new(move |_| -> Result<Vec<engine::ParsedEntity>, engine::PlaygroundError> {
        match (engine.get(), targets.get()) {
            (Ok(engine), Ok(targets)) => engine.collect_compose_entities(&targets),
            (Err(err), _) => Err(err),
            (_, Err(err)) => Err(err),
        }
    });

    let compose_preview = Memo::new(move |_| -> Result<Vec<EntityRow>, engine::PlaygroundError> {
        compose_entities.get().and_then(entity_rows)
    });

    let snippet_preview = Memo::new(move |_| -> Result<String, engine::PlaygroundError> {
        match (engine.get(), active_target.get()) {
            (Err(err), _) => Err(err),
            (Ok(_), None) => Ok("Select a mocked target to render the custom snippet.".to_owned()),
            (Ok(engine), Some(target)) => engine.render_template(&target, &snippet_input.get()),
        }
    });

    view! {
        <main class="shell">
            <section class="hero">
                <div>
                    <h1>"Autokuma Playground"</h1>
                    <p class="lede">
                        "Swiss army knife for AutoKuma: Paste your docker-compose file and AutoKuma config, inspect what service and container metadata is available, render a Tera snippet with AutoKuma-compatible context, and preview the parsed entities entirely in the browser."
                    </p>
                </div>
            </section>

            <section class="grid inputs">
                <article class="panel">
                    <div class="panel-head">
                        <h2>"docker-compose.yml"</h2>
                        <span class="badge">"YAML"</span>
                    </div>
                    <textarea
                        class="editor tall"
                        prop:value=move || compose_input.get()
                        on:input=move |ev| set_compose_input.set(event_target_value(&ev))
                    />
                    <Show
                        when=move || compose_input_error.get().is_some()
                        fallback=|| ()
                    >
                        <div class="error-block">
                            <h3>"Compose error"</h3>
                            <pre>{move || compose_input_error.get().unwrap_or_default()}</pre>
                        </div>
                    </Show>
                </article>

                <article class="panel">
                    <div class="panel-head split">
                        <h2>"AutoKuma Config"</h2>
                        <label class="format-picker">
                            <span>"Format"</span>
                            <select
                                on:change=move |ev: Event| {
                                    set_config_format.set(ConfigFormat::from(event_target_value(&ev).as_str()));
                                }
                            >
                                {ConfigFormat::all().into_iter().map(|format| {
                                    let selected_format = format.clone();
                                    let selected = move || config_format.get() == format;
                                    view! {
                                        <option value=selected_format.as_str() selected=selected>{selected_format.label()}</option>
                                    }
                                }).collect_view()}
                            </select>
                        </label>
                    </div>
                    <textarea
                        class="editor tall"
                        prop:value=move || config_input.get()
                        on:input=move |ev| set_config_input.set(event_target_value(&ev))
                    />
                    <Show
                        when=move || config_input_error.get().is_some()
                        fallback=|| ()
                    >
                        <div class="error-block">
                            <h3>"Config error"</h3>
                            <pre>{move || config_input_error.get().unwrap_or_default()}</pre>
                        </div>
                    </Show>
                </article>
            </section>
            <section class="grid">
                <article class="panel">
                    <div class="panel-head">
                        <h2>"AutoKuma Entities"</h2>
                        <span class="badge">"Auto-detected"</span>
                    </div>
                    <Show
                        when=move || compose_preview.get().is_ok()
                        fallback=move || view! {
                            <div class="error-block">
                                <h3>"Preview error"</h3>
                                <pre>{move || compose_preview.get().err().map(|e| e.to_string()).unwrap_or_default()}</pre>
                            </div>
                        }
                    >
                        <div class="variable-list">
                            {move || {
                                compose_preview
                                    .get()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|row| {
                                        view! {
                                            <section class="variable-card">
                                                <div class="variable-meta">
                                                    <code>{row.id}</code>
                                                    <p>{row.entity_type}</p>
                                                </div>
                                                <div class="variable-render">{render_value(&row.value, false)}</div>
                                            </section>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </Show>
                </article>
            </section>

            <section class="grid">
                <article class="panel">
                    <div class="panel-head">
                        <h2>"Template Workspace"</h2>
                        <span class="badge">"3 subsections"</span>
                    </div>

                    <div class="stacked-subsections">
                        <section class="subpanel">
                            <div class="panel-head">
                                <h3>"Select Context"</h3>
                                <span class="badge">{move || targets.get().ok().map(|v| v.len()).unwrap_or(0).to_string()}</span>
                            </div>
                            <Show
                                when=move || targets.get().is_ok()
                                fallback=move || view! {
                                    <div class="error-block">
                                        <h3>"Compose error"</h3>
                                        <pre>{move || targets.get().err().map(|e| e.to_string()).unwrap_or_default()}</pre>
                                    </div>
                                }
                            >
                                <label class="stack-label">
                                    <span>"Select service or container"</span>
                                    <select
                                        class="target-select"
                                        on:change=move |ev: Event| set_selected_target_id.set(event_target_value(&ev))
                                    >
                                        {move || {
                                            targets
                                                .get()
                                                .unwrap_or_default()
                                                .into_iter()
                                                .map(|target| {
                                                    let value = target.id.clone();
                                                    let option_value = value.clone();
                                                    let label = format!("{} • {}", target.kind.label(), target.name);
                                                    let is_selected = move || {
                                                        let current = selected_target_id.get();
                                                        current == value || (current.is_empty() && active_target.get().map(|t| t.id == value).unwrap_or(false))
                                                    };
                                                    view! {
                                                        <option value=option_value selected=is_selected>{label}</option>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </select>
                                </label>

                                <div class="subpanel nested-subpanel">
                                    <h3>"Available Variables"</h3>
                                    <div class="variable-list">
                                        {move || {
                                            active_target
                                                .get()
                                                .map(|target| variable_rows(&target))
                                                .unwrap_or_else(empty_variable_rows)
                                                .into_iter()
                                                .map(|row| {
                                                    view! {
                                                        <section class="variable-card">
                                                            <div class="variable-meta">
                                                                <code>{row.template}</code>
                                                                <p>{row.description}</p>
                                                            </div>
                                                            <div class="variable-render">{render_value(&row.value, false)}</div>
                                                        </section>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </div>
                                </div>
                            </Show>
                        </section>

                        <section class="subpanel">
                            <div class="panel-head">
                                <h3>"Tera Snippet"</h3>
                                <span class="badge">"Snippet syntax"</span>
                            </div>
                            <textarea
                                class="editor"
                                prop:value=move || snippet_input.get()
                                on:input=move |ev| set_snippet_input.set(event_target_value(&ev))
                            />
                            <p class="hint">
                                "Write snippet lines in AutoKuma snippet format using the documented variables for the selected target, for example "
                                <code>"my_id.http.name: {{ container_name }}"</code>
                                ". Container targets expose "
                                <code>"container_id"</code>
                                ", "
                                <code>"image_id"</code>
                                ", "
                                <code>"image"</code>
                                ", "
                                <code>"container_name"</code>
                                ", "
                                <code>"container"</code>
                                ", and "
                                <code>"system_info"</code>
                                ". Service targets expose "
                                <code>"service"</code>
                                " and "
                                <code>"system_info"</code>
                                "."
                            </p>
                        </section>

                        <section class="subpanel">
                            <div class="panel-head">
                                <h3>"Rendered Snippet"</h3>
                                <span class="badge">"Preview"</span>
                            </div>
                            <Show
                                when=move || snippet_preview.get().is_ok()
                                fallback=move || view! {
                                    <div class="error-block">
                                        <h3>"Preview error"</h3>
                                        <pre>{move || snippet_preview.get().err().map(|e| e.to_string()).unwrap_or_default()}</pre>
                                    </div>
                                }
                            >
                                <pre>{move || snippet_preview.get().unwrap_or_default()}</pre>
                            </Show>
                        </section>
                    </div>
                </article>
            </section>

            <details class="app-footer">
                <summary class="app-footer-summary">
                    <span class="footer-label">"AutoKuma Build"</span>
                    <span class="footer-version">{SHORT_VERSION}</span>
                </summary>
                <div class="footer-lines">
                    {build_info_lines().into_iter().map(|line| {
                        view! { <div>{line}</div> }
                    }).collect_view()}
                </div>
            </details>
        </main>
    }
}