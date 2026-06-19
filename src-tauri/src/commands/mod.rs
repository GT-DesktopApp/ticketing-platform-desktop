// Commands layer: thin #[tauri::command] handlers. They pull services from
// managed state, delegate, and convert AppError -> String at the edge. No
// business logic or SQL here.

pub mod category_commands;
pub mod ticket_commands;
pub mod unit_commands;
pub mod user_type_commands;
