use bollard::query_parameters::{LogsOptionsBuilder, RestartContainerOptionsBuilder};
use serenity::futures::StreamExt;

use crate::GlobalData;

pub async fn restart_server(global_data: &GlobalData) -> Result<(), String> {
    println!("Restarting container: {}", global_data.container_name);
    let _ = &global_data
        .docker
        .restart_container(
            &global_data.container_name,
            Some(RestartContainerOptionsBuilder::new().t(30).build()),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_logs(global_data: &GlobalData) -> (Vec<String>, Vec<bollard::errors::Error>) {
    let logs = global_data.docker.logs(
        &global_data.container_name,
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
