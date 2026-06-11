use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

const GALENE_PORT: u16 = 8443;
const GALENE_ROOM: &str = "classroom";

pub struct VideoRuntime {
    child: Mutex<Option<Child>>,
    port: u16,
    room: String,
}

impl VideoRuntime {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            port: GALENE_PORT,
            room: GALENE_ROOM.into(),
        }
    }

    pub fn galene_binary(app_data: &Path) -> PathBuf {
        app_data.join("galene").join(if cfg!(windows) {
            "galene.exe"
        } else {
            "galene"
        })
    }

    pub fn galene_installed(app_data: &Path) -> bool {
        Self::galene_binary(app_data).exists()
    }

    pub fn start(&self, app_data: &Path) -> Result<String, String> {
        self.stop();

        let binary = Self::galene_binary(app_data);
        if !binary.exists() {
            return Err(
                "Galene not installed. Run: powershell -File scripts/install-galene.ps1".into(),
            );
        }

        let config_dir = app_data.join("galene");
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
        let config_path = config_dir.join("config.json");
        std::fs::write(&config_path, default_galene_config()).map_err(|e| e.to_string())?;

        let data_path = config_dir.join("data");
        std::fs::create_dir_all(&data_path).map_err(|e| e.to_string())?;

        let child = Command::new(&binary)
            .arg("-config")
            .arg(&config_path)
            .arg("-datadir")
            .arg(&data_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start Galene: {e}"))?;

        *self.child.lock().map_err(|e| e.to_string())? = Some(child);

        std::thread::sleep(std::time::Duration::from_millis(800));

        let ip = local_ip_address::local_ip()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "127.0.0.1".into());

        Ok(format!(
            "http://{}:{}/group/{}/",
            ip, self.port, self.room
        ))
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.child
            .lock()
            .ok()
            .and_then(|mut g| g.as_mut().map(|c| c.try_wait().ok().flatten().is_none()))
            .unwrap_or(false)
    }

    pub fn status(&self, app_data: &Path) -> crate::models::VideoStatus {
        let installed = Self::galene_installed(app_data);
        let running = self.is_running();
        let ip = local_ip_address::local_ip()
            .ok()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "127.0.0.1".into());
        let url = if running {
            Some(format!("http://{}:{}/group/{}/", ip, self.port, self.room))
        } else {
            None
        };
        let message = if !installed {
            "Install Galene with scripts/install-galene.ps1 for live video.".into()
        } else if running {
            "Galene video room is live.".into()
        } else {
            "Galene installed but not running.".into()
        };

        crate::models::VideoStatus {
            running,
            port: self.port,
            room: self.room.clone(),
            url,
            galene_installed: installed,
            message,
        }
    }
}

fn default_galene_config() -> String {
    r#"{
  "proxyURL": "http://localhost:8443/",
  "users": {
    "teacher": { "password": "classmate", "permissions": "op" }
  },
  "groups": {
    "classroom": {
      "public": true,
      "displayName": "ClassMate Live Class",
      "allowRecording": true,
      "allowAnonymous": true,
      "codecs": ["vp8", "opus"]
    }
  }
}"#
    .to_string()
}
