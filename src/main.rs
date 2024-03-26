use console::style;
use dialoguer::theme::ColorfulTheme;
use neopass::config::INACTIVITY_DELAY;
use neopass::entry::{add_a_new_entry, modify_entry};
use neopass::select::{Select, SelectOutput};
use neopass::utils::{
    add_first_entry, build_rows, clear_screen, display_instructions, display_password_copied,
    get_user_password, set_password_in_clipboard, write_entries_in_file,
};
use std::error::Error;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let mut password = String::new();
    let mut entries = Vec::new();

    clear_screen()?;
    get_user_password(&mut entries, &mut password)?;

    let mut copied_item = None;
    let mut _last_activity = Instant::now();

    loop {
        if entries.is_empty() {
            add_first_entry(&mut entries, &mut password)?;
            _last_activity = Instant::now();
            continue;
        }

        display_instructions();

        if copied_item.is_some() {
            display_password_copied();
        }

        let mut rows = build_rows(&entries);

        let header = format!(
            "  {}\n  {}\n  {}",
            rows.remove(0),
            rows.remove(0),
            rows.remove(0)
        );
        let footer = format!("  {}\n", rows.remove(rows.len() - 1));
        let theme = ColorfulTheme {
            header: style(header).for_stderr(),
            footer: style(footer).for_stderr(),
            ..Default::default()
        };

        // Reset the timer on user activity
        _last_activity = Instant::now();

        // Display entries.
        if let Some(selection) = Select::with_theme(&theme)
            .default(copied_item.unwrap_or_default())
            .items(&rows)
            .interact_opt()?
        {
            copied_item = None;

            // Check if INACTIVITY_DELAY seconds have elapsed since the last activity
            if _last_activity.elapsed() >= Duration::from_secs(INACTIVITY_DELAY) {
                clear_screen()?;
                get_user_password(&mut entries, &mut password)?;
                _last_activity = Instant::now();
            }

            match selection {
                // User selected one item.
                SelectOutput::Copy(index) => {
                    set_password_in_clipboard(&entries, index, &mut copied_item)?;
                }

                // User wants to add a new item.
                SelectOutput::Add => {
                    clear_screen()?;
                    add_a_new_entry(&mut entries);
                    write_entries_in_file(&entries, &password)?;
                }

                // User wants to delete an item.
                SelectOutput::Delete(index) => {
                    let _removed_instance = entries.remove(index);
                    write_entries_in_file(&entries, &password)?;
                }

                // User wants to modify one item.
                SelectOutput::Modify(index) => {
                    clear_screen()?;
                    modify_entry(&mut entries, index);
                    write_entries_in_file(&entries, &password)?;
                }
            }
        } else {
            return Ok(());
        }
        clear_screen()?;
    }
}
