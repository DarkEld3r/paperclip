#[macro_use]
extern crate lazy_static;
use paperclip::api_v2_schema;
#[macro_use]
extern crate serde_derive;

use paperclip::v2::{
    self,
    codegen::{DefaultEmitter, Emitter, EmitterState},
    models::{Api, HttpMethod, Version},
};

use std::fs::File;
use std::io::Read;

lazy_static! {
    static ref ROOT: String = String::from(env!("CARGO_MANIFEST_DIR"));
    static ref SCHEMA: Api<K8sSchema> = {
        let fd =
            File::open(ROOT.clone() + "/tests/k8s-v1.16.0-alpha.0-openapi-v2.json").expect("file?");
        let raw: Api<K8sSchema> = v2::from_reader(fd).expect("deserializing spec");
        raw.resolve().expect("resolution")
    };
    static ref CODEGEN: () = {
        env_logger::builder()
            .filter(Some("paperclip"), log::LevelFilter::Info)
            .init();
        let mut state = EmitterState::default();
        state.working_dir = (&*ROOT).into();
        state.working_dir.push("tests");
        state.working_dir.push("test_k8s");

        let emitter = DefaultEmitter::from(state);
        emitter.generate(&SCHEMA).expect("codegen");
    };
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PatchStrategy {
    Merge,
    RetainKeys,
    #[serde(rename = "merge,retainKeys")]
    MergeAndRetain,
}

#[api_v2_schema]
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct K8sSchema {
    #[serde(rename = "x-kubernetes-patch-strategy")]
    patch_strategy: Option<PatchStrategy>,
}

#[test]
fn test_definition_ref_cycles() {
    assert_eq!(SCHEMA.swagger, Version::V2);
    assert_eq!(SCHEMA.definitions.len(), 614);

    let json_props_def = &SCHEMA.definitions
        ["io.k8s.apiextensions-apiserver.pkg.apis.apiextensions.v1beta1.JSONSchemaProps"];
    let desc = json_props_def.read().description.clone();
    let all_of = json_props_def.read().properties["allOf"].clone();
    let items = all_of.read().items.as_ref().unwrap().clone();
    assert_eq!(items.read().description, desc); // both point to same `JSONSchemaProps`
}

#[test]
fn test_resolved_schema() {
    let api_versions = &SCHEMA.paths["/api/"].methods[&HttpMethod::Get].responses["200"].schema;
    let schema = api_versions.as_ref().expect("bleh?").read();
    assert!(schema.reference.is_none()); // this was a reference
    assert_eq!(
        &SCHEMA.definitions["io.k8s.apimachinery.pkg.apis.meta.v1.APIVersions"]
            .read()
            .description
            .as_ref()
            .unwrap()
            .as_str(),
        schema.description.as_ref().unwrap()
    );
}

fn assert_file_contains_content_at(path: &str, matching_content: &str, index: usize) {
    let _ = &*CODEGEN;

    let mut contents = String::new();
    let mut fd = File::open(path).expect("missing file");
    fd.read_to_string(&mut contents).expect("reading file");

    assert_eq!(contents.find(matching_content), Some(index));
}

#[test]
fn test_child_module_declarations() {
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/api/mod.rs"),
        "
pub mod admissionregistration {
    include!(\"./admissionregistration/mod.rs\");
}

pub mod apps {
    include!(\"./apps/mod.rs\");
}

pub mod auditregistration {
    include!(\"./auditregistration/mod.rs\");
}

pub mod authentication {
    include!(\"./authentication/mod.rs\");
}

pub mod authorization {
    include!(\"./authorization/mod.rs\");
}

pub mod autoscaling {
    include!(\"./autoscaling/mod.rs\");
}

pub mod batch {
    include!(\"./batch/mod.rs\");
}

pub mod certificates {
    include!(\"./certificates/mod.rs\");
}

pub mod coordination {
    include!(\"./coordination/mod.rs\");
}

pub mod core {
    include!(\"./core/mod.rs\");
}

pub mod events {
    include!(\"./events/mod.rs\");
}

pub mod extensions {
    include!(\"./extensions/mod.rs\");
}

pub mod networking {
    include!(\"./networking/mod.rs\");
}

pub mod node {
    include!(\"./node/mod.rs\");
}

pub mod policy {
    include!(\"./policy/mod.rs\");
}

pub mod rbac {
    include!(\"./rbac/mod.rs\");
}

pub mod scheduling {
    include!(\"./scheduling/mod.rs\");
}

pub mod settings {
    include!(\"./settings/mod.rs\");
}

pub mod storage {
    include!(\"./storage/mod.rs\");
}
",
        0,
    );

    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/api/batch/v1/mod.rs"),
        "
pub mod job {
    include!(\"./job.rs\");
}

pub mod job_condition {
    include!(\"./job_condition.rs\");
}

pub mod job_list {
    include!(\"./job_list.rs\");
}

pub mod job_spec {
    include!(\"./job_spec.rs\");
}

pub mod job_status {
    include!(\"./job_status.rs\");
}
",
        0,
    );
}

#[test]
fn test_struct_for_complex_object() {
    // We're interested in this definition because:
    // - It uses some Rust keywords.
    // - It has a number of camelcase fields.
    // - It has some fields which are maps.
    // - It uses pretty much all types (including custom types).
    // - It references other definitions (directly and through an array).
    // - It's a cyclic type.
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/apiextensions_apiserver/pkg/apis/apiextensions/v1beta1/json_schema_props.rs"),
        "
/// JSONSchemaProps is a JSON-Schema following Specification Draft 4 (http://json-schema.org/).
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct JsonSchemaProps {
    #[serde(rename = \"$ref\")]
    pub ref_: Option<String>,
    #[serde(rename = \"$schema\")]
    pub schema: Option<String>,
    #[serde(rename = \"additionalItems\")]
    pub additional_items: Option<String>,
    #[serde(rename = \"additionalProperties\")]
    pub additional_properties: Option<String>,
    #[serde(rename = \"allOf\")]
    pub all_of: Option<Vec<crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    #[serde(rename = \"anyOf\")]
    pub any_of: Option<Vec<crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    pub default: Option<String>,
    pub definitions: Option<std::collections::BTreeMap<String, crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    pub dependencies: Option<std::collections::BTreeMap<String, String>>,
    pub description: Option<String>,
    #[serde(rename = \"enum\")]
    pub enum_: Option<Vec<String>>,
    pub example: Option<String>,
    #[serde(rename = \"exclusiveMaximum\")]
    pub exclusive_maximum: Option<bool>,
    #[serde(rename = \"exclusiveMinimum\")]
    pub exclusive_minimum: Option<bool>,
    #[serde(rename = \"externalDocs\")]
    pub external_docs: Option<crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::external_documentation::ExternalDocumentation>,
    pub format: Option<String>,
    pub id: Option<String>,
    pub items: Option<String>,
    #[serde(rename = \"maxItems\")]
    pub max_items: Option<i64>,
    #[serde(rename = \"maxLength\")]
    pub max_length: Option<i64>,
    #[serde(rename = \"maxProperties\")]
    pub max_properties: Option<i64>,
    pub maximum: Option<f64>,
    #[serde(rename = \"minItems\")]
    pub min_items: Option<i64>,
    #[serde(rename = \"minLength\")]
    pub min_length: Option<i64>,
    #[serde(rename = \"minProperties\")]
    pub min_properties: Option<i64>,
    pub minimum: Option<f64>,
    #[serde(rename = \"multipleOf\")]
    pub multiple_of: Option<f64>,
    pub not: Option<Box<crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    pub nullable: Option<bool>,
    #[serde(rename = \"oneOf\")]
    pub one_of: Option<Vec<crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    pub pattern: Option<String>,
    #[serde(rename = \"patternProperties\")]
    pub pattern_properties: Option<std::collections::BTreeMap<String, crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    pub properties: Option<std::collections::BTreeMap<String, crate::io::k8s::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::json_schema_props::JsonSchemaProps>>,
    pub required: Option<Vec<String>>,
    pub title: Option<String>,
    #[serde(rename = \"type\")]
    pub type_: Option<String>,
    #[serde(rename = \"uniqueItems\")]
    pub unique_items: Option<bool>,
    /// x-kubernetes-embedded-resource defines that the value is an embedded Kubernetes runtime.Object, with TypeMeta and ObjectMeta. The type must be object. It is allowed to further restrict the embedded object. kind, apiVersion and metadata are validated automatically. x-kubernetes-preserve-unknown-fields is allowed to be true, but does not have to be if the object is fully specified (up to kind, apiVersion, metadata).
    #[serde(rename = \"x-kubernetes-embedded-resource\")]
    pub x_kubernetes_embedded_resource: Option<bool>,
    /// x-kubernetes-int-or-string specifies that this value is either an integer or a string. If this is true, an empty type is allowed and type as child of anyOf is permitted if following one of the following patterns:
    ///
    /// 1) anyOf:
    ///    - type: integer
    ///    - type: string
    /// 2) allOf:
    ///    - anyOf:
    ///      - type: integer
    ///      - type: string
    ///    - ... zero or more
    #[serde(rename = \"x-kubernetes-int-or-string\")]
    pub x_kubernetes_int_or_string: Option<bool>,
    /// x-kubernetes-preserve-unknown-fields stops the API server decoding step from pruning fields which are not specified in the validation schema. This affects fields recursively, but switches back to normal pruning behaviour if nested properties or additionalProperties are specified in the schema. This can either be true or undefined. False is forbidden.
    #[serde(rename = \"x-kubernetes-preserve-unknown-fields\")]
    pub x_kubernetes_preserve_unknown_fields: Option<bool>,
}
",
        0,
    );
}

#[test]
fn test_root_mod() {
    // Root mod contains the builder markers and helper code for client.
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/mod.rs"),
        "
pub mod k8s {
    include!(\"./k8s/mod.rs\");
}

pub mod client {
    use futures::Future;

    /// Common API errors.
    #[derive(Debug, Fail)]
    pub enum ApiError {
        #[fail(display = \"API request failed for path: {} (code: {})\", _0, _1)]
        Failure(String, reqwest::StatusCode),
        #[fail(display = \"An error has occurred while performing the API request: {}\", _0)]
        Reqwest(reqwest::Error),
    }

    /// Represents an API client.
    pub trait ApiClient {
        /// Base path for this API.
        fn base_url(&self) -> &'static str { \"https://example.com\" }

        /// Consumes a method and a relative path and produces a request builder for a single API call.
        fn request_builder(&self, method: reqwest::Method, rel_path: &str) -> reqwest::r#async::RequestBuilder;
    }

    impl ApiClient for reqwest::r#async::Client {
        #[inline]
        fn request_builder(&self, method: reqwest::Method, rel_path: &str) -> reqwest::r#async::RequestBuilder {
            self.request(method, &(String::from(self.base_url()) + rel_path))
        }
    }

    /// A trait for indicating that the implementor can send an API call.
    pub trait Sendable {
        /// The output object from this API request.
        type Output: serde::de::DeserializeOwned + Send + 'static;

        /// HTTP method used by this call.
        const METHOD: reqwest::Method;

        /// Relative URL for this API call formatted appropriately with parameter values.
        ///
        /// **NOTE:** This URL **must** begin with `/`.
        fn rel_path(&self) -> std::borrow::Cow<'static, str>;

        /// Modifier for this object. Builders override this method if they
        /// wish to add query parameters, set body, etc.
        fn modify(&self, req: reqwest::r#async::RequestBuilder) -> reqwest::r#async::RequestBuilder {
            req
        }

        /// Sends the request and returns a future for the response object.
        fn send(&self, client: &dyn ApiClient) -> Box<dyn Future<Item=Self::Output, Error=ApiError> + Send> {
            Box::new(self.send_raw(client).and_then(|mut resp| {
                resp.json::<Self::Output>().map_err(ApiError::Reqwest)
            })) as Box<_>
        }

        /// Convenience method for returning a raw response after sending a request.
        fn send_raw(&self, client: &dyn ApiClient) -> Box<dyn Future<Item=reqwest::r#async::Response, Error=ApiError> + Send> {
            let rel_path = self.rel_path();
            let req = client.request_builder(Self::METHOD, &rel_path);
            Box::new(self.modify(req).send().map_err(ApiError::Reqwest).and_then(move |resp| {
                if resp.status().is_success() {
                    futures::future::ok(resp)
                } else {
                    futures::future::err(ApiError::Failure(rel_path.into_owned(), resp.status()).into())
                }
            })) as Box<_>
        }
    }
}

pub mod generics {
    pub trait Optional {}
",
        0,
    );
}

#[test]
fn test_same_object_creates_multiple_builders() {
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/api/core/v1/config_map.rs"),
        "
impl ConfigMap {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ConfigMapBuilder {
        ConfigMapBuilder {
            body: Default::default(),
        }
    }

    /// create a ConfigMap
    #[inline]
    pub fn create_core_v1_namespaced_config_map() -> ConfigMapPostBuilder<crate::io::generics::MissingNamespace> {
        ConfigMapPostBuilder {
            inner: Default::default(),
            _param_namespace: core::marker::PhantomData,
        }
    }

    /// read the specified ConfigMap
    #[inline]
    pub fn read_core_v1_namespaced_config_map() -> ConfigMapGetBuilder1<crate::io::generics::MissingName, crate::io::generics::MissingNamespace> {
        ConfigMapGetBuilder1 {
            inner: Default::default(),
            _param_name: core::marker::PhantomData,
            _param_namespace: core::marker::PhantomData,
        }
    }

    /// replace the specified ConfigMap
    #[inline]
    pub fn replace_core_v1_namespaced_config_map() -> ConfigMapPutBuilder1<crate::io::generics::MissingName, crate::io::generics::MissingNamespace> {
        ConfigMapPutBuilder1 {
            inner: Default::default(),
            _param_name: core::marker::PhantomData,
            _param_namespace: core::marker::PhantomData,
        }
    }
}

impl Into<ConfigMap> for ConfigMapBuilder {
    fn into(self) -> ConfigMap {
        self.body
    }
}

impl Into<ConfigMap> for ConfigMapPostBuilder<crate::io::generics::NamespaceExists> {
    fn into(self) -> ConfigMap {
        self.inner.body
    }
}

impl Into<ConfigMap> for ConfigMapPutBuilder1<crate::io::generics::NameExists, crate::io::generics::NamespaceExists> {
    fn into(self) -> ConfigMap {
        self.inner.body
    }
}
",
        1880,
    );
}

#[test]
fn test_same_object_with_multiple_builders_has_basic_builder() {
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/api/core/v1/pod.rs"),
        "
/// Builder for [`Pod`](./struct.Pod.html) object.
#[derive(Debug, Clone)]
pub struct PodBuilder {
    body: Pod,
}

impl PodBuilder {
    /// APIVersion defines the versioned schema of this representation of an object. Servers should convert recognized schemas to the latest internal value, and may reject unrecognized values. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#resources
    #[inline]
    pub fn api_version(mut self, value: impl Into<String>) -> Self {
        self.body.api_version = Some(value.into());
        self
    }

    /// Kind is a string value representing the REST resource this object represents. Servers may infer this from the endpoint the client submits requests to. Cannot be updated. In CamelCase. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#types-kinds
    #[inline]
    pub fn kind(mut self, value: impl Into<String>) -> Self {
        self.body.kind = Some(value.into());
        self
    }

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    #[inline]
    pub fn metadata(mut self, value: crate::io::k8s::apimachinery::pkg::apis::meta::v1::object_meta::ObjectMeta) -> Self {
        self.body.metadata = Some(value.into());
        self
    }

    /// Specification of the desired behavior of the pod. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    #[inline]
    pub fn spec(mut self, value: crate::io::k8s::api::core::v1::pod_spec::PodSpecBuilder<crate::io::generics::ContainersExists>) -> Self {
        self.body.spec = Some(value.into());
        self
    }

    /// Most recently observed status of the pod. This data may not be up to date. Populated by the system. Read-only. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    #[inline]
    pub fn status(mut self, value: crate::io::k8s::api::core::v1::pod_status::PodStatus) -> Self {
        self.body.status = Some(value.into());
        self
    }
}
",
        4140,
    )
}

#[test]
fn test_simple_object_builder_with_required_fields() {
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/api/rbac/v1/policy_rule.rs"),
        "
impl PolicyRule {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> PolicyRuleBuilder<crate::io::generics::MissingVerbs> {
        PolicyRuleBuilder {
            inner: Default::default(),
            _verbs: core::marker::PhantomData,
        }
    }
}

impl Into<PolicyRule> for PolicyRuleBuilder<crate::io::generics::VerbsExists> {
    fn into(self) -> PolicyRule {
        self.inner.body
    }
}

/// Builder for [`PolicyRule`](./struct.PolicyRule.html) object.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PolicyRuleBuilder<Verbs> {
    inner: PolicyRuleBuilderContainer,
    _verbs: core::marker::PhantomData<Verbs>,
}

#[derive(Debug, Default, Clone)]
struct PolicyRuleBuilderContainer {
    body: PolicyRule,
}

impl<Verbs> PolicyRuleBuilder<Verbs> {
    /// APIGroups is the name of the APIGroup that contains the resources.  If multiple API groups are specified, any action requested against one of the enumerated resources in any API group will be allowed.
    #[inline]
    pub fn api_groups(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.inner.body.api_groups = Some(value.map(|value| value.into()).collect::<Vec<_>>());
        self
    }

    /// NonResourceURLs is a set of partial urls that a user should have access to.  *s are allowed, but only as the full, final step in the path Since non-resource URLs are not namespaced, this field is only applicable for ClusterRoles referenced from a ClusterRoleBinding. Rules can either apply to API resources (such as \"pods\" or \"secrets\") or non-resource URL paths (such as \"/api\"),  but not both.
    #[inline]
    pub fn non_resource_ur_ls(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.inner.body.non_resource_ur_ls = Some(value.map(|value| value.into()).collect::<Vec<_>>());
        self
    }

    /// ResourceNames is an optional white list of names that the rule applies to.  An empty set means that everything is allowed.
    #[inline]
    pub fn resource_names(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.inner.body.resource_names = Some(value.map(|value| value.into()).collect::<Vec<_>>());
        self
    }

    /// Resources is a list of resources this rule applies to.  ResourceAll represents all resources.
    #[inline]
    pub fn resources(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.inner.body.resources = Some(value.map(|value| value.into()).collect::<Vec<_>>());
        self
    }

    /// Verbs is a list of Verbs that apply to ALL the ResourceKinds and AttributeRestrictions contained in this rule.  VerbAll represents all kinds.
    #[inline]
    pub fn verbs(mut self, value: impl Iterator<Item = impl Into<String>>) -> PolicyRuleBuilder<crate::io::generics::VerbsExists> {
        self.inner.body.verbs = value.map(|value| value.into()).collect::<Vec<_>>();
        unsafe { std::mem::transmute(self) }
    }
}
",
        1564,
    );
}

#[test]
fn test_builder_with_field_parameter_collision_and_method_collision() {
    // grace_period_seconds, orphan_dependents and propagation_policy
    // exist in both the object and as a query parameter. If one is set,
    // we should also set the other.
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/apimachinery/pkg/apis/meta/v1/delete_options.rs"),
        "
/// Builder created by [`DeleteOptions::delete_rbac_authorization_v1_namespaced_role`](./struct.DeleteOptions.html#method.delete_rbac_authorization_v1_namespaced_role) method for a `DELETE` operation associated with `DeleteOptions`.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct DeleteOptionsDeleteBuilder59<Name, Namespace> {
    inner: DeleteOptionsDeleteBuilder59Container,
    _param_name: core::marker::PhantomData<Name>,
    _param_namespace: core::marker::PhantomData<Namespace>,
}

#[derive(Debug, Default, Clone)]
struct DeleteOptionsDeleteBuilder59Container {
    body: DeleteOptions,
    param_dry_run: Option<String>,
    param_grace_period_seconds: Option<i64>,
    param_orphan_dependents: Option<bool>,
    param_propagation_policy: Option<String>,
    param_name: Option<String>,
    param_namespace: Option<String>,
    param_pretty: Option<String>,
}

impl<Name, Namespace> DeleteOptionsDeleteBuilder59<Name, Namespace> {
    /// When present, indicates that modifications should not be persisted. An invalid or unrecognized dryRun directive will result in an error response and no further processing of the request. Valid values are: - All: all dry run stages will be processed
    #[inline]
    pub fn dry_run(mut self, value: impl Into<String>) -> Self {
        self.inner.param_dry_run = Some(value.into());
        self
    }

    /// The duration in seconds before the object should be deleted. Value must be non-negative integer. The value zero indicates delete immediately. If this value is nil, the default grace period for the specified type will be used. Defaults to a per object value if not specified. zero means delete immediately.
    #[inline]
    pub fn grace_period_seconds(mut self, value: impl Into<i64>) -> Self {
        self.inner.param_grace_period_seconds = Some({
            let val = value.into();
            self.inner.body.grace_period_seconds = val.clone().into();
            val
        });
        self
    }

    /// Deprecated: please use the PropagationPolicy, this field will be deprecated in 1.7. Should the dependent objects be orphaned. If true/false, the \"orphan\" finalizer will be added to/removed from the object's finalizers list. Either this field or PropagationPolicy may be set, but not both.
    #[inline]
    pub fn orphan_dependents(mut self, value: impl Into<bool>) -> Self {
        self.inner.param_orphan_dependents = Some({
            let val = value.into();
            self.inner.body.orphan_dependents = val.clone().into();
            val
        });
        self
    }

    /// Whether and how garbage collection will be performed. Either this field or OrphanDependents may be set, but not both. The default policy is decided by the existing finalizer set in the metadata.finalizers and the resource-specific default policy. Acceptable values are: 'Orphan' - orphan the dependents; 'Background' - allow the garbage collector to delete the dependents in the background; 'Foreground' - a cascading policy that deletes all dependents in the foreground.
    #[inline]
    pub fn propagation_policy(mut self, value: impl Into<String>) -> Self {
        self.inner.param_propagation_policy = Some({
            let val = value.into();
            self.inner.body.propagation_policy = val.clone().into();
            val
        });
        self
    }

    /// name of the Role
    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> DeleteOptionsDeleteBuilder59<crate::io::generics::NameExists, Namespace> {
        self.inner.param_name = Some(value.into());
        unsafe { std::mem::transmute(self) }
    }

    /// object name and auth scope, such as for teams and projects
    #[inline]
    pub fn namespace(mut self, value: impl Into<String>) -> DeleteOptionsDeleteBuilder59<Name, crate::io::generics::NamespaceExists> {
        self.inner.param_namespace = Some(value.into());
        unsafe { std::mem::transmute(self) }
    }

    /// If 'true', then the output is pretty printed.
    #[inline]
    pub fn pretty(mut self, value: impl Into<String>) -> Self {
        self.inner.param_pretty = Some(value.into());
        self
    }

    /// APIVersion defines the versioned schema of this representation of an object. Servers should convert recognized schemas to the latest internal value, and may reject unrecognized values. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#resources
    #[inline]
    pub fn api_version(mut self, value: impl Into<String>) -> Self {
        self.inner.body.api_version = Some(value.into());
        self
    }

    /// Kind is a string value representing the REST resource this object represents. Servers may infer this from the endpoint the client submits requests to. Cannot be updated. In CamelCase. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#types-kinds
    #[inline]
    pub fn kind(mut self, value: impl Into<String>) -> Self {
        self.inner.body.kind = Some(value.into());
        self
    }

    /// Must be fulfilled before a deletion is carried out. If not possible, a 409 Conflict status will be returned.
    #[inline]
    pub fn preconditions(mut self, value: crate::io::k8s::apimachinery::pkg::apis::meta::v1::preconditions::Preconditions) -> Self {
        self.inner.body.preconditions = Some(value.into());
        self
    }
}

impl crate::io::client::Sendable for DeleteOptionsDeleteBuilder59<crate::io::generics::NameExists, crate::io::generics::NamespaceExists> {
    type Output = crate::io::k8s::apimachinery::pkg::apis::meta::v1::status::Status;

    const METHOD: reqwest::Method = reqwest::Method::DELETE;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        format!(\"/apis/rbac.authorization.k8s.io/v1/namespaces/{namespace}/roles/{name}\", name=self.inner.param_name.as_ref().expect(\"missing parameter name?\"), namespace=self.inner.param_namespace.as_ref().expect(\"missing parameter namespace?\")).into()
    }

    fn modify(&self, req: reqwest::r#async::RequestBuilder) -> reqwest::r#async::RequestBuilder {
        req
        .json(&self.inner.body)
        .query(&[
            (\"dryRun\", self.inner.param_dry_run.as_ref().map(std::string::ToString::to_string)),
            (\"gracePeriodSeconds\", self.inner.param_grace_period_seconds.as_ref().map(std::string::ToString::to_string)),
            (\"orphanDependents\", self.inner.param_orphan_dependents.as_ref().map(std::string::ToString::to_string)),
            (\"propagationPolicy\", self.inner.param_propagation_policy.as_ref().map(std::string::ToString::to_string)),
            (\"pretty\", self.inner.param_pretty.as_ref().map(std::string::ToString::to_string))
        ])
    }
}
",
        436104,
    );
}

#[test]
fn test_unit_builder_with_no_modifier() {
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/apimachinery/pkg/apis/meta/v1/api_group_list.rs"),
        "
impl ApiGroupList {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ApiGroupListBuilder<crate::io::generics::MissingGroups> {
        ApiGroupListBuilder {
            inner: Default::default(),
            _groups: core::marker::PhantomData,
        }
    }

    /// get available API versions
    #[inline]
    pub fn get() -> ApiGroupListGetBuilder {
        ApiGroupListGetBuilder
    }
}

impl Into<ApiGroupList> for ApiGroupListBuilder<crate::io::generics::GroupsExists> {
    fn into(self) -> ApiGroupList {
        self.inner.body
    }
}

/// Builder for [`ApiGroupList`](./struct.ApiGroupList.html) object.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ApiGroupListBuilder<Groups> {
    inner: ApiGroupListBuilderContainer,
    _groups: core::marker::PhantomData<Groups>,
}

#[derive(Debug, Default, Clone)]
struct ApiGroupListBuilderContainer {
    body: ApiGroupList,
}

impl<Groups> ApiGroupListBuilder<Groups> {
    /// APIVersion defines the versioned schema of this representation of an object. Servers should convert recognized schemas to the latest internal value, and may reject unrecognized values. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#resources
    #[inline]
    pub fn api_version(mut self, value: impl Into<String>) -> Self {
        self.inner.body.api_version = Some(value.into());
        self
    }

    /// groups is a list of APIGroup.
    #[inline]
    pub fn groups(mut self, value: impl Iterator<Item = crate::io::k8s::apimachinery::pkg::apis::meta::v1::api_group::ApiGroupBuilder<crate::io::generics::NameExists, crate::io::generics::VersionsExists>>) -> ApiGroupListBuilder<crate::io::generics::GroupsExists> {
        self.inner.body.groups = value.map(|value| value.into()).collect::<Vec<_>>();
        unsafe { std::mem::transmute(self) }
    }

    /// Kind is a string value representing the REST resource this object represents. Servers may infer this from the endpoint the client submits requests to. Cannot be updated. In CamelCase. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#types-kinds
    #[inline]
    pub fn kind(mut self, value: impl Into<String>) -> Self {
        self.inner.body.kind = Some(value.into());
        self
    }
}

/// Builder created by [`ApiGroupList::get`](./struct.ApiGroupList.html#method.get) method for a `GET` operation associated with `ApiGroupList`.
#[derive(Debug, Clone)]
pub struct ApiGroupListGetBuilder;


impl crate::io::client::Sendable for ApiGroupListGetBuilder {
    type Output = ApiGroupList;

    const METHOD: reqwest::Method = reqwest::Method::GET;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        \"/apis/\".into()
    }
}
",
        970,
    );
}

#[test]
fn test_builder_field_with_iterators() {
    assert_file_contains_content_at(
        &(ROOT.clone() + "/tests/test_k8s/io/k8s/api/certificates/v1beta1/certificate_signing_request_spec.rs"),
        "
impl<Request> CertificateSigningRequestSpecBuilder<Request> {
    /// Extra information about the requesting user. See user.Info interface for details.
    #[inline]
    pub fn extra(mut self, value: impl Iterator<Item = (String, impl Iterator<Item = impl Into<String>>)>) -> Self {
        self.inner.body.extra = Some(value.map(|(key, value)| (key, value.map(|value| value.into()).collect::<Vec<_>>())).collect::<std::collections::BTreeMap<_, _>>());
        self
    }

    /// Group information about the requesting user. See user.Info interface for details.
    #[inline]
    pub fn groups(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.inner.body.groups = Some(value.map(|value| value.into()).collect::<Vec<_>>());
        self
    }

    /// Base64-encoded PKCS#10 CSR data
    #[inline]
    pub fn request(mut self, value: impl Into<String>) -> CertificateSigningRequestSpecBuilder<crate::io::generics::RequestExists> {
        self.inner.body.request = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// UID information about the requesting user. See user.Info interface for details.
    #[inline]
    pub fn uid(mut self, value: impl Into<String>) -> Self {
        self.inner.body.uid = Some(value.into());
        self
    }

    /// allowedUsages specifies a set of usage contexts the key will be valid for. See: https://tools.ietf.org/html/rfc5280#section-4.2.1.3
    ///      https://tools.ietf.org/html/rfc5280#section-4.2.1.12
    #[inline]
    pub fn usages(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.inner.body.usages = Some(value.map(|value| value.into()).collect::<Vec<_>>());
        self
    }

    /// Information about the requesting user. See user.Info interface for details.
    #[inline]
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.inner.body.username = Some(value.into());
        self
    }
}
",
        2133,
    );
}
