/// Cached system statistics for the header bar
#[derive(Debug, Clone)]
pub struct SysStats {
    pub cpu_pct: f32,
    pub mem_used_gb: f32,
    pub mem_total_gb: f32,
    pub cpu_brand: String,
    pub temp_c: Option<f32>,
    pub temp_label: String,
}
