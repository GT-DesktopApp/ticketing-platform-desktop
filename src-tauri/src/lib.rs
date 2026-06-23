// Application composition root.
//
// Wires the layers together (config -> db -> repository -> service), exposes the
// service through Tauri managed state, and registers the command surface.
// Layer dependencies point inward: commands -> service -> repository -> db.

mod commands;
mod config;
mod db;
mod domain;
mod error;
mod repository;
mod service;
mod sync;

use config::AppConfig;
use repository::{CategoryRepository, TicketRepository, UnitRepository, UserTypeRepository};
use service::{CategoryService, TicketService, UnitService, UserTypeService};
use tauri::Manager;

/// State shared with every command. Holds the application's services (not raw
/// pools), so command handlers stay thin and storage details stay hidden.
pub struct AppState {
    pub config: AppConfig,
    pub tickets: TicketService,
    pub categories: CategoryService,
    pub units: UnitService,
    pub user_types: UserTypeService,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::from_env();
    init_logging(&config.log_level);

    tauri::Builder::default()
        .setup({
            let config = config.clone();
            move |app| {
                let handle = app.handle().clone();

                // Open DB + run migrations before any command can be invoked.
                let pool = tauri::async_runtime::block_on(db::init(&handle, &config))
                    .expect("failed to initialise local database");

                // Resolve device + tenant identity (device_id persisted once).
                let identity = tauri::async_runtime::block_on(sync::identity::resolve(
                    &pool, &config,
                ))
                .expect("failed to resolve device identity");

                // Compose layers: repository over the pool, service over the repo.
                let tickets =
                    TicketService::new(TicketRepository::new(pool.clone()), identity.clone());
                let categories = CategoryService::new(CategoryRepository::new(pool.clone()));
                let units = UnitService::new(UnitRepository::new(pool.clone()));
                let user_types = UserTypeService::new(UserTypeRepository::new(pool.clone()));

                // Start the background sync drain (off the UI thread) and the
                // nightly safety-net scheduler (reconcile + snapshot @ 12:00 IST).
                let snapshot_dir = handle
                    .path()
                    .app_data_dir()
                    .map(|d| d.join("snapshots"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("snapshots"));
                sync::worker::spawn(pool.clone(), identity.clone(), config.clone());
                sync::nightly::spawn(pool, identity, config.clone(), snapshot_dir);

                app.manage(AppState {
                    config: config.clone(),
                    tickets,
                    categories,
                    units,
                    user_types,
                });

                tracing::info!("application initialised");
                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::ticket_commands::ticket_list,
            commands::ticket_commands::ticket_get,
            commands::ticket_commands::ticket_create,
            commands::ticket_commands::ticket_update,
            commands::ticket_commands::ticket_delete,
            commands::ticket_commands::ticket_validate,
            // Categories
            commands::category_commands::category_list,
            commands::category_commands::category_create,
            commands::category_commands::category_update,
            commands::category_commands::category_set_active,
            commands::category_commands::category_delete,
            // Units
            commands::unit_commands::unit_list,
            commands::unit_commands::unit_create,
            commands::unit_commands::unit_update,
            commands::unit_commands::unit_set_active,
            commands::unit_commands::unit_delete,
            // User types
            commands::user_type_commands::user_type_list,
            commands::user_type_commands::user_type_create,
            commands::user_type_commands::user_type_update,
            commands::user_type_commands::user_type_set_active,
            commands::user_type_commands::user_type_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Structured logging via tracing, controlled by APP_LOG_LEVEL.
fn init_logging(level: &str) {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));
    // Ignore the error if a global subscriber is already set (e.g. in tests).
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
