use crate::errors::*;
use crate::providers::aws;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn test_aws_basic() {
    let ep = "/meta-data/public-keys";
    let client = crate::retry::Client::try_new()
        .chain_err(|| "failed to create http client")
        .unwrap()
        .max_retries(0)
        .return_on_404(true);
    let provider = aws::AwsProvider { client };

    provider.fetch_ssh_keys().unwrap_err();

    let _m = mockito::mock("GET", ep).with_status(503).create();
    provider.fetch_ssh_keys().unwrap_err();

    let _m = mockito::mock("GET", ep).with_status(200).create();
    let v = provider.fetch_ssh_keys().unwrap();
    assert_eq!(v.len(), 0);

    let _m = mockito::mock("GET", ep).with_status(404).create();
    let v = provider.fetch_ssh_keys().unwrap();
    assert_eq!(v.len(), 0);

    mockito::reset();
    provider.fetch_ssh_keys().unwrap_err();
}

#[test]
fn test_aws_attributes() {
    let instance_id = "test-instance-id";
    let instance_type = "test-instance-type";
    let ipv4_local = "test-ipv4-local";
    let ipv4_public = "test-ipv4-public";
    let availability_zone = "test-availability-zone";
    let hostname = "test-hostname";
    let public_hostname = "test-public-hostname";
    let instance_id_doc = r#"{"region": "test-region"}"#;
    let region = "test-region";

    let endpoints = maplit::btreemap! {
        "/meta-data/instance-id" => instance_id,
        "/meta-data/instance-type" => instance_type,
        "/meta-data/local-ipv4" => ipv4_local,
        "/meta-data/public-ipv4" => ipv4_public,
        "/meta-data/placement/availability-zone" => availability_zone,
        "/meta-data/hostname" => hostname,
        "/meta-data/public-hostname" => public_hostname,
        "/dynamic/instance-identity/document" => instance_id_doc,
    };

    let mut mocks = Vec::with_capacity(endpoints.len());
    for (endpoint, body) in endpoints {
        let m = mockito::mock("GET", endpoint)
            .with_status(200)
            .with_body(body)
            .create();
        mocks.push(m);
    }

    let attributes = maplit::hashmap! {
        format!("{}_INSTANCE_ID", aws::ENV_PREFIX) => instance_id.to_string(),
        format!("{}_INSTANCE_TYPE", aws::ENV_PREFIX) => instance_type.to_string(),
        format!("{}_IPV4_LOCAL", aws::ENV_PREFIX) => ipv4_local.to_string(),
        format!("{}_IPV4_PUBLIC", aws::ENV_PREFIX) => ipv4_public.to_string(),
        format!("{}_AVAILABILITY_ZONE", aws::ENV_PREFIX) => availability_zone.to_string(),
        format!("{}_HOSTNAME", aws::ENV_PREFIX) => hostname.to_string(),
        format!("{}_PUBLIC_HOSTNAME", aws::ENV_PREFIX) => public_hostname.to_string(),
        format!("{}_REGION", aws::ENV_PREFIX) => region.to_string(),
    };

    let client = crate::retry::Client::try_new()
        .chain_err(|| "failed to create http client")
        .unwrap()
        .max_retries(0)
        .return_on_404(true);
    let provider = aws::AwsProvider { client };

    let v = provider.attributes().unwrap();
    assert_eq!(v, attributes);

    mockito::reset();
    provider.attributes().unwrap_err();
}
