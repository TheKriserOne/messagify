
#[macro_export]
macro_rules! callbacks {
    () => {
        tauri::generate_handler![
            // Discord commands
            crate::messangers::discord::api::set_token,
            crate::messangers::discord::api::get_token,   
        ]
    };
}
