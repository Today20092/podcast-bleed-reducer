//! Native project-save presentation; persistence and recovery are owned by `Project`.
use super::CleanSlateApp;

impl CleanSlateApp {
    pub(super) fn save_project(&mut self, choose: bool) -> bool {
        let Some(project) = self.project.as_mut() else {
            return false;
        };
        let path = if choose || project.save_destination().is_none() {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("CleanSlate project", &["cleanslate"])
                .set_file_name("Episode.cleanslate");
            if let Some(destination) = project.save_destination() {
                if let Some(parent) = destination.parent() {
                    dialog = dialog.set_directory(parent);
                }
                if let Some(name) = destination.file_name() {
                    dialog = dialog.set_file_name(name.to_string_lossy());
                }
            }
            let Some(path) = dialog.save_file() else {
                return false;
            };
            Some(path)
        } else {
            None
        };
        let result = if let Some(path) = path {
            project.save_as(&path)
        } else {
            project.save_named()
        };
        match result {
            Ok(()) => {
                self.messages = vec![format!(
                    "Project saved to {}",
                    project.save_destination().unwrap().display()
                )];
                if let Some(warning) = project.save_warning() {
                    self.messages.push(warning.into());
                }
                true
            }
            Err(error) => {
                self.messages.push(format!(
                    "Project was not saved. Local recovery and work are retained: {error}"
                ));
                false
            }
        }
    }
}
