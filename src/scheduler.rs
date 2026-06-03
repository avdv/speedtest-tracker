use crate::{db::Database, speedtest};
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct SpeedtestScheduler {
    scheduler: JobScheduler,
}

impl SpeedtestScheduler {
    pub async fn new(db: Database) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        
        // Read schedule from environment variables
        let schedule_expr_var = env::var("SPEEDTEST_SCHEDULE");
        println!("SPEEDTEST_SCHEDULE: {:?}", schedule_expr_var);
        let schedule_expr = schedule_expr_var.unwrap_or_else(|_| "0 0 * * * *".to_string()); // Default: every hour
        
        let server_ids_str = env::var("SPEEDTEST_SERVERS").ok();
        
        tracing::info!("Configuring speedtest scheduler with cron: {}", schedule_expr);
        
        // Convert 5-field cron to 6-field (add seconds at start) if needed
        let schedule_expr = if schedule_expr.split_whitespace().count() == 5 {
            format!("0 {}", schedule_expr)
        } else {
            schedule_expr
        };
        
        tracing::info!("Using cron expression (with seconds): {}", schedule_expr);
        
        if let Some(ref servers) = server_ids_str {
            tracing::info!("Will use servers: {}", servers);
        } else {
            tracing::info!("Will use random servers");
        }
        
        // Clone values for the job closure
        let db_clone = db.clone();
        let servers_clone = server_ids_str.clone();
        
        // Create the scheduled job
        let job = Job::new_async(schedule_expr.as_str(), move |_uuid, _lock| {
            let db = db_clone.clone();
            let servers = servers_clone.clone();
            
            Box::pin(async move {
                tracing::info!("Starting scheduled speedtest");
                
                // Parse server IDs if provided
                let server_id = if let Some(server_str) = servers {
                    let ids: Vec<i64> = server_str
                        .split(',')
                        .filter_map(|s| s.trim().parse::<i64>().ok())
                        .collect();
                    
                    if !ids.is_empty() {
                        // Pick a random server from the list
                        use rand::Rng;
                        let idx = rand::thread_rng().gen_range(0..ids.len());
                        Some(ids[idx])
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                // Run speedtest
                match speedtest::run_speedtest(server_id).await {
                    Ok(result) => {
                        let download_mbps = result.download as f64 * 8.0 / 1_000_000.0;
                        let upload_mbps = result.upload as f64 * 8.0 / 1_000_000.0;
                        
                        tracing::info!(
                            "Scheduled speedtest completed: download={:.2} Mbps, upload={:.2} Mbps",
                            download_mbps,
                            upload_mbps
                        );
                        
                        // Save result to database with scheduled=true
                        if let Err(e) = speedtest::save_result(&db, result, true).await {
                            tracing::error!("Failed to save scheduled speedtest result: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Scheduled speedtest failed: {}", e);
                    }
                }
            })
        })?;
        
        scheduler.add(job).await?;
        
        Ok(Self { scheduler })
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.start().await?;
        tracing::info!("Speedtest scheduler started");
        Ok(())
    }
}
