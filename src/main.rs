use futures_util::stream::StreamExt;
use tokio::{select, signal};
use zbus::{Connection, proxy};
use zvariant::OwnedValue;

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: OwnedValue) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;

    let proxy = SettingsProxy::new(&conn).await?;

    let mut stream = proxy.receive_setting_changed().await?;

    loop {
        select! {
            _ = signal::ctrl_c() => {
                break;
            }
            Some(signal) = stream.next() => {
                let args = signal.args()?;
                println!(
                    "Changed: {}.{} = {:?}",
                    args.namespace, args.key, args.value
                );
            }
        }
    }

    Ok(())
}
