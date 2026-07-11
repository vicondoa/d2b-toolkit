use d2b_toolkit_core::{
    Capability, CapabilitySet, DisplayEnvironmentPosture, EnvironmentPosture, ErrorEnvelope,
    ExecutionIdentityPosture, FeatureFlag, GraphicalLaunchPosture, HelloOk, IsolationPosture,
    KnownFeatureFlag, LauncherExecDisposition, LauncherItemKind, OperationId, ProtocolToken,
    PublicRequest, PublicResponse, SessionPersistencePosture, ToolkitError, Version,
    WorkloadAvailability, WorkloadIdentity, WorkloadOpResponse, WorkloadProviderKind,
    WorkloadPublicSummary, WorkloadState, WorkloadTarget, CURRENT_PROTOCOL_VERSION,
    MAX_LAUNCHER_ITEMS_PER_WORKLOAD, MAX_PRESENTATION_TEXT_LEN, MAX_WORKLOADS_PER_RESPONSE,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

const FIXTURES: &str = "fixtures/public-workload-v3-v1";

fn fixture(name: &str) -> Value {
    let body = match name {
        "local-vm-list-response.json" => {
            include_str!("fixtures/public-workload-v3-v1/local-vm-list-response.json")
        }
        "first-class-local-vm-list-response.json" => {
            include_str!("fixtures/public-workload-v3-v1/first-class-local-vm-list-response.json")
        }
        "unsafe-local-list-response.json" => {
            include_str!("fixtures/public-workload-v3-v1/unsafe-local-list-response.json")
        }
        "workload-frames.json" => {
            include_str!("fixtures/public-workload-v3-v1/workload-frames.json")
        }
        "all-enums.json" => {
            include_str!("fixtures/public-workload-v3-v1/all-enums.json")
        }
        "malformed-secret-injections.json" => {
            include_str!("fixtures/public-workload-v3-v1/malformed-secret-injections.json")
        }
        _ => panic!("unknown {FIXTURES} fixture"),
    };
    serde_json::from_str(body).unwrap()
}

fn assert_round_trip<T>(value: &Value)
where
    T: DeserializeOwned + Serialize,
{
    let decoded: T = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), *value);
}

fn list_workload(name: &str) -> WorkloadPublicSummary {
    let response: PublicResponse = serde_json::from_value(fixture(name)).unwrap();
    match response {
        PublicResponse::Workload {
            response: WorkloadOpResponse::List(result),
            ..
        } => result.workloads.into_iter().next().unwrap(),
        other => panic!("unexpected fixture response: {other:?}"),
    }
}

#[test]
fn fixture_frames_match_public_v3_serde_exactly() {
    assert_eq!(CURRENT_PROTOCOL_VERSION, 3);
    let frames = fixture("workload-frames.json");
    for request in [
        "listRequest",
        "inventoryRequest",
        "statusRequest",
        "launcherExecRequest",
    ] {
        assert_round_trip::<PublicRequest>(&frames[request]);
    }

    assert_round_trip::<PublicResponse>(&frames["launcherExecResponse"]);
    assert_round_trip::<PublicResponse>(&fixture("local-vm-list-response.json"));
    assert_round_trip::<PublicResponse>(&fixture("first-class-local-vm-list-response.json"));
    assert_round_trip::<PublicResponse>(&fixture("unsafe-local-list-response.json"));
}

#[test]
fn hello_preserves_unknown_features_and_requires_known_features() {
    let hello = HelloOk {
        server_version: Version::new("0.4.0"),
        selected_version: Version::new("0.4.0"),
        capabilities: vec![
            KnownFeatureFlag::ConfiguredLaunchV1.wire_value(),
            KnownFeatureFlag::UnsafeLocalProviderV1.wire_value(),
            KnownFeatureFlag::UnsafeLocalShellV1.wire_value(),
            FeatureFlag::new("future-workload-feature").unwrap(),
        ],
    };
    assert!(hello.has_feature(KnownFeatureFlag::ConfiguredLaunchV1));
    hello
        .require_feature(KnownFeatureFlag::UnsafeLocalShellV1)
        .unwrap();
    let negotiated = hello.negotiated_capabilities();
    assert!(negotiated.has(KnownFeatureFlag::UnsafeLocalProviderV1));
    assert!(negotiated
        .features()
        .any(|feature| feature.as_str() == "future-workload-feature"));
    let round_trip: HelloOk =
        serde_json::from_value(serde_json::to_value(&hello).unwrap()).unwrap();
    assert_eq!(round_trip, hello);

    let old = HelloOk {
        server_version: Version::new("0.4.0"),
        selected_version: Version::new("0.4.0"),
        capabilities: vec![KnownFeatureFlag::TypedErrors.wire_value()],
    };
    assert!(matches!(
        old.require_feature(KnownFeatureFlag::ConfiguredLaunchV1),
        Err(ToolkitError::FeatureUnavailable {
            feature: KnownFeatureFlag::ConfiguredLaunchV1
        })
    ));
}

#[test]
fn first_class_local_vm_identity_needs_no_legacy_vm_name() {
    let workload = list_workload("first-class-local-vm-list-response.json");
    assert_eq!(workload.provider_kind, WorkloadProviderKind::LocalVm);
    assert!(workload.identity.legacy_vm_name.is_none());
    assert_eq!(
        workload.identity.canonical_target.as_str(),
        "builder.dev.d2b"
    );
}

#[test]
fn unsafe_local_fixture_keeps_firefox_generic_and_unknown_capability() {
    let workload = list_workload("unsafe-local-list-response.json");
    assert_eq!(workload.provider_kind, WorkloadProviderKind::UnsafeLocal);
    assert_eq!(
        workload.availability,
        WorkloadAvailability::HelperUnavailable
    );
    let firefox = &workload.launcher_items[0];
    assert_eq!(firefox.id.as_str(), "firefox");
    assert_eq!(firefox.name, "Firefox");
    assert_eq!(firefox.kind, LauncherItemKind::Exec);
    assert!(firefox.graphical);
    assert!(firefox.capabilities.has(Capability::ConfiguredLaunch));
    assert_eq!(
        workload
            .capabilities
            .unknown_iter()
            .map(ProtocolToken::as_str)
            .collect::<Vec<_>>(),
        vec!["future-desktop-capability"]
    );
    let encoded = serde_json::to_string(&workload).unwrap();
    for forbidden in ["argv", "\"env\"", "\"cwd\"", "\"path\""] {
        assert!(!encoded.contains(forbidden));
    }
}

#[test]
fn workload_collections_and_presentation_text_are_decode_bounded() {
    let workload = serde_json::to_value(list_workload("unsafe-local-list-response.json")).unwrap();
    let too_many_workloads = serde_json::json!({
        "workloads": vec![workload.clone(); MAX_WORKLOADS_PER_RESPONSE + 1]
    });
    assert!(
        serde_json::from_value::<d2b_toolkit_core::WorkloadListResult>(too_many_workloads).is_err()
    );

    let mut too_many_items = workload.clone();
    too_many_items["launcherItems"] = Value::Array(vec![
        workload["launcherItems"][0].clone();
        MAX_LAUNCHER_ITEMS_PER_WORKLOAD + 1
    ]);
    assert!(serde_json::from_value::<WorkloadPublicSummary>(too_many_items).is_err());

    let mut long_name = workload;
    long_name["launcherItems"][0]["name"] =
        Value::String("x".repeat(MAX_PRESENTATION_TEXT_LEN + 1));
    assert!(serde_json::from_value::<WorkloadPublicSummary>(long_name).is_err());
}

fn enum_values<T>(fixture: &Value, key: &str) -> Vec<T>
where
    T: DeserializeOwned + Serialize,
{
    let values: Vec<T> = serde_json::from_value(fixture[key].clone()).unwrap();
    assert_eq!(serde_json::to_value(&values).unwrap(), fixture[key]);
    values
}

#[test]
fn every_provider_posture_and_state_enum_round_trips() {
    let values = fixture("all-enums.json");
    assert_eq!(
        enum_values::<WorkloadProviderKind>(&values, "providerKinds").len(),
        4
    );
    assert_eq!(
        enum_values::<IsolationPosture>(&values, "isolationPostures").len(),
        3
    );
    assert_eq!(
        enum_values::<EnvironmentPosture>(&values, "environmentPostures").len(),
        2
    );
    assert_eq!(
        enum_values::<DisplayEnvironmentPosture>(&values, "displayEnvironmentPostures").len(),
        3
    );
    assert_eq!(
        enum_values::<ExecutionIdentityPosture>(&values, "executionIdentityPostures").len(),
        3
    );
    assert_eq!(
        enum_values::<SessionPersistencePosture>(&values, "sessionPersistencePostures").len(),
        2
    );
    assert_eq!(
        enum_values::<WorkloadAvailability>(&values, "availability").len(),
        8
    );
    assert_eq!(
        enum_values::<GraphicalLaunchPosture>(&values, "graphicalLaunchPostures").len(),
        5
    );
    assert_eq!(
        enum_values::<WorkloadState>(&values, "workloadStates").len(),
        5
    );
    assert_eq!(
        enum_values::<LauncherItemKind>(&values, "launcherItemKinds").len(),
        2
    );
    assert_eq!(
        enum_values::<LauncherExecDisposition>(&values, "launcherExecDispositions").len(),
        2
    );

    let capabilities: CapabilitySet =
        serde_json::from_value(values["knownCapabilities"].clone()).unwrap();
    assert_eq!(capabilities.iter().count(), 22);
    assert_eq!(
        serde_json::to_value(capabilities).unwrap(),
        values["knownCapabilities"]
    );
}

#[test]
fn launcher_selection_is_explicit_then_default_then_sole() {
    let mut workload = list_workload("unsafe-local-list-response.json");
    let terminal = ProtocolToken::parse("terminal").unwrap();
    assert_eq!(
        workload.select_launcher_item(Some(&terminal)).unwrap().name,
        "Terminal"
    );
    assert_eq!(workload.select_launcher_item(None).unwrap().name, "Firefox");

    workload.default_item_id = None;
    let err = workload.select_launcher_item(None).unwrap_err();
    match err {
        ToolkitError::AmbiguousLauncherItems { candidates } => {
            assert_eq!(candidates.len(), 2);
            assert_eq!(candidates.as_slice()[0].id(), "firefox");
            assert_eq!(candidates.as_slice()[0].name(), "Firefox");
            assert_eq!(candidates.as_slice()[1].id(), "terminal");
        }
        other => panic!("unexpected selection error: {other:?}"),
    }

    workload.launcher_items.truncate(1);
    assert_eq!(
        workload.select_launcher_item(None).unwrap().id.as_str(),
        "firefox"
    );
}

#[test]
fn malformed_and_secret_injections_fail_without_echoing_values() {
    let malformed = fixture("malformed-secret-injections.json");
    let request_error =
        serde_json::from_value::<PublicRequest>(malformed["requestInjection"].clone()).unwrap_err();
    let summary_error =
        serde_json::from_value::<WorkloadPublicSummary>(malformed["summaryInjection"].clone())
            .unwrap_err();
    let identity_error =
        serde_json::from_value::<WorkloadIdentity>(malformed["malformedIdentity"].clone())
            .unwrap_err();
    let target_error =
        WorkloadTarget::parse(malformed["malformedTarget"].as_str().unwrap()).unwrap_err();
    let operation_error =
        OperationId::parse(malformed["secretShapedOperationId"].as_str().unwrap()).unwrap_err();
    let rendered = format!(
        "{request_error:?} {summary_error:?} {identity_error:?} \
         {target_error:?} {operation_error:?}"
    );
    for canary in [
        "ARG_SECRET_CANARY",
        "ENV_SECRET_CANARY",
        "CWD_SECRET_CANARY",
        "PATH_SECRET_CANARY",
        "OUTPUT_SECRET_CANARY",
        "OPAQUE_SESSION_CANARY",
        "launch-secret-operation",
    ] {
        assert!(!rendered.contains(canary), "leaked {canary}");
    }
}

#[test]
fn debug_and_metric_labels_exclude_identity_and_operation_values() {
    let frames = fixture("workload-frames.json");
    let request: PublicRequest =
        serde_json::from_value(frames["launcherExecRequest"].clone()).unwrap();
    let response: PublicResponse =
        serde_json::from_value(frames["launcherExecResponse"].clone()).unwrap();
    for rendered in [format!("{request:?}"), format!("{response:?}")] {
        assert!(!rendered.contains("launch-42"));
    }

    let envelope = ErrorEnvelope {
        kind: "provider-unavailable".into(),
        exit_code: 69,
        message: "OUTPUT_SECRET_CANARY".into(),
        remediation: "/home/alice/CWD_SECRET_CANARY".into(),
    };
    assert_eq!(envelope.metrics_label_value(), "daemon-error");
    let rendered = format!("{envelope:?}");
    assert!(!rendered.contains("OUTPUT_SECRET_CANARY"));
    assert!(!rendered.contains("CWD_SECRET_CANARY"));

    let workload = list_workload("unsafe-local-list-response.json");
    let labels = [
        workload.provider_kind.metrics_label_value(),
        workload.execution_posture.isolation.metrics_label_value(),
        workload.availability.metrics_label_value(),
        workload.graphical_posture.metrics_label_value(),
        workload.state.metrics_label_value(),
    ];
    assert_eq!(
        labels,
        [
            "unsafe-local",
            "unsafe-local",
            "helper-unavailable",
            "graphical-session-inactive",
            "stopped"
        ]
    );
    for label in labels {
        assert!(!label.contains("tools"));
        assert!(!label.contains("Firefox"));
        assert!(!label.contains("alice"));
    }
}

#[test]
fn private_helper_protocol_is_absent_from_sources_and_dependencies() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let mut source = String::new();
    collect_source(workspace, &mut source);
    let forbidden = [
        ["Unsafe", "Local", "Helper", "Request"].concat(),
        ["Unsafe", "Local", "Helper", "Response"].concat(),
        ["Helper", "Registration"].concat(),
        ["Helper", "Hello"].concat(),
        ["unsafe-local-", "helper", ".sock"].concat(),
        ["d2b-unsafe-local-", "helper"].concat(),
    ];
    for token in forbidden {
        assert!(
            !source.contains(&token),
            "private helper protocol token entered toolkit sources"
        );
    }
}

fn collect_source(path: &Path, output: &mut String) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == ".git" || name == "target")
        {
            continue;
        }
        if path.is_dir() {
            collect_source(&path, output);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            || path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml")
        {
            output.push_str(&fs::read_to_string(path).unwrap());
        }
    }
}
