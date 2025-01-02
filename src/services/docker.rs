use bollard::{container::ListContainersOptions, Docker};
use crate::models::templates::ContainerStatus;
use log::error;

pub async fn get_containers() -> Vec<ContainerStatus> {
    match Docker::connect_with_local_defaults() {
        Ok(docker) => match docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
        {
            Ok(containers) => containers
                .into_iter()
                .map(|c| ContainerStatus {
                    image: c.image.unwrap_or_default(),
                    state: c.state.unwrap_or_default(),
                })
                .collect(),
            Err(_) => {
                error!(
                    "{}",
                    crate::models::errors::SystemError::DockerListContainersFailed.message()
                );
                vec![]
            }
        },
        Err(_) => {
            error!(
                "{}",
                crate::models::errors::SystemError::DockerConnectionFailed.message()
            );
            vec![]
        }
    }
}