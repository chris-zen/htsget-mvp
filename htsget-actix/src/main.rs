use tokio::select;

use htsget_actix::run_server;
use htsget_actix::Config;
use htsget_search::storage::data_server::HttpTicketFormatter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  Config::setup_tracing()?;

  if let Some(config) = Config::parse_args() {
    let config = Config::from_config(config)?;

    if config.data_server().enabled() {
      let server = config.data_server().clone();
      let mut formatter = HttpTicketFormatter::try_from(server.clone())?;

      let local_server = formatter.bind_data_server().await?;
      let local_server =
        tokio::spawn(async move { local_server.serve(&server.local_path()).await });

      let ticket_server_config = config.ticket_server().clone();
      select! {
        local_server = local_server => Ok(local_server??),
        actix_server = run_server(
          config.owned_resolvers(),
          ticket_server_config,
        )? => actix_server
      }
    } else {
      let ticket_server_config = config.ticket_server().clone();
      run_server(config.owned_resolvers(), ticket_server_config)?.await
    }
  } else {
    Ok(())
  }
}
