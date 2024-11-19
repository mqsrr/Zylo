use testcontainers::{ContainerAsync, GenericImage};
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;

pub async fn get_connection_info(container: &ContainerAsync<GenericImage>, internal_port: u16) -> (String, u16){
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(internal_port).await.unwrap();

    (host.to_string(), port)
}


pub async fn create_redis() -> ContainerAsync<GenericImage> {
    GenericImage::new("redis", "alpine")
        .with_exposed_port(ContainerPort::Tcp(6379))
        .with_wait_for(WaitFor::seconds(5))
        .start()
        .await
        .unwrap()
}


pub async fn create_mongo() -> ContainerAsync<GenericImage> {
    GenericImage::new("mongo", "8.0.1")
        .with_exposed_port(ContainerPort::Tcp(27017))
        .with_wait_for(WaitFor::seconds(5))
        .start()
        .await
        .unwrap()
}

pub async fn create_rabbitmq() -> ContainerAsync<GenericImage> {
    GenericImage::new("rabbitmq", "alpine")
        .with_exposed_port(ContainerPort::Tcp(5672))
        .with_wait_for(WaitFor::seconds(5))
        .start()
        .await
        .unwrap()
}
