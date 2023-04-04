mod api;
mod rabbit;

use futures::join;
use crate::api::serve;
use crate::rabbit::{init_rabbit, rmq_listen};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let actix_thread = serve("localhost");
    let rmq_thread = rmq_listen(init_rabbit().clone());
    let _ = join!(
        actix_thread,
    rmq_thread
    );
    Ok(())
}





