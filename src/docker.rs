use bollard::query_parameters::{
    AttachContainerOptionsBuilder, LogsOptionsBuilder, RestartContainerOptionsBuilder,
};
use serenity::futures::StreamExt;

use crate::ServerState;

pub async fn restart_server(server_state: &ServerState) -> Result<(), String> {
    let container_name = &server_state.bot_config.container_name;
    log::info!(
        "Restarting container: {}",
        container_name
    );
    let _ = &server_state
        .docker
        .restart_container(
            container_name,
            Some(RestartContainerOptionsBuilder::new().t(30).build()),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_logs(global_data: &ServerState) -> (Vec<String>, Vec<bollard::errors::Error>) {
    let logs = global_data.docker.logs(
        &global_data.bot_config.container_name,
        Some(
            LogsOptionsBuilder::new()
                .tail("20")
                .stdout(true)
                .stderr(true)
                .build(),
        ),
    );

    let (oks, errs): (Vec<_>, Vec<_>) = logs
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .partition(Result::is_ok);

    let ok_logs: Vec<_> = oks
        .into_iter()
        .filter_map(Result::ok)
        .map(|log| log.to_string())
        .collect();
    let errors: Vec<_> = errs.into_iter().filter_map(Result::err).collect();
    if !errors.is_empty() {
        log::error!("{errors:?}");
    }

    (ok_logs, errors)
}

pub async fn attach_and_listen<Fut>(
    server_state: &ServerState,
    func: impl Fn(String) -> Fut,
) -> Result<(), bollard::errors::Error>
where
    Fut: Future<Output = ()>,
{
    let container_name = &server_state.bot_config.container_name;
    log::debug!("Attaching to container: {container_name}");
    let mut attachment = server_state
        .docker
        .attach_container(
            container_name,
            Some(
                AttachContainerOptionsBuilder::new()
                    .stdout(true)
                    .stderr(true)
                    .stream(true)
                    .build(),
            ),
        )
        .await
        .expect("Could not attach to the container");

    while let Some(line) = attachment.output.next().await {
        let line = line?;
        match line {
            bollard::container::LogOutput::StdErr { message }
            | bollard::container::LogOutput::StdOut { message } => {
                let message = match str::from_utf8(&message) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!("Could not parse msg as utf8: {e:?}");
                        continue;
                    }
                };

                let messages: Vec<_> = message.lines().collect();
                for message in messages {
                    log::trace!("Received message from container {container_name}: {message}");
                    func(message.to_string()).await;
                }
            }
            // bollard::container::LogOutput::StdIn { message } => todo!(),
            // bollard::container::LogOutput::Console { message } => todo!(),
            _ => {}
        }
    }

    Ok(())
}
