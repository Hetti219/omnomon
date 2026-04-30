use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug, Clone)]
#[command(name = "omnomon", version, about = "Unified system monitor")]
pub struct CliArgs {
    /// Refresh rate in milliseconds (250–5000)
    #[arg(short = 'r', long, default_value_t = 1000)]
    pub rate: u64,

    /// Color theme: default, gruvbox, dracula, nord, catppuccin, solarized
    #[arg(short = 't', long)]
    pub theme: Option<String>,

    /// Config file path
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    /// Disable GPU monitoring
    #[arg(long)]
    pub no_gpu: bool,

    /// Show temperatures in Fahrenheit
    #[arg(long)]
    pub fahrenheit: bool,

    /// Enable debug logging to /tmp/omnomon.log
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct ConfigFile {
    pub general: GeneralConfig,
    pub theme: ThemeConfig,
    pub network: NetworkConfig,
    pub process: ProcessConfig,
    pub dashboard: DashboardConfig,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GeneralConfig {
    pub refresh_rate_ms: u64,
    pub temperature_unit: String,
    pub default_tab: String,
    pub graph_time_window: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: 1000,
            temperature_unit: "celsius".into(),
            default_tab: "dashboard".into(),
            graph_time_window: "60s".into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "default".into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct NetworkConfig {
    pub default_interface: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_interface: "auto".into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ProcessConfig {
    pub default_sort: String,
    pub show_gpu_column: bool,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            default_sort: "cpu".into(),
            show_gpu_column: true,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct DashboardConfig {
    pub show_battery: bool,
    pub show_thermal: bool,
    pub show_disk: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            show_battery: true,
            show_thermal: true,
            show_disk: true,
        }
    }
}

impl ConfigFile {
    pub fn load(path: Option<&PathBuf>) -> Self {
        let path = path
            .cloned()
            .or_else(|| dirs::config_dir().map(|d| d.join("omnomon/config.toml")));
        if let Some(p) = path {
            if let Ok(s) = std::fs::read_to_string(&p) {
                if let Ok(cfg) = toml::from_str::<ConfigFile>(&s) {
                    return cfg;
                }
            }
        }
        ConfigFile::default()
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub refresh_rate: Duration,
    pub theme_name: String,
    pub fahrenheit: bool,
    pub no_gpu: bool,
    pub graph_time_window: Duration,
    pub default_interface: String,
    pub default_sort: String,
    pub show_gpu_column: bool,
    pub show_battery: bool,
    pub show_thermal: bool,
    pub show_disk: bool,
    pub default_tab: String,
}

impl ResolvedConfig {
    pub fn from(args: &CliArgs, file: &ConfigFile) -> Self {
        let rate_ms = if args.rate != 1000 {
            args.rate
        } else {
            file.general.refresh_rate_ms.max(250).min(5000)
        };
        let theme_name = args
            .theme
            .clone()
            .unwrap_or_else(|| file.theme.name.clone());
        let fahrenheit = args.fahrenheit
            || file.general.temperature_unit.eq_ignore_ascii_case("fahrenheit");
        let graph_time_window = match file.general.graph_time_window.as_str() {
            "30s" => Duration::from_secs(30),
            "5m" => Duration::from_secs(300),
            _ => Duration::from_secs(60),
        };
        Self {
            refresh_rate: Duration::from_millis(rate_ms.clamp(250, 5000)),
            theme_name,
            fahrenheit,
            no_gpu: args.no_gpu,
            graph_time_window,
            default_interface: file.network.default_interface.clone(),
            default_sort: file.process.default_sort.clone(),
            show_gpu_column: file.process.show_gpu_column,
            show_battery: file.dashboard.show_battery,
            show_thermal: file.dashboard.show_thermal,
            show_disk: file.dashboard.show_disk,
            default_tab: file.general.default_tab.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_toml() {
        let toml = r#"
            [general]
            refresh_rate_ms = 500
            [theme]
            name = "gruvbox"
        "#;
        let cfg: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(cfg.general.refresh_rate_ms, 500);
        assert_eq!(cfg.theme.name, "gruvbox");
    }

    #[test]
    fn defaults_applied() {
        let cfg: ConfigFile = toml::from_str("").unwrap();
        assert_eq!(cfg.general.refresh_rate_ms, 1000);
        assert_eq!(cfg.process.default_sort, "cpu");
    }
}
