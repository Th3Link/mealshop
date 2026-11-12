use super::{Authority, Scope, api, schemas, tools};
use crate::household::{self, Shared};
use axum::{
    Extension,
    body::Bytes,
    extract::{State, rejection::BytesRejection},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{Value, json};
const MODERN: &str = "2026-07-28";
const LEGACY: &str = "2025-11-25";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}
struct Fault {
    status: StatusCode,
    code: i32,
    message: &'static str,
    data: Value,
}
impl Fault {
    fn invalid() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: -32602,
            message: "Invalid params",
            data: Value::Null,
        }
    }
}
fn fault(id: Value, f: Fault) -> Response {
    let mut body = json!({"jsonrpc":"2.0","error":{"code":f.code,"message":f.message}});
    body["id"] = id;
    body["error"]["data"] = f.data;
    household::reply(f.status, body)
}
fn protocol_error(id: Value, code: i32, message: &'static str, status: StatusCode) -> Response {
    fault(
        id,
        Fault {
            status,
            code,
            message,
            data: Value::Null,
        },
    )
}
fn server_info() -> Value {
    json!({"name":"Mealshop recipe tools","version":env!("CARGO_PKG_VERSION")})
}
fn header_matches(headers: &HeaderMap, key: &str, value: &str, encoded: bool) -> bool {
    headers
        .get(key)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| {
            if encoded {
                if let Some(text) = h
                    .strip_prefix("=?base64?")
                    .and_then(|s| s.strip_suffix("?="))
                {
                    return STANDARD.decode(text).is_ok_and(|b| b == value.as_bytes());
                }
            }
            h == value
        })
}
fn era(request: &Request, headers: &HeaderMap) -> Result<bool, Fault> {
    let meta = &request.params["_meta"];
    let version = meta["io.modelcontextprotocol/protocolVersion"].as_str();
    let header = headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok());
    if let Some(version) = version.or(header) {
        if !matches!(version, MODERN | LEGACY) {
            return Err(Fault {
                status: StatusCode::BAD_REQUEST,
                code: -32022,
                message: "Unsupported protocol version",
                data: json!({"supported":[MODERN,LEGACY],"requested":version}),
            });
        }
    }
    let modern =
        version == Some(MODERN) || header == Some(MODERN) || request.method == "server/discover";
    if modern {
        if version.is_none() || !meta["io.modelcontextprotocol/clientCapabilities"].is_object() {
            return Err(Fault::invalid());
        }
        if version != Some(MODERN)
            || !header_matches(headers, "mcp-protocol-version", MODERN, false)
            || !header_matches(headers, "mcp-method", &request.method, false)
            || (request.method == "tools/call"
                && !request.params["name"]
                    .as_str()
                    .is_some_and(|name| header_matches(headers, "mcp-name", name, true)))
        {
            return Err(Fault {
                status: StatusCode::BAD_REQUEST,
                code: -32020,
                message: "MCP headers do not match the request",
                data: Value::Null,
            });
        }
    } else if request.method != "initialize" && header != Some(LEGACY) {
        return Err(Fault {
            status: StatusCode::BAD_REQUEST,
            code: -32022,
            message: "Send the negotiated MCP protocol version",
            data: json!({"supported":[MODERN,LEGACY]}),
        });
    }
    Ok(modern)
}
fn valid_id(id: &Value) -> bool {
    id.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 128)
        || id.as_i64().is_some()
        || id.as_u64().is_some()
}
fn mime(headers: &HeaderMap, key: &str, expected: &str) -> bool {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| {
            s.split(',').any(|m| {
                m.split(';')
                    .next()
                    .is_some_and(|m| m.trim().eq_ignore_ascii_case(expected))
            })
        })
}
fn transport(headers: &HeaderMap) -> Option<Response> {
    let problem = if headers.contains_key("origin") {
        Some((
            StatusCode::FORBIDDEN,
            -32001,
            "Browser-origin RPC is not allowed",
        ))
    } else if !mime(headers, "content-type", "application/json") {
        Some((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            -32600,
            "Send application/json",
        ))
    } else if !mime(headers, "accept", "application/json")
        || !mime(headers, "accept", "text/event-stream")
    {
        Some((
            StatusCode::NOT_ACCEPTABLE,
            -32600,
            "Accept application/json and text/event-stream",
        ))
    } else {
        None
    };
    problem.map(|(status, code, message)| protocol_error(Value::Null, code, message, status))
}
pub(super) async fn handle(
    State(state): State<Shared>,
    headers: HeaderMap,
    who: Option<Extension<Authority>>,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    if !household::authorized(&state, &headers) {
        return household::denied();
    }
    if super::maintain_previews(&state).is_err() {
        return protocol_error(
            Value::Null,
            -32603,
            "MCP storage unavailable",
            StatusCode::SERVICE_UNAVAILABLE,
        );
    }
    if let Some(response) = transport(&headers) {
        return response;
    }
    let Ok(body) = body else {
        return protocol_error(
            Value::Null,
            -32600,
            "MCP request exceeds 128 KiB",
            StatusCode::PAYLOAD_TOO_LARGE,
        );
    };
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            return protocol_error(Value::Null, -32700, "Parse error", StatusCode::BAD_REQUEST);
        }
    };
    let id = value
        .get("id")
        .filter(|id| valid_id(id))
        .cloned()
        .unwrap_or(Value::Null);
    let request: Request = match serde_json::from_value(value) {
        Ok(r) => r,
        Err(_) => return protocol_error(id, -32600, "Invalid Request", StatusCode::BAD_REQUEST),
    };
    if request.jsonrpc != "2.0"
        || request.method.len() > 100
        || request.id.as_ref().is_some_and(|id| !valid_id(id))
    {
        return protocol_error(id, -32600, "Invalid Request", StatusCode::BAD_REQUEST);
    }
    let modern = match era(&request, &headers) {
        Ok(v) => v,
        Err(f) => return fault(id, f),
    };
    if request.id.is_none()
        && (modern
            || !matches!(
                request.method.as_str(),
                "notifications/initialized" | "notifications/cancelled"
            ))
    {
        return protocol_error(
            id,
            -32600,
            "This operation requires a request ID",
            StatusCode::BAD_REQUEST,
        );
    }
    let notification = request.id.is_none();
    let who = api::authority(&state, &headers, who);
    match tokio::task::spawn_blocking(move || dispatch(&state, &who, &request, modern)).await {
        Ok(Ok(mut result)) => {
            if notification {
                return (StatusCode::ACCEPTED, [("cache-control", "no-store")]).into_response();
            }
            if modern {
                result["resultType"] = json!("complete");
                result["_meta"] = json!({"io.modelcontextprotocol/serverInfo":server_info()});
            }
            household::reply(
                StatusCode::OK,
                json!({"jsonrpc":"2.0","id":id,"result":result}),
            )
        }
        Ok(Err(f)) => fault(id, f),
        Err(_) => protocol_error(
            id,
            -32603,
            "MCP storage unavailable",
            StatusCode::SERVICE_UNAVAILABLE,
        ),
    }
}
fn params(request: &Request, allowed: &[&str]) -> Result<(), Fault> {
    if request.params.is_null() {
        return Ok(());
    }
    if !request.params.as_object().is_some_and(|p| {
        p.keys()
            .all(|k| k == "_meta" || allowed.contains(&k.as_str()))
    }) {
        return Err(Fault::invalid());
    }
    Ok(())
}
fn tool_parameters(request: &Request) -> Result<(&str, Value), Fault> {
    params(request, &["name", "arguments"])?;
    let name = request.params["name"].as_str().ok_or_else(Fault::invalid)?;
    if !matches!(
        name,
        "ingredient_search"
            | "ingredient_get"
            | "recipe_search"
            | "recipe_get"
            | "recipe_preview_create"
            | "recipe_create"
    ) {
        return Err(Fault::invalid());
    }
    let args = request
        .params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !args.is_object() {
        return Err(Fault::invalid());
    }
    Ok((name, args))
}
fn dispatch(
    state: &crate::household::Household,
    actor: &Authority,
    request: &Request,
    modern: bool,
) -> Result<Value, Fault> {
    let unavailable = || Fault {
        status: StatusCode::SERVICE_UNAVAILABLE,
        code: -32603,
        message: "MCP storage unavailable",
        data: Value::Null,
    };
    let mut recipes = state
        .recipes
        .as_ref()
        .ok_or_else(unavailable)?
        .lock()
        .map_err(|_| unavailable())?;
    let ingredients = state.catalogue.lock().map_err(|_| unavailable())?;
    let mut store = state.mcp.lock().map_err(|_| unavailable())?;
    let scope = store.scope(&actor.user, actor.role);
    if scope == Scope::Disabled {
        return Err(Fault {
            status: StatusCode::FORBIDDEN,
            code: -32001,
            message: "MCP access is not enabled for this account",
            data: Value::Null,
        });
    }
    match request.method.as_str() {
        "server/discover" => {
            params(request, &[])?;
            Ok(
                json!({"supportedVersions":[MODERN,LEGACY],"capabilities":{"tools":{}},"instructions":schemas::INSTRUCTIONS,"cacheScope":"private","ttlMs":0}),
            )
        }
        "initialize" if !modern => {
            params(request, &["protocolVersion", "capabilities", "clientInfo"])?;
            if !request.params["protocolVersion"].is_string()
                || !request.params["capabilities"].is_object()
                || !request.params["clientInfo"].is_object()
            {
                return Err(Fault::invalid());
            }
            Ok(
                json!({"protocolVersion":LEGACY,"capabilities":{"tools":{}},"serverInfo":server_info(),"instructions":schemas::INSTRUCTIONS}),
            )
        }
        "notifications/initialized" | "notifications/cancelled" if !modern => Ok(json!({})),
        "ping" => {
            params(request, &[])?;
            Ok(json!({}))
        }
        "tools/list" => {
            params(request, &["cursor"])?;
            if request.params.get("cursor").is_some() {
                return Err(Fault::invalid());
            }
            Ok(
                json!({"tools":schemas::tools(scope).map_err(|_|unavailable())?,"cacheScope":"private","ttlMs":0}),
            )
        }
        "tools/call" => {
            let (name, args) = tool_parameters(request)?;
            let catalogue = ingredients.catalogue();
            let mut ctx = tools::Context {
                actor,
                scope,
                store: &mut store,
                recipes: &mut recipes,
                catalogue: &catalogue,
                ingredient_revision: ingredients.revision(),
                images: &state.images,
                now: api::now().map_err(|_| unavailable())?,
            };
            let (value, is_error) = match tools::execute(&mut ctx, name, args) {
                Ok(value) => (value, false),
                Err(error) => (json!({"code":error.code,"message":error.message}), true),
            };
            Ok(
                json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":is_error}),
            )
        }
        _ => Err(Fault {
            status: StatusCode::BAD_REQUEST,
            code: -32601,
            message: "Method not found",
            data: json!({"supported":[MODERN,LEGACY]}),
        }),
    }
}
