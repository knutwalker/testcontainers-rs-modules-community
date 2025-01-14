use testcontainers::{core::WaitFor, Image};

const NAME: &str = "softwaremill/elasticmq";
const TAG: &str = "0.14.6";

#[derive(Debug, Default)]
pub struct ElasticMq;

impl Image for ElasticMq {
    type Args = ();

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Started SQS rest server")]
    }
}

#[cfg(test)]
mod tests {
    use aws_config::meta::region::RegionProviderChain;
    use aws_sdk_sqs::{config::Credentials, Client};
    use testcontainers::clients;

    use crate::elasticmq::ElasticMq;

    #[tokio::test]
    async fn sqs_list_queues() {
        let docker = clients::Cli::default();
        let node = docker.run(ElasticMq);
        let host_port = node.get_host_port_ipv4(9324);
        let client = build_sqs_client(host_port).await;

        let result = client.list_queues().send().await.unwrap();
        assert!(result.queue_urls.is_none());
    }

    async fn build_sqs_client(host_port: u16) -> Client {
        let endpoint_uri = format!("http://127.0.0.1:{host_port}");
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let creds = Credentials::new("fakeKey", "fakeSecret", None, None, "test");

        let shared_config = aws_config::from_env()
            .region(region_provider)
            .endpoint_url(endpoint_uri)
            .credentials_provider(creds)
            .load()
            .await;

        Client::new(&shared_config)
    }
}
