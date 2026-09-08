use async_trait::async_trait;
use std::time::Duration;
use tauri::{Emitter, Runtime};
use tauri_plugin_background_service::{BackgroundService, ServiceContext, ServiceError};

pub struct AirplayService {
    tick_count: u64,
}

impl AirplayService {
    pub fn new() -> Self {
        Self { tick_count: 0 }
    }
}

#[async_trait]
impl<R: Runtime> BackgroundService<R> for AirplayService {
    async fn init(&mut self, _ctx: &ServiceContext<R>) -> Result<(), ServiceError> {
        // One-time setup: load config, open handles, seed state
        Ok(())
    }

    async fn run(&mut self, ctx: &ServiceContext<R>) -> Result<(), ServiceError> {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = ctx.shutdown.cancelled() => {
                    let _ = ctx.app.emit("service-stopped", self.tick_count);
                    break;
                }
                _ = interval.tick() => {
                    self.tick_count += 1;

                    if self.tick_count == 1 {
                        ctx.notifier.show("Background Service", "Service is running");
                    }

                    let _ = ctx.app.emit("service-tick", self.tick_count);
                }
            }
        }

        Ok(())
    }
}